use dioxus::prelude::*;
use workshop_common::dto::{CreateSaleRequest, PosProductResponse, SaleItemRequest};
use workshop_common::money::format_clp;
use workshop_common::PaymentMethod;
use rust_decimal::Decimal;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[derive(Clone, PartialEq)]
struct CartItem {
    product: PosProductResponse,
    quantity: i32,
}

#[component]
pub fn Pos() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let mut cart = use_signal(Vec::<CartItem>::new);
    let mut barcode_input = use_signal(|| "".to_string());
    let mut error = use_signal(|| None::<String>);
    let mut success = use_signal(|| None::<String>);
    let mut customer_name = use_signal(|| "".to_string());
    let mut payment_method = use_signal(|| "cash".to_string());
    let mut saving = use_signal(|| false);

    let subtotal: Decimal = cart
        .read()
        .iter()
        .map(|item| item.product.price * Decimal::from(item.quantity))
        .sum();
    // En Chile el precio ya incluye IVA — el total es el subtotal
    // El IVA se extrae solo para desglose informativo
    let (base, _tax) = workshop_common::money::extract_iva(subtotal);
    let total = subtotal;

    let on_scan_keydown = move |evt: Event<KeyboardData>| {
        if evt.key() == Key::Enter {
            let code = barcode_input.read().clone();
            if code.trim().is_empty() {
                return;
            }
            let client = match auth.api_client() {
                Some(c) => c,
                None => {
                    error.set(Some("No hay cliente API".to_string()));
                    return;
                }
            };
            error.set(None);
            success.set(None);
            spawn(async move {
                match client.lookup_product_by_barcode(code.trim()).await {
                    Ok(product) => {
                        let mut current = cart.read().clone();
                        if let Some(existing) = current
                            .iter_mut()
                            .find(|c| c.product.product_id == product.product_id)
                        {
                            existing.quantity += 1;
                        } else {
                            current.push(CartItem {
                                product,
                                quantity: 1,
                            });
                        }
                        cart.set(current);
                        barcode_input.set(String::new());
                    }
                    Err(ApiError::NotFound(_)) => {
                        error.set(Some("Producto no encontrado".to_string()));
                        barcode_input.set(String::new());
                    }
                    Err(e) => {
                        error.set(Some(e.user_message().to_string()));
                        barcode_input.set(String::new());
                    }
                }
            });
        }
    };

    rsx! {
        AppShell { title: "Punto de venta".to_string(), active_route: Route::Pos {},
            div { class: "pos-container",
                div { class: "pos-scan-section",
                    div { class: "pos-scan-wrapper",
                        input {
                            class: "pos-scan-input",
                            id: "barcode-input",
                            r#type: "text",
                            value: "{barcode_input}",
                            placeholder: "Escanear o escribir código...",
                            autofocus: true,
                            oninput: move |evt: FormEvent| barcode_input.set(evt.value().clone()),
                            onkeydown: on_scan_keydown,
                        }
                    }
                    if let Some(err) = error.read().as_ref() {
                        div { class: "alert alert-danger mt-sm", "{err}" }
                    }
                    if let Some(msg) = success.read().as_ref() {
                        div { class: "alert alert-success mt-sm", "{msg}" }
                    }
                }
                div { class: "pos-cart-section",
                    if cart.read().is_empty() {
                        div { class: "pos-empty", "Escaneá un producto para agregar al carrito" }
                    } else {
                        table { class: "pos-cart-table",
                            thead {
                                tr {
                                    th { "Producto" }
                                    th { "Cant" }
                                    th { class: "pos-cart-price", "Precio"}
                                    th { class: "pos-cart-subtotal", "Subtotal"}
                                    th { }
                                }
                            }
                            tbody {
                                for (index, item) in cart.read().iter().enumerate() {
                                    tr { class: "pos-cart-row",
                                        td { class: "pos-cart-name", "{item.product.name}" }
                                        td {
                                            button {
                                                class: "pos-qty-btn",
                                                onclick: move |_| {
                                                    let mut current = cart.read().clone();
                                                    if let Some(i) = current.get_mut(index) {
                                                        i.quantity = (i.quantity - 1).max(1);
                                                    }
                                                    cart.set(current);
                                                },
                                                "-"
                                            }
                                            span { class: "pos-qty-value", "{item.quantity}" }
                                            button {
                                                class: "pos-qty-btn",
                                                onclick: move |_| {
                                                    let mut current = cart.read().clone();
                                                    if let Some(i) = current.get_mut(index) {
                                                        i.quantity += 1;
                                                    }
                                                    cart.set(current);
                                                },
                                                "+"
                                            }
                                        }
                                        td { class: "pos-cart-price", "{format_clp(item.product.price)}" }
                                        td { class: "pos-cart-subtotal", "{format_clp(item.product.price * Decimal::from(item.quantity))}" }
                                        td {
                                            button {
                                                class: "btn-icon btn-danger",
                                                onclick: move |_| {
                                                    let mut current = cart.read().clone();
                                                    current.remove(index);
                                                    cart.set(current);
                                                },
                                                {IconName::Trash.render()}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "pos-totals",
                            div { class: "pos-total-row",
                                span { "Subtotal (neto)" }
                                span { "{format_clp(base)}" }
                            }
                            div { class: "pos-total-row",
                                span { "IVA (19%)" }
                                span { "{format_clp(_tax)}" }
                            }
                            div { class: "pos-total-row pos-total-final",
                                span { "TOTAL" }
                                span { "{format_clp(total)}" }
                            }
                        }
                    }
                }
                div { class: "pos-footer-section",
                    div { class: "pos-footer-row",
                        Input {
                            label: Some("Cliente".to_string()),
                            value: customer_name.read().clone(),
                            oninput: move |evt: FormEvent| customer_name.set(evt.value().clone()),
                            placeholder: Some("Opcional".to_string()),
                        }
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
                    }
                    div { class: "pos-actions",
                        Button {
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| {
                                cart.set(Vec::new());
                                error.set(None);
                                success.set(None);
                            },
                            "Vaciar carrito"
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            loading: *saving.read(),
                            onclick: move |_| {
                                let items: Vec<CartItem> = cart.read().clone();
                                if items.is_empty() {
                                    error.set(Some("El carrito está vacío".to_string()));
                                    return;
                                }
                                let client = match auth.api_client() {
                                    Some(c) => c,
                                    None => {
                                        error.set(Some("No hay cliente API".to_string()));
                                        return;
                                    }
                                };
                                let payment = match payment_method.read().as_str() {
                                    "card" => PaymentMethod::Card,
                                    "transfer" => PaymentMethod::Transfer,
                                    _ => PaymentMethod::Cash,
                                };
                                let sale_items: Vec<SaleItemRequest> = items
                                    .iter()
                                    .map(|item| SaleItemRequest {
                                        product_id: item.product.product_id,
                                        quantity: item.quantity,
                                        unit_price: item.product.price,
                                        discount: None,
                                    })
                                    .collect();
                                let request = CreateSaleRequest {
                                    customer_name: Some(customer_name.read().clone())
                                        .filter(|s| !s.is_empty()),
                                    customer_email: None,
                                    customer_phone: None,
                                    payment_method: payment,
                                    discount_amount: None,
                                    items: sale_items,
                                };
                                saving.set(true);
                                error.set(None);
                                spawn(async move {
                                    match client.create_sale(&request).await {
                                        Ok(_) => {
                                            cart.set(Vec::new());
                                            customer_name.set(String::new());
                                            payment_method.set("cash".to_string());
                                            success.set(Some("Venta registrada".to_string()));
                                        }
                                        Err(e) => {
                                            error.set(Some(e.user_message().to_string()));
                                        }
                                    }
                                    saving.set(false);
                                });
                            },
                            "Confirmar venta"
                        }
                    }
                }
            }
        }
    }
}
