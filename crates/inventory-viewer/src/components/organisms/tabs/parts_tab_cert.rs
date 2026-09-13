use dioxus::prelude::*;
use inventory_common::dto::{RepairPartResponse, UpdateRepairRequest};
use inventory_common::money::format_clp;
use inventory_common::Product;
use rust_decimal::Decimal;

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::icons::IconName;

#[component]
pub fn PartsTabCert(
    parts: Signal<Vec<RepairPartResponse>>,
    products: Signal<Vec<Product>>,
    selected_product_id: Signal<Option<uuid::Uuid>>,
    new_part_name: Signal<String>,
    new_part_qty: Signal<String>,
    new_part_cost: Signal<String>,
    labor_cost_input: Signal<String>,
    total_parts: Decimal,
    total_repair: Decimal,
    add_part: EventHandler<MouseEvent>,
    remove_part: EventHandler<uuid::Uuid>,
    repair_id: uuid::Uuid,
) -> Element {
    let auth = use_auth();
    rsx! {
        div { class: "detail-section",
            label { class: "form-label", "Repuestos / Insumos" }
            if parts.read().is_empty() {
                p { class: "text-muted", "Sin insumos registrados" }
            } else {
                table { class: "data-table mt-sm",
                    thead {
                        tr {
                            th { "Nombre" }
                            th { "Cant." }
                            th { "Costo Unit." }
                            th { "Total" }
                            th { "" }
                        }
                    }
                    tbody {
                        for part in parts.read().iter() {
                            tr {
                                td { "{part.name}" }
                                td { "{part.quantity.normalize()}" }
                                td {
                                    { part.unit_cost
                                        .map(format_clp)
                                        .unwrap_or_else(|| "-".to_string()) }
                                }
                                td {
                                    { part.total_cost
                                        .map(format_clp)
                                        .unwrap_or_else(|| "-".to_string()) }
                                }
                                td {
                                    button {
                                        class: "btn-icon btn-danger",
                                        title: "Eliminar",
                                        onclick: {
                                            let part_id = part.id;
                                            move |_| remove_part.call(part_id)
                                        },
                                        {IconName::Trash.render()}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "form-row-parts mt-md",
                div { class: "form-group",
                    label { class: "form-label", "Producto (opcional)" }
                    select {
                        class: "input",
                        value: {
                            match selected_product_id.read().as_ref() {
                                Some(id) => id.to_string(),
                                None => String::new(),
                            }
                        },
                        onchange: move |evt: Event<FormData>| {
                            let val = evt.value();
                            if val.is_empty() {
                                selected_product_id.set(None);
                            } else {
                                if let Ok(id) = uuid::Uuid::parse_str(&val) {
                                    selected_product_id.set(Some(id));
                                    if let Some(prod) = products.read().iter().find(|p| p.id == id) {
                                        new_part_name.set(prod.name.clone());
                                        new_part_cost.set(prod.price.trunc().to_string());
                                    }
                                }
                            }
                        },
                        option { value: "", "-- Seleccionar producto --" }
                        for prod in products.read().iter() {
                            option { value: "{prod.id}", "{prod.name}" }
                        }
                    }
                }
                Input {
                    label: Some("Cantidad".to_string()),
                    r#type: "number".to_string(),
                    value: new_part_qty.read().clone(),
                    oninput: move |evt: FormEvent| new_part_qty.set(evt.value().clone()),
                }
                Input {
                    label: Some("Costo unitario".to_string()),
                    r#type: "number".to_string(),
                    value: new_part_cost.read().clone(),
                    oninput: move |evt: FormEvent| new_part_cost.set(evt.value().clone()),
                }
                div { class: "form-group",
                    label { class: "form-label", " " }
                    button {
                        class: "btn-icon btn-primary",
                        title: "Agregar insumo",
                        onclick: add_part,
                        {IconName::Plus.render()}
                    }
                }
            }
            div { class: "cost-summary mt-md",
                label { class: "form-label", "Costo de mano de obra" }
                div { class: "form-row",
                    div { class: "form-group",
                        input {
                            class: "input",
                            r#type: "text",
                            placeholder: "$0",
                            value: labor_cost_input.read().clone(),
                            oninput: move |evt: Event<FormData>| labor_cost_input.set(evt.value().clone()),
                        }
                    }
                    div { class: "form-group",
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: {
                                let repair_id = repair_id;
                                move |_| {
                                    let parsed: Option<Decimal> =
                                        labor_cost_input.read().replace(".", "").replace(",", "").replace("$", "").replace(" ", "").parse().ok();
                                    let client = auth.api_client();
                                    let req = UpdateRepairRequest {
                                        status: None,
                                        diagnosis: None,
                                        technician_id: None,
                                        estimated_cost: None,
                                        final_cost: None,
                                        labor_cost: parsed,
                                        estimated_delivery: None,
                                    };
                                    spawn(async move {
                                        if let Some(client) = client {
                                            let _ = client.update_repair(repair_id, &req).await;
                                        }
                                    });
                                }
                            },
                            "Guardar"
                        }
                    }
                }
                div { class: "cost-totals mt-sm",
                    div { class: "cost-row",
                        span { "Mano de obra" }
                        span { "{labor_cost_input.read()}" }
                    }
                    div { class: "cost-row",
                        span { "Repuestos" }
                        span { "{format_clp(total_parts)}" }
                    }
                    div { class: "cost-row cost-total",
                        span { strong { "Total" } }
                        span { strong { "{format_clp(total_repair)}" } }
                    }
                }
            }
        }
    }
}
