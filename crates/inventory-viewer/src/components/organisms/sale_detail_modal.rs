use dioxus::prelude::*;
use inventory_common::dto::SaleDetailResponse;
use inventory_common::money::format_clp;
use rust_decimal::Decimal;

use crate::app_state::use_auth;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::modal::Modal;

#[component]
pub fn SaleDetailModal(
    show: bool,
    sale_id: uuid::Uuid,
    on_close: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let detail = use_signal(|| None::<SaleDetailResponse>);

    use_effect(move || {
        if !show {
            return;
        }
        let client = match auth.api_client() {
            Some(c) => c,
            None => {
                error.set(Some("No hay cliente API".to_string()));
                loading.set(false);
                return;
            }
        };
        let mut loading_set = loading;
        let mut error_set = error;
        let mut detail_set = detail;
        loading_set.set(true);
        error_set.set(None);
        spawn(async move {
            match client.get_sale(sale_id).await {
                Ok(d) => {
                    detail_set.set(Some(d));
                }
                Err(e) => {
                    error_set.set(Some(e.user_message().to_string()));
                }
            }
            loading_set.set(false);
        });
    });

    if !show {
        return rsx! {};
    }

    let loading_val = *loading.read();
    let error_val = error.read().clone();
    let detail_val = detail.read().clone();

    rsx! {
        Modal {
            show: true,
            title: "Detalle de venta".to_string(),
            on_close: on_close,
            if loading_val {
                div { class: "empty-state", Spinner {} }
            } else if let Some(ref err) = error_val {
                div { class: "alert alert-danger", "{err}" }
            } else if let Some(d) = detail_val {
                SaleDetailBody { data: d }
            } else {
                div { class: "empty-state", "No se encontró la venta" }
            }
        }
    }
}

#[component]
fn SaleDetailBody(data: SaleDetailResponse) -> Element {
    let sale = &data.sale;
    let items = &data.items;

    let rows: Vec<Element> = items.iter().map(|item| {
        let name = item.product_name.as_deref().unwrap_or("-").to_string();
        let qty = item.quantity.to_string();
        let uprice = format_clp(item.unit_price);
        let itotal = format_clp(item.total);
        rsx! {
            tr {
                td { "{name}" }
                td { class: "text-right", "{qty}" }
                td { class: "text-right", "{uprice}" }
                td { class: "text-right", "{itotal}" }
            }
        }
    }).collect();

    let has_items = !items.is_empty();
    let subtotal = format_clp(sale.subtotal);
    let discount = format_clp(sale.discount_amount);
    let tax = format_clp(sale.tax_amount);
    let total = format_clp(sale.total);
    let has_discount = sale.discount_amount > Decimal::ZERO;

    let customer = sale.customer_name.as_deref().unwrap_or("Sin nombre").to_string();
    let email = sale.customer_email.as_deref().unwrap_or("-").to_string();
    let phone = sale.customer_phone.as_deref().unwrap_or("-").to_string();
    let payment = crate::i18n::translate_payment(&sale.payment_method).to_string();
    let status = crate::i18n::translate_sale_status(&sale.status);
    let date = sale.created_at.format("%d-%m-%Y %H:%M").to_string();

    rsx! {
        div { class: "sale-detail",
            div { class: "detail-section",
                div { class: "detail-row",
                    span { class: "detail-label", "Cliente" }
                    span { class: "detail-value", "{customer}" }
                }
                div { class: "detail-row",
                    span { class: "detail-label", "Email" }
                    span { class: "detail-value", "{email}" }
                }
                div { class: "detail-row",
                    span { class: "detail-label", "Teléfono" }
                    span { class: "detail-value", "{phone}" }
                }
                div { class: "detail-row",
                    span { class: "detail-label", "Método de pago" }
                    span { class: "detail-value", "{payment}" }
                }
                div { class: "detail-row",
                    span { class: "detail-label", "Estado" }
                    span { class: "detail-value", "{status}" }
                }
                div { class: "detail-row",
                    span { class: "detail-label", "Fecha" }
                    span { class: "detail-value", "{date}" }
                }
            }

            if has_items {
                div { class: "detail-section mt-md",
                    h4 { class: "detail-section-title", "Productos" }
                    table { class: "data-table mt-sm",
                        thead {
                            tr {
                                th { "Producto" }
                                th { class: "text-right", "Cant." }
                                th { class: "text-right", "P. Unitario" }
                                th { class: "text-right", "Total" }
                            }
                        }
                        tbody {
                            {rows.into_iter()}
                        }
                    }
                }
            }

            div { class: "detail-section mt-md",
                div { class: "detail-row",
                    span { class: "detail-label", "Subtotal" }
                    span { class: "detail-value", "{subtotal}" }
                }
                if has_discount {
                    div { class: "detail-row",
                        span { class: "detail-label", "Descuento" }
                        span { class: "detail-value text-danger", "-{discount}" }
                    }
                }
                div { class: "detail-row",
                    span { class: "detail-label", "IVA (19%)" }
                    span { class: "detail-value", "{tax}" }
                }
                div { class: "detail-row detail-total",
                    span { class: "detail-label font-semibold", "Total" }
                    span { class: "detail-value font-semibold", "{total}" }
                }
            }
        }
    }
}
