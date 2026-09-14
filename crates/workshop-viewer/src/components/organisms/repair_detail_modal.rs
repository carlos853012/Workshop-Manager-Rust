use dioxus::prelude::*;
use workshop_common::dto::{
    AddRepairPartRequest, RepairDetail, RepairPartResponse, UpdateRepairRequest,
};
use workshop_common::{Product, RepairStatus, User};
use rust_decimal::Decimal;

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::confirm_modal::ConfirmModal;
use crate::components::molecules::modal::Modal;
use crate::components::organisms::tabs::general_tab::GeneralTab;
use crate::components::organisms::tabs::parts_tab::PartsTab;
use crate::components::organisms::tabs::status_tab::StatusTab;

pub fn status_label(s: &RepairStatus) -> &'static str {
    match s {
        RepairStatus::Pending => "Pendiente",
        RepairStatus::InProgress => "En Progreso",
        RepairStatus::Completed => "Completada",
        RepairStatus::Cancelled => "Cancelada",
        RepairStatus::Deleted => "Eliminada",
    }
}

pub fn can_transition(current: &RepairStatus, target: &RepairStatus) -> bool {
    matches!(
        (current, target),
        (RepairStatus::Pending, RepairStatus::InProgress)
            | (RepairStatus::Pending, RepairStatus::Cancelled)
            | (RepairStatus::InProgress, RepairStatus::Completed)
            | (RepairStatus::InProgress, RepairStatus::Cancelled)
    )
}

