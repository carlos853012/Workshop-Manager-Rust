use dioxus::prelude::*;
use inventory_common::dto::RepairPartResponse;
use inventory_common::Product;

use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::icons::IconName;

#[component]
pub fn PartsTab(
    parts: Signal<Vec<RepairPartResponse>>,
    new_part_name: Signal<String>,
    new_part_qty: Signal<String>,
    new_part_cost: Signal<String>,
    selected_product_id: Signal<Option<uuid::Uuid>>,
    products: Signal<Vec<Product>>,
    add_part: EventHandler<()>,
    remove_part: EventHandler<uuid::Uuid>,
) -> Element {
    let parts_data: Vec<(uuid::Uuid, String, String, String, String)> = parts
        .read()
        .iter()
        .map(|p| {
            let unit = p
                .unit_cost
                .map(inventory_common::money::format_clp)
                .unwrap_or_else(|| "-".to_string());
            let total = p
                .total_cost
                .map(inventory_common::money::format_clp)
                .unwrap_or_else(|| "-".to_string());
            (p.id, p.name.clone(), p.quantity.normalize().to_string(), unit, total)
        })
        .collect();
    let is_empty = parts_data.is_empty();

    rsx! {
        div { class: "detail-section mt-md",
            label { class: "form-label", "Insumos / Piezas" }
            if is_empty {
                p { class: "text-muted", "Sin insumos registrados" }
            } else {
                table { class: "data-table mt-md",
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
                        for (pid, name, qty, unit, total) in parts_data.iter() {
                            tr {
                                td { "{name}" }
                                td { "{qty}" }
                                td { "{unit}" }
                                td { "{total}" }
                                td {
                                    button {
                                        class: "btn-icon btn-danger",
                                        title: "Eliminar",
                                        onclick: {
                                            let part_id = *pid;
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
            div { class: "form-row mt-md",
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
                    label: Some("Nombre *".to_string()),
                    value: new_part_name.read().clone(),
                    oninput: move |evt: FormEvent| new_part_name.set(evt.value().clone()),
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
            }
            Button {
                class: Some("mt-sm".to_string()),
                variant: ButtonVariant::Primary,
                onclick: move |_| add_part.call(()),
                "Agregar insumo"
            }
        }
    }
}
