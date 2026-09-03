use dioxus::prelude::*;
use inventory_common::dto::CreateProductRequest;
use inventory_common::Product;
use rust_decimal::Decimal;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::badge::{Badge, BadgeVariant};
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::modal::Modal;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{AppShell, require_auth};

#[component]
pub fn Products() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let products = use_signal(Vec::<Product>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let mut show_modal = use_signal(|| false);
    let mut refresh = use_signal(|| 0);

    let load_data = move || {
        let client = auth.api_client();
        let mut products_set = products;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => match client.list_products(current_page, 10).await {
                    Ok(response) => {
                        products_set.set(response.items);
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

    let columns: Vec<Column<Product>> = vec![
        Column {
            key: "name".to_string(),
            header: "Nombre".to_string(),
            render: |p| rsx! { span { "{p.name}" } },
        },
        Column {
            key: "sku".to_string(),
            header: "SKU".to_string(),
            render: |p| rsx! { span { class: "text-muted", "{p.sku.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "price".to_string(),
            header: "Precio".to_string(),
            render: |p| rsx! { span { "${p.price}" } },
        },
        Column {
            key: "stock".to_string(),
            header: "Stock".to_string(),
            render: |p| rsx! {
                if p.stock <= p.min_stock {
                    Badge { variant: BadgeVariant::Danger, "{p.stock}" }
                } else {
                    span { "{p.stock}" }
                }
            },
        },
    ];

    let rows = products.read().clone();

    rsx! {
        AppShell { title: "Productos".to_string(), active_route: "/products".to_string(),
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Listado de productos".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_modal.set(true),
                        "Nuevo producto"
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
            ProductFormModal {
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

#[component]
fn ProductFormModal(show: bool, on_close: EventHandler<()>, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut name = use_signal(|| "".to_string());
    let mut sku = use_signal(|| "".to_string());
    let mut price = use_signal(|| "".to_string());
    let mut cost = use_signal(|| "".to_string());
    let mut stock = use_signal(|| "".to_string());
    let mut min_stock = use_signal(|| "".to_string());
    let saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let on_submit = move |_| {
        let name_value = name.read().trim().to_string();
        if name_value.is_empty() {
            form_error.set(Some("El nombre es obligatorio".to_string()));
            return;
        }
        let price_dec = price.read().parse::<Decimal>().unwrap_or(Decimal::ZERO);
        let cost_dec = cost.read().parse::<Decimal>().unwrap_or(Decimal::ZERO);
        let stock_i = stock.read().parse::<i32>().unwrap_or(0);
        let min_stock_i = min_stock.read().parse::<i32>().unwrap_or(0);

        let request = CreateProductRequest {
            name: name_value,
            description: None,
            category: None,
            brand: None,
            model: None,
            sku: Some(sku.read().clone()).filter(|s| !s.is_empty()),
            price: price_dec,
            cost: cost_dec,
            stock: stock_i,
            min_stock: min_stock_i,
            location: None,
            supplier_id: None,
        };

        let client = auth.api_client();
        let mut saving_set = saving;
        let mut error_set = form_error;
        let on_saved_clone = on_saved;

        saving_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => match client.create_product(&request).await {
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
            title: "Nuevo producto".to_string(),
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
            div { class: "form-row",
                Input {
                    label: Some("Nombre *".to_string()),
                    value: name.read().clone(),
                    oninput: move |evt: FormEvent| name.set(evt.value().clone()),
                }
                Input {
                    label: Some("SKU".to_string()),
                    value: sku.read().clone(),
                    oninput: move |evt: FormEvent| sku.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Precio *".to_string()),
                    r#type: "number".to_string(),
                    value: price.read().clone(),
                    oninput: move |evt: FormEvent| price.set(evt.value().clone()),
                }
                Input {
                    label: Some("Costo".to_string()),
                    r#type: "number".to_string(),
                    value: cost.read().clone(),
                    oninput: move |evt: FormEvent| cost.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Stock *".to_string()),
                    r#type: "number".to_string(),
                    value: stock.read().clone(),
                    oninput: move |evt: FormEvent| stock.set(evt.value().clone()),
                }
                Input {
                    label: Some("Stock mínimo".to_string()),
                    r#type: "number".to_string(),
                    value: min_stock.read().clone(),
                    oninput: move |evt: FormEvent| min_stock.set(evt.value().clone()),
                }
            }
        }
    }
}
