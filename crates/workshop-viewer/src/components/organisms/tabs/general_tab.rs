use dioxus::prelude::*;
use workshop_common::dto::RepairDetail;

use crate::components::atoms::button::{Button, ButtonVariant};

#[component]
pub fn GeneralTab(
    detail: RepairDetail,
    diagnosis: Signal<String>,
    save_diagnosis: EventHandler<MouseEvent>,
) -> Element {
    let customer_name = detail
        .repair
        .customer_name
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let customer_email = detail
        .repair
        .customer_email
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let vehicle = detail
        .repair
        .vehicle
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let license_plate = detail
        .repair
        .license_plate
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let priority_str = format!("{}", detail.repair.priority);
    let status_text = super::super::repair_detail_modal::status_label(&detail.repair.status);
    let estimated_cost_str = detail
        .repair
        .estimated_cost
        .map(workshop_common::money::format_clp)
        .unwrap_or_else(|| "-".to_string());
    let final_cost_str = detail
        .repair
        .final_cost
        .map(workshop_common::money::format_clp)
        .unwrap_or_else(|| "-".to_string());
    let delivery_str = detail
        .repair
        .estimated_delivery
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "-".to_string());
    let created_at_str = detail
        .repair
        .created_at
        .format("%Y-%m-%d %H:%M")
        .to_string();
    let description = detail
        .repair
        .description
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let diagnosis_value = diagnosis.read().clone();

    rsx! {
        div { class: "detail-section mt-md",
            div { class: "detail-row",
                span { class: "detail-label", "Cliente" }
                span { "{customer_name}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Email" }
                span { "{customer_email}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Vehículo" }
                span { "{vehicle}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Patente" }
                span { "{license_plate}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Prioridad" }
                span { "{priority_str}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Estado" }
                span { "{status_text}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Costo estimado" }
                span { "{estimated_cost_str}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Costo final" }
                span { "{final_cost_str}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Fecha estimada" }
                span { "{delivery_str}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Creada" }
                span { "{created_at_str}" }
            }
            div { class: "form-group mt-md",
                label { class: "form-label", "Descripción" }
                p { "{description}" }
            }
            div { class: "form-group mt-md",
                label { class: "form-label", "Diagnóstico" }
                textarea {
                    class: "input",
                    value: "{diagnosis_value}",
                    rows: 3,
                    oninput: move |evt| diagnosis.set(evt.value()),
                }
                Button {
                    class: Some("mt-sm".to_string()),
                    variant: ButtonVariant::Primary,
                    onclick: move |evt| save_diagnosis.call(evt),
                    "Guardar diagnóstico"
                }
            }
        }
    }
}
