use dioxus::prelude::*;
use inventory_common::dto::RepairDetail;

#[component]
pub fn InfoTab(detail: RepairDetail) -> Element {
    let rep = &detail.repair;
    rsx! {
        div { class: "detail-section",
            div { class: "detail-row",
                span { class: "detail-label", "Cliente" }
                span { "{rep.customer_name.as_deref().unwrap_or(\"-\")}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Email" }
                span { "{rep.customer_email.as_deref().unwrap_or(\"-\")}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Teléfono" }
                span { "{rep.customer_phone.as_deref().unwrap_or(\"-\")}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Vehículo" }
                span { "{rep.vehicle.as_deref().unwrap_or(\"-\")}" }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Patente" }
                span {
                    if let Some(ref plate) = rep.license_plate {
                        "{plate.to_uppercase()}"
                    } else {
                        "-"
                    }
                }
            }
            div { class: "detail-row",
                span { class: "detail-label", "Estado" }
                span { "{rep.status}" }
            }
            if let Some(ref desc) = rep.description {
                if !desc.is_empty() {
                    div { class: "form-group mt-md",
                        label { class: "form-label", "Descripción" }
                        p { "{desc}" }
                    }
                }
            }
            if let Some(ref diag) = rep.diagnosis {
                if !diag.is_empty() {
                    div { class: "form-group mt-md",
                        label { class: "form-label", "Diagnóstico" }
                        p { "{diag}" }
                    }
                }
            }
        }
    }
}
