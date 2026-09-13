use dioxus::prelude::*;
use inventory_common::dto::{CreateProductRequest, PosProductResponse};

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::molecules::modal::Modal;

#[component]
pub fn StockEntryModal(
    show: bool,
    product: Option<PosProductResponse>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let mut quantity = use_signal(|| "".to_string());
    let saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let has_product = product.is_some();
    let product_name_display = product.as_ref().map(|p| p.name.clone()).unwrap_or_default();
    let current_stock = product.as_ref().map(|p| p.stock).unwrap_or(0);
    let product_id = product.as_ref().map(|p| p.product_id);
    let submit_sku = product.as_ref().and_then(|p| p.sku.clone());
    let submit_barcode = product.as_ref().and_then(|p| p.barcode.clone());
    let submit_price = product.as_ref().map(|p| p.price).unwrap_or_default();
    let submit_cost = product.as_ref().map(|p| p.cost).unwrap_or_default();
    let submit_min_stock = product.as_ref().map(|p| p.min_stock).unwrap_or(0);
    let submit_supplier_id = product.as_ref().and_then(|p| p.supplier_id);

    let on_submit = {
        let product_name_display = product_name_display.clone();
        move |_| {
            let qty = match quantity.read().parse::<i32>() {
                Ok(q) if q > 0 => q,
                _ => {
                    form_error.set(Some("Ingrese una cantidad válida".to_string()));
                    return;
                }
            };
            let pid = match product_id {
                Some(id) => id,
                None => return,
            };
            let client = match auth.api_client() {
                Some(c) => c,
                None => {
                    form_error.set(Some("No hay cliente API".to_string()));
                    return;
                }
            };
            let product_name = product_name_display.clone();
            let sku = submit_sku.clone();
            let barcode = submit_barcode.clone();
            let price = submit_price;
            let cost = submit_cost;
            let min_stock = submit_min_stock;
            let supplier_id = submit_supplier_id;
            let mut saving_clone = saving;
            let mut error_clone = form_error;
            let on_saved_clone = on_saved;

            saving_clone.set(true);
            error_clone.set(None);

            spawn(async move {
                let new_stock = current_stock + qty;
                let request = CreateProductRequest {
                    name: product_name,
                    description: None,
                    category: None,
                    brand: None,
                    model: None,
                    sku,
                    barcode,
                    price,
                    cost,
                    stock: new_stock,
                    min_stock,
                    location: None,
                    supplier_id,
                };
                match client.update_product(pid, &request).await {
                    Ok(_) => {
                        on_saved_clone.call(());
                    }
                    Err(e) => {
                        error_clone.set(Some(e.user_message().to_string()));
                    }
                }
                saving_clone.set(false);
            });
        }
    };

    if !show || !has_product {
        return rsx! {};
    }

    rsx! {
        Modal {
            show: true,
            title: "Ingreso de stock".to_string(),
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
                    "Confirmar"
                }
            },
            if let Some(err) = form_error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            div { class: "stock-entry-info",
                p { "Producto: " strong { "{product_name_display}" } }
                p { "Stock actual: " strong { "{current_stock}" } }
            }
            div { class: "mt-md",
                Input {
                    label: Some("Cantidad a ingresar *".to_string()),
                    r#type: "number".to_string(),
                    value: quantity.read().clone(),
                    oninput: move |evt: FormEvent| quantity.set(evt.value().clone()),
                    placeholder: Some("Ej: 10".to_string()),
                }
            }
        }
    }
}
