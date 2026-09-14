use dioxus::prelude::*;
use workshop_common::dto::RepairDetail;
use workshop_common::{RepairStatus, User};

use crate::components::atoms::button::{Button, ButtonVariant};

#[component]
pub fn StatusTab(
    detail: RepairDetail,
    mechanics: Signal<Vec<User>>,
    assign_mechanic: EventHandler<Event<FormData>>,
    show_status_confirm: Signal<bool>,
    pending_status: Signal<Option<RepairStatus>>,
) -> Element {
    let technician_str = detail
        .repair
        .technician_id
        .map(|id| id.to_string())
        .unwrap_or_default();
    let can_start = super::super::repair_detail_modal::can_transition(&detail.repair.status, &RepairStatus::InProgress);
    let can_complete = super::super::repair_detail_modal::can_transition(&detail.repair.status, &RepairStatus::Completed);
    let can_cancel = super::super::repair_detail_modal::can_transition(&detail.repair.status, &RepairStatus::Cancelled);
    let mech_options: Vec<(String, String)> = mechanics
        .read()
        .iter()
        .map(|m| {
            let label = m.display_name.clone().unwrap_or_else(|| m.email.clone());
            (m.id.to_string(), label)
        })
        .collect();
    let timeline: Vec<(String, String, String)> = detail
        .updates
        .iter()
        .map(|u| {
            let status = u
                .status
                .as_ref()
                .map(|s| super::super::repair_detail_modal::status_label(s))
                .unwrap_or("")
                .to_string();
            let desc = u.description.clone().unwrap_or_default();
            let when = u.created_at.format("%Y-%m-%d %H:%M").to_string();
            (status, desc, when)
        })
        .collect();
    let has_updates = !timeline.is_empty();

    rsx! {
        div { class: "detail-section mt-md",
            div { class: "form-group",
                label { class: "form-label", "Asignar mecánico" }
                select {
                    class: "input",
                    value: "{technician_str}",
                    onchange: move |evt| assign_mechanic.call(evt),
                    option { value: "", "-- Sin asignar --" }
                    for (id, label) in mech_options.iter() {
                        option {
                            value: "{id}",
                            "{label}"
                        }
                    }
                }
            }
            div { class: "mt-md",
                label { class: "form-label", "Cambiar estado" }
                div { class: "flex gap-sm mt-sm",
                    if can_start {
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| {
                                pending_status.set(Some(RepairStatus::InProgress));
                                show_status_confirm.set(true);
                            },
                            "Iniciar trabajo"
                        }
                    }
                    if can_complete {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                pending_status.set(Some(RepairStatus::Completed));
                                show_status_confirm.set(true);
                            },
                            "Completar"
                        }
                    }
                    if can_cancel {
                        Button {
                            variant: ButtonVariant::Danger,
                            onclick: move |_| {
                                pending_status.set(Some(RepairStatus::Cancelled));
                                show_status_confirm.set(true);
                            },
                            "Cancelar reparación"
                        }
                    }
                }
            }
            if has_updates {
                div { class: "mt-md",
                    label { class: "form-label", "Historial de cambios" }
                    div { class: "timeline",
                        for (status, desc, when) in timeline.iter() {
                            div { class: "timeline-item",
                                div { class: "timeline-dot" }
                                div { class: "timeline-content",
                                    p { "{status} — {desc}" }
                                    p { class: "text-muted text-xs", "{when}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
