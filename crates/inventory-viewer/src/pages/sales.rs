use dioxus::prelude::*;
use inventory_common::dto::{CreateSaleRequest, SaleItemRequest};
use inventory_common::{PaymentMethod, Sale};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::modal::Modal;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{AppShell, require_auth};

#[component]
pub fn Sales() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let sales = use_signal(Vec::<Sale>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let mut show_modal = use_signal(|| false);
    let mut refresh = use_signal(|| 0);

    let load_data = move || {
        let client = auth.api_client();
        let mut sales_set = sales;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => match client.list_sales(current_page, 10).await {
                    Ok(response) => {
                        sales_set.set(response.items);
                        total_set.set(response.total);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        error_set.set(Some("Sesión expirada".to_string()));
                    }
                    Err(e) => {
                        error_set.set(Some(e.to_string()));
                    }
                },
                None => {
                    error_set.set(Some("No hay cliente API".to_string()));
                }
            }
            loading_set.set(false);
        });
    };

    use_effect(move || {
        let _ = *refresh.read();
        load_data();
    });

    let columns: Vec<Column<Sale>> = vec![
        Column {
            key: "customer".to_string(),
            header: "Cliente".to_string(),
            render: |s| rsx! { span { "{s.customer_name.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "payment".to_string(),
            header: "Pago".to_string(),
            render: |s| rsx! { span { "{s.payment_method}" } },
        },
        Column {
            key: "total".to_string(),
            header: "Total".to_string(),
            render: |s| rsx! { span { "${s.total}" } },
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: |s| rsx! { span { class: "badge", "{s.status}" } },
        },
    ];

    let rows = sales.read().clone();

    rsx! {
        AppShell { title: "Ventas".to_string(), active_route: "/sales".to_string(),
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Listado de ventas".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_modal.set(true),
                        "Nueva venta"
                    }
                },
                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
                } else {
                    DataTable {
                        columns: columns.clone(),
                        rows: rows.clone(),
                        page: *page.read(),
                        total: *total.read(),
                        on_page_change: move |new_page: i32| page.set(new_page),
                    }
                }
            }
            SaleFormModal {
                show: *show_modal.read(),
                on_close: move |_| show_modal.set(false),
                on_saved: move |_| {
                    show_modal.set(false);
                    let current = *refresh.read();
                    refresh.set(current + 1);
                },
            }
        }
    }
}

#[derive(Clone)]
struct SaleItemForm {
    product_id: String,
    quantity: String,
    unit_price: String,
}