#[component]
pub fn RepairDetailModal(
    show: bool,
    repair_id: uuid::Uuid,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut detail = use_signal(|| None::<RepairDetail>);
    let mut parts = use_signal(Vec::<RepairPartResponse>::new);
    let mechanics = use_signal(Vec::<User>::new);
    let mut active_tab = use_signal(|| "general".to_string());

    let mut diagnosis = use_signal(String::new);
    let mut new_part_name = use_signal(String::new);
    let mut new_part_qty = use_signal(|| "1".to_string());
    let mut new_part_cost = use_signal(String::new);
    let mut selected_product_id = use_signal(|| None::<uuid::Uuid>);
    let products = use_signal(Vec::<Product>::new);

    let mut show_status_confirm = use_signal(|| false);
    let pending_status = use_signal(|| None::<RepairStatus>);

    use_effect(move || {
        if !show {
            return;
        }
        let client = auth.api_client();
        let mut loading_set = loading;
        let mut error_set = error;
        let mut detail_set = detail;
        let mut parts_set = parts;
        let mut mechanics_set = mechanics;
        let mut products_set = products;

        loading_set.set(true);
        error_set.set(None);

        spawn(async move {
            if let Some(client) = client {
                match client.get_repair(repair_id).await {
                    Ok(d) => {
                        diagnosis.set(d.repair.diagnosis.clone().unwrap_or_default());
                        detail_set.set(Some(d));
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                        loading_set.set(false);
                        return;
                    }
                }
                match client.list_repair_parts(repair_id).await {
                    Ok(p) => { parts_set.set(p); }
                    Err(e) => { error_set.set(Some(e.user_message().to_string())); }
                }
                match client.list_users(1, 50).await {
                    Ok(u) => {
                        let mechs: Vec<User> = u
                            .items
                            .into_iter()
                            .filter(|u| matches!(u.role, workshop_common::UserRole::Mechanic))
                            .collect();
                        mechanics_set.set(mechs);
                    }
                    Err(e) => { error_set.set(Some(e.user_message().to_string())); }
                }
                match client.list_products(1, 100).await {
                    Ok(prod_page) => { products_set.set(prod_page.items); }
                    Err(e) => { error_set.set(Some(e.user_message().to_string())); }
                }
            }
            loading_set.set(false);
        });
    });

    let save_diagnosis = move |_| {
        let client = auth.api_client();
        let diag = diagnosis.read().clone();
        let on_saved2 = on_saved;
        spawn(async move {
            if let Some(client) = client {
                let req = UpdateRepairRequest {
                    status: None,
                    diagnosis: Some(diag),
                    technician_id: None,
                    estimated_cost: None,
                    final_cost: None,
                    labor_cost: None,
                    estimated_delivery: None,
                };
                match client.update_repair(repair_id, &req).await {
                    Ok(updated) => {
                        detail.set(Some(updated));
                        on_saved2.call(());
                    }
                    Err(e) => { error.set(Some(e.user_message().to_string())); }
                }
            }
        });
    };

    let assign_mechanic = move |evt: Event<FormData>| {
        let value = evt.value();
        let tech_id = if value.is_empty() {
            None
        } else {
            value.parse::<uuid::Uuid>().ok()
        };
        let client = auth.api_client();
        let on_saved2 = on_saved;
        spawn(async move {
            if let Some(client) = client {
                let req = UpdateRepairRequest {
                    status: None,
                    diagnosis: None,
                    technician_id: tech_id,
                    estimated_cost: None,
                    final_cost: None,
                    labor_cost: None,
                    estimated_delivery: None,
                };
                match client.update_repair(repair_id, &req).await {
                    Ok(updated) => {
                        detail.set(Some(updated));
                        on_saved2.call(());
                    }
                    Err(e) => { error.set(Some(e.user_message().to_string())); }
                }
            }
        });
    };

    let add_part = move |_| {
        let name = new_part_name.read().trim().to_string();
        if name.is_empty() {
            return;
        }
        let qty: Decimal = new_part_qty.read().parse().unwrap_or(Decimal::ONE);
        let cost: Option<Decimal> = new_part_cost.read().parse().ok();
        let product_id = *selected_product_id.read();
        let client = auth.api_client();
        spawn(async move {
            if let Some(client) = client {
                let req = AddRepairPartRequest {
                    name,
                    quantity: qty,
                    unit_cost: cost,
                    product_id,
                };
                match client.add_repair_part(repair_id, &req).await {
                    Ok(part) => {
                        let mut current = parts.read().clone();
                        current.push(part);
                        parts.set(current);
                        new_part_name.set(String::new());
                        new_part_qty.set("1".to_string());
                        new_part_cost.set(String::new());
                        selected_product_id.set(None);
                    }
                    Err(e) => { error.set(Some(e.user_message().to_string())); }
                }
            }
        });
    };

    let remove_part = move |part_id: uuid::Uuid| {
        let client = auth.api_client();
        spawn(async move {
            if let Some(client) = client {
                match client.remove_repair_part(repair_id, part_id).await {
                    Ok(()) => {
                        let current = parts.read().clone();
                        let filtered: Vec<RepairPartResponse> =
                            current.into_iter().filter(|x| x.id != part_id).collect();
                        parts.set(filtered);
                    }
                    Err(e) => { error.set(Some(e.user_message().to_string())); }
                }
            }
        });
    };

    if !show {
        return rsx! {};
    }

    let current_tab = active_tab.read().clone();
    let show_footer = !*loading.read() && error.read().is_none();

    rsx! {
        Modal {
            show: true,
            title: "Detalle de reparación".to_string(),
            on_close: move |_| on_close.call(()),
            footer: if show_footer {
                Some(rsx! {
                    Button {
                        class: Some("cancel-button".to_string()),
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| on_close.call(()),
                        "Cancelar"
                    }
                })
            } else {
                None
            },
            if *loading.read() {
                div { class: "empty-state", Spinner {} }
            } else if let Some(ref err) = *error.read() {
                div { class: "alert alert-danger mb-md", "{err}" }
            } else if let Some(ref d) = *detail.read() {
                { let d = d.clone(); rsx! {
                div { class: "tabs",
                    button {
                        class: if current_tab == "general" { "tab-link active" } else { "tab-link" },
                        onclick: move |_| active_tab.set("general".to_string()),
                        "General"
                    }
                    button {
                        class: if current_tab == "status" { "tab-link active" } else { "tab-link" },
                        onclick: move |_| active_tab.set("status".to_string()),
                        "Estado"
                    }
                    button {
                        class: if current_tab == "parts" { "tab-link active" } else { "tab-link" },
                        onclick: move |_| active_tab.set("parts".to_string()),
                        "Insumos"
                    }
                }
                if current_tab == "general" {
                    GeneralTab {
                        detail: d.clone(),
                        diagnosis: diagnosis,
                        save_diagnosis: save_diagnosis,
                    }
                }
                if current_tab == "status" {
                    StatusTab {
                        detail: d.clone(),
                        mechanics: mechanics,
                        assign_mechanic: assign_mechanic,
                        show_status_confirm: show_status_confirm,
                        pending_status: pending_status,
                    }
                }
                if current_tab == "parts" {
                    PartsTab {
                        parts: parts,
                        new_part_name: new_part_name,
                        new_part_qty: new_part_qty,
                        new_part_cost: new_part_cost,
                        selected_product_id: selected_product_id,
                        products: products,
                        add_part: add_part,
                        remove_part: remove_part,
                    }
                }
                }}
            } else {
                div { class: "empty-state", "No se encontró la reparación" }
            }
        }
        ConfirmModal {
            show: *show_status_confirm.read(),
            title: "Confirmar cambio de estado".to_string(),
            message: format!(
                "¿Cambiar estado a \"{}\"?",
                pending_status.read().as_ref().map(|s| status_label(s)).unwrap_or("")
            ),
            confirm_text: Some("Confirmar".to_string()),
            on_confirm: move |_| {
                let new_status = match pending_status.read().as_ref().cloned() {
                    Some(s) => s,
                    None => return,
                };
                show_status_confirm.set(false);
                let client = auth.api_client();
                let on_saved2 = on_saved;
                spawn(async move {
                    if let Some(client) = client {
                        let req = UpdateRepairRequest {
                            status: Some(new_status),
                            diagnosis: None,
                            technician_id: None,
                            estimated_cost: None,
                            final_cost: None,
                            labor_cost: None,
                            estimated_delivery: None,
                        };
                        match client.update_repair(repair_id, &req).await {
                            Ok(updated) => {
                                detail.set(Some(updated));
                                on_saved2.call(());
                            }
                            Err(e) => { error.set(Some(e.user_message().to_string())); }
                        }
                    }
                });
            },
            on_cancel: move |_| show_status_confirm.set(false),
        }
    }
}
