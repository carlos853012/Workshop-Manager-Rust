use dioxus::prelude::*;
use rust_decimal::Decimal;
use workshop_common::dto::CreateProductRequest;
use workshop_common::Product;

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::molecules::modal::Modal;

#[component]
pub fn ProductFormModal(
    show: bool,
    edit_product: Option<Product>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let is_edit = edit_product.is_some();

    let mut name = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default()
    });
    let mut sku = use_signal(|| {
        edit_product
            .as_ref()
            .and_then(|p| p.sku.clone())
            .unwrap_or_default()
    });
    let mut barcode = use_signal(|| {
        edit_product
            .as_ref()
            .and_then(|p| p.barcode.clone())
            .unwrap_or_default()
    });
    let mut price = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.price.trunc().to_string())
            .unwrap_or_default()
    });
    let mut cost = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.cost.trunc().to_string())
            .unwrap_or_default()
    });
    let mut stock = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.stock.to_string())
            .unwrap_or_default()
    });
    let mut min_stock = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.min_stock.to_string())
            .unwrap_or_default()
    });
    let saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let title = if is_edit {
        "Editar producto"
    } else {
        "Nuevo producto"
    };

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
            barcode: Some(barcode.read().clone()).filter(|s| !s.is_empty()),
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
        let edit_id = edit_product.as_ref().map(|p| p.id);

        saving_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => {
                    let result = if let Some(id) = edit_id {
                        client.update_product(id, &request).await
                    } else {
                        client.create_product(&request).await
                    };
                    match result {
                        Ok(_) => {
                            on_saved_clone.call(());
                        }
                        Err(e) => {
                            error_set.set(Some(e.user_message().to_string()));
                        }
                    }
                }
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
            title: title.to_string(),
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                Button {
                    class: Some("cancel-button".to_string()),
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
                    label: Some("Código de barras".to_string()),
                    value: barcode.read().clone(),
                    oninput: move |evt: FormEvent| barcode.set(evt.value().clone()),
                    placeholder: Some("Vacío = auto-generar".to_string()),
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