#[component]
fn SaleFormModal(show: bool, on_close: EventHandler<()>, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut customer_name = use_signal(|| "".to_string());
    let mut customer_email = use_signal(|| "".to_string());
    let mut payment_method = use_signal(|| "cash".to_string());
    let mut items = use_signal(|| vec![SaleItemForm {
        product_id: "".to_string(),
        quantity: "1".to_string(),
        unit_price: "".to_string(),
    }]);
    let saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let add_item = move |_| {
        let mut current = items.read().clone();
        current.push(SaleItemForm {
            product_id: "".to_string(),
            quantity: "1".to_string(),
            unit_price: "".to_string(),
        });
        items.set(current);
    };

    let mut remove_item = move |index: usize| {
        let mut current = items.read().clone();
        if current.len() > 1 {
            current.remove(index);
            items.set(current);
        }
    };

    let mut update_item = move |index: usize, field: &'static str, value: String| {
        let mut current = items.read().clone();
        if let Some(item) = current.get_mut(index) {
            match field {
                "product_id" => item.product_id = value,
                "quantity" => item.quantity = value,
                "unit_price" => item.unit_price = value,
                _ => {}
            }
        }
        items.set(current);
    };

    let on_submit = move |_| {
        let mut parsed_items = Vec::new();
        for item in items.read().iter() {
            let product_id = match Uuid::parse_str(&item.product_id) {
                Ok(id) => id,
                Err(_) => {
                    form_error.set(Some("ID de producto inválido".to_string()));
                    return;
                }
            };
            let quantity = item.quantity.parse::<i32>().unwrap_or(0);
            if quantity <= 0 {
                form_error.set(Some("Cantidad debe ser mayor a 0".to_string()));
                return;
            }
            let unit_price = item.unit_price.parse::<Decimal>().unwrap_or(Decimal::ZERO);
            parsed_items.push(SaleItemRequest {
                product_id,
                quantity,
                unit_price,
            });
        }

        let payment = match payment_method.read().as_str() {
            "card" => PaymentMethod::Card,
            "transfer" => PaymentMethod::Transfer,
            _ => PaymentMethod::Cash,
        };

        let request = CreateSaleRequest {
            customer_name: Some(customer_name.read().clone()).filter(|s| !s.is_empty()),
            customer_email: Some(customer_email.read().clone()).filter(|s| !s.is_empty()),
            customer_phone: None,
            payment_method: payment,
            items: parsed_items,
        };

        let client = auth.api_client();
        let mut saving_set = saving;
        let mut error_set = form_error;
        let on_saved_clone = on_saved;

        saving_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => match client.create_sale(&request).await {
                    Ok(_) => {
                        on_saved_clone.call(());
                    }
                    Err(e) => {
                        error_set.set(Some(e.to_string()));
                    }
                },
                None => {
                    error_set.set(Some("No hay cliente API".to_string()));
                }
            }
            saving_set.set(false);
        });
    };

    rsx! {
        Modal {
            show: show,
            title: "Nueva venta".to_string(),
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_close.call(()),
                    "Cancelar"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    loading: *saving.read(),
                    onclick: on_submit,
                    "Guardar"
                }
            },
            if let Some(err) = form_error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Input {
                label: Some("Cliente".to_string()),
                value: customer_name.read().clone(),
                oninput: move |evt: FormEvent| customer_name.set(evt.value().clone()),
            }
            div { class: "mt-md" }
            Input {
                label: Some("Email".to_string()),
                r#type: "email".to_string(),
                value: customer_email.read().clone(),
                oninput: move |evt: FormEvent| customer_email.set(evt.value().clone()),
            }
            div { class: "mt-md" }
            div { class: "form-group",
                label { class: "form-label", "Método de pago" }
                select {
                    class: "input",
                    value: "{payment_method.read()}",
                    onchange: move |evt: Event<FormData>| payment_method.set(evt.value().clone()),
                    option { value: "cash", "Efectivo" }
                    option { value: "card", "Tarjeta" }
                    option { value: "transfer", "Transferencia" }
                }
            }
            div { class: "mt-md" }
            h4 { class: "font-semibold mb-md", "Items" }
            for (index, item) in items.read().iter().enumerate() {
                div { class: "form-row",
                    Input {
                        label: Some("Producto ID".to_string()),
                        value: item.product_id.clone(),
                        oninput: move |evt: FormEvent| update_item(index, "product_id", evt.value().clone()),
                    }
                    Input {
                        label: Some("Cantidad".to_string()),
                        r#type: "number".to_string(),
                        value: item.quantity.clone(),
                        oninput: move |evt: FormEvent| update_item(index, "quantity", evt.value().clone()),
                    }
                    Input {
                        label: Some("Precio unitario".to_string()),
                        r#type: "number".to_string(),
                        value: item.unit_price.clone(),
                        oninput: move |evt: FormEvent| update_item(index, "unit_price", evt.value().clone()),
                    }
                    Button {
                        variant: ButtonVariant::Danger,
                        onclick: move |_| remove_item(index),
                        "X"
                    }
                }
            }
            div { class: "mt-md" }
            Button {
                variant: ButtonVariant::Ghost,
                onclick: add_item,
                "+ Agregar item"
            }
        }
    }
}
