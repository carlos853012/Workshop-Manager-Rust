use dioxus::prelude::*;
use inventory_common::dto::{
    AddRepairPartRequest, RepairDetail, RepairPartResponse, UpdateRepairRequest,
};
use inventory_common::{Priority, RepairStatus, User};
use rust_decimal::Decimal;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::confirm_modal::ConfirmModal;
use crate::components::molecules::modal::Modal;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;
use std::rc::Rc;

#[component]
pub fn Repairs() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let repairs = use_signal(Vec::<inventory_common::Repair>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let mut show_create_modal = use_signal(|| false);
    let mut refresh = use_signal(|| 0);
    let mut show_detail_modal = use_signal(|| false);
    let mut detail_repair_id = use_signal(|| None::<uuid::Uuid>);

    let load_data = move || {
        let client = auth.api_client();
        let mut repairs_set = repairs;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        spawn(async move {
            if let Some(client) = client {
                match client.list_repairs(current_page, 10).await {
                    Ok(response) => {
                        repairs_set.set(response.items);
                        total_set.set(response.total);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        error_set.set(Some("Sesión expirada".to_string()));
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
            } else {
                error_set.set(Some("No hay cliente API".to_string()));
            }
            loading_set.set(false);
        });
    };

    use_effect(move || {
        let _ = *refresh.read();
        load_data();
    });

    let columns: Vec<Column<inventory_common::Repair>> = vec![
        Column {
            key: "customer".to_string(),
            header: "Cliente".to_string(),
            render: Rc::new(|r: &inventory_common::Repair| {
                rsx! { span { "{r.customer_name.as_deref().unwrap_or(\"-\")}" } }
            }),
        },
        Column {
            key: "vehicle".to_string(),
            header: "Vehículo".to_string(),
            render: Rc::new(|r: &inventory_common::Repair| {
                rsx! { span { "{r.vehicle.as_deref().unwrap_or(\"-\")}" } }
            }),
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: Rc::new(|r: &inventory_common::Repair| {
                rsx! {
                    span { "{r.status}" }
                }
            }),
        },
        Column {
            key: "priority".to_string(),
            header: "Prioridad".to_string(),
            render: Rc::new(|r: &inventory_common::Repair| rsx! { span { "{r.priority}" } }),
        },
        Column {
            key: "actions".to_string(),
            header: "".to_string(),
            render: Rc::new({
                let mut detail_id = detail_repair_id;
                let mut show_detail = show_detail_modal;
                move |r: &inventory_common::Repair| {
                    let rid = r.id;
                    rsx! {
                        div { class: "table-actions",
                            button {
                                class: "btn-icon",
                                title: "Ver detalle",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    detail_id.set(Some(rid));
                                    show_detail.set(true);
                                },
                                {IconName::DocumentText.render()}
                            }
                        }
                    }
                }
            }),
        },
    ];

    let rows = repairs.read().clone();

    rsx! {
        AppShell { title: "Reparaciones".to_string(), active_route: Route::Repairs {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Listado de reparaciones".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_create_modal.set(true),
                        "Nueva reparación"
                    }
                },
                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
                } else {
                    DataTable {
                        columns: columns.clone(),
                        rows: rows.clone(),
                        page: *page.read(),
                        total: *total.read(),
                        on_page_change: move |new_page: i32| page.set(new_page),
                    }
                }
            }
            RepairFormModal {
                show: *show_create_modal.read(),
                on_close: move |_| show_create_modal.set(false),
                on_saved: move |_| {
                    show_create_modal.set(false);
                    let current = *refresh.read();
                    refresh.set(current + 1);
                },
            }
            if let Some(rid) = *detail_repair_id.read() {
                RepairDetailModal {
                    key: "{rid}",
                    show: *show_detail_modal.read(),
                    repair_id: rid,
                    on_close: move |_| {
                        show_detail_modal.set(false);
                        detail_repair_id.set(None);
                    },
                    on_saved: move |_| {
                        let current = *refresh.read();
                        refresh.set(current + 1);
                    },
                }
            }
        }
    }
}

#[component]
fn RepairFormModal(show: bool, on_close: EventHandler<()>, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut customer_name = use_signal(|| "".to_string());
    let mut customer_email = use_signal(|| "".to_string());
    let mut vehicle = use_signal(|| "".to_string());
    let mut license_plate = use_signal(|| "".to_string());
    let mut description = use_signal(|| "".to_string());
    let mut priority = use_signal(|| "medium".to_string());
    let mut estimated_cost = use_signal(|| "".to_string());
    let mut estimated_delivery = use_signal(|| "".to_string());
    let saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let on_submit = move |_| {
        let name_value = customer_name.read().trim().to_string();
        if name_value.is_empty() {
            form_error.set(Some("El nombre del cliente es obligatorio".to_string()));
            return;
        }

        let priority_enum = match priority.read().as_str() {
            "high" => Priority::High,
            "low" => Priority::Low,
            _ => Priority::Medium,
        };

        let delivery_date = if estimated_delivery.read().is_empty() {
            None
        } else {
            match chrono::NaiveDate::parse_from_str(&estimated_delivery.read(), "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => {
                    form_error.set(Some("Fecha inválida (YYYY-MM-DD)".to_string()));
                    return;
                }
            }
        };

        let cost = estimated_cost.read().parse::<Decimal>().ok();

        let request = inventory_common::dto::CreateRepairRequest {
            customer_name: Some(name_value),
            customer_email: Some(customer_email.read().clone()).filter(|s| !s.is_empty()),
            customer_phone: None,
            vehicle: Some(vehicle.read().clone()).filter(|s| !s.is_empty()),
            license_plate: Some(license_plate.read().clone()).filter(|s| !s.is_empty()),
            description: Some(description.read().clone()).filter(|s| !s.is_empty()),
            priority: priority_enum,
            estimated_cost: cost,
            estimated_delivery: delivery_date,
        };

        let client = auth.api_client();
        let mut saving_set = saving;
        let mut error_set = form_error;
        let on_saved_clone = on_saved;

        saving_set.set(true);
        error_set.set(None);

        spawn(async move {
            if let Some(client) = client {
                match client.create_repair(&request).await {
                    Ok(_) => {
                        on_saved_clone.call(());
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
            } else {
                error_set.set(Some("No hay cliente API".to_string()));
            }
            saving_set.set(false);
        });
    };

    rsx! {
        Modal {
            show: show,
            title: "Nueva reparación".to_string(),
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
                    label: Some("Cliente *".to_string()),
                    value: customer_name.read().clone(),
                    oninput: move |evt: FormEvent| customer_name.set(evt.value().clone()),
                }
                Input {
                    label: Some("Email".to_string()),
                    r#type: "email".to_string(),
                    value: customer_email.read().clone(),
                    oninput: move |evt: FormEvent| customer_email.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Vehículo".to_string()),
                    value: vehicle.read().clone(),
                    oninput: move |evt: FormEvent| vehicle.set(evt.value().clone()),
                }
                Input {
                    label: Some("Patente".to_string()),
                    value: license_plate.read().clone(),
                    oninput: move |evt: FormEvent| license_plate.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                div { class: "form-group",
                    label { class: "form-label", "Prioridad" }
                    select {
                        class: "input",
                        value: "{priority.read()}",
                        onchange: move |evt: Event<FormData>| priority.set(evt.value().clone()),
                        option { value: "low", "Baja" }
                        option { value: "medium", "Media" }
                        option { value: "high", "Alta" }
                    }
                }
                Input {
                    label: Some("Costo estimado".to_string()),
                    r#type: "number".to_string(),
                    value: estimated_cost.read().clone(),
                    oninput: move |evt: FormEvent| estimated_cost.set(evt.value().clone()),
                }
            }
            div { class: "mt-md" }
            Input {
                label: Some("Fecha estimada de entrega (YYYY-MM-DD)".to_string()),
                value: estimated_delivery.read().clone(),
                oninput: move |evt: FormEvent| estimated_delivery.set(evt.value().clone()),
            }
            div { class: "mt-md" }
            div { class: "form-group",
                label { class: "form-label", "Descripción" }
                textarea {
                    class: "input",
                    value: "{description.read()}",
                    rows: "3",
                    oninput: move |evt: FormEvent| description.set(evt.value().clone()),
                }
            }
        }
    }
}

fn status_label(s: &RepairStatus) -> &'static str {
    match s {
        RepairStatus::Pending => "Pendiente",
        RepairStatus::InProgress => "En Progreso",
        RepairStatus::Completed => "Completada",
        RepairStatus::Cancelled => "Cancelada",
        RepairStatus::Deleted => "Eliminada",
    }
}

fn can_transition(current: &RepairStatus, target: &RepairStatus) -> bool {
    matches!(
        (current, target),
        (RepairStatus::Pending, RepairStatus::InProgress)
            | (RepairStatus::Pending, RepairStatus::Cancelled)
            | (RepairStatus::InProgress, RepairStatus::Completed)
            | (RepairStatus::InProgress, RepairStatus::Cancelled)
    )
}

#[component]
fn RepairDetailModal(
    show: bool,
    repair_id: uuid::Uuid,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);
    let mut detail = use_signal(|| None::<RepairDetail>);
    let mut parts = use_signal(Vec::<RepairPartResponse>::new);
    let mechanics = use_signal(Vec::<User>::new);
    let mut active_tab = use_signal(|| "general".to_string());

    let mut diagnosis = use_signal(String::new);
    let mut new_part_name = use_signal(String::new);
    let mut new_part_qty = use_signal(|| "1".to_string());
    let mut new_part_cost = use_signal(String::new);

    let mut show_status_confirm = use_signal(|| false);
    let pending_status = use_signal(|| None::<RepairStatus>);

    use_effect(move || {
        let client = auth.api_client();
        let mut loading_set = loading;
        let mut error_set = error;
        let mut detail_set = detail;
        let mut parts_set = parts;
        let mut mechanics_set = mechanics;

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
                if let Ok(p) = client.list_repair_parts(repair_id).await {
                    parts_set.set(p);
                }
                if let Ok(u) = client.list_users(1, 50).await {
                    let mechs: Vec<User> = u
                        .items
                        .into_iter()
                        .filter(|u| format!("{}", u.role) == "mechanic")
                        .collect();
                    mechanics_set.set(mechs);
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
                    estimated_delivery: None,
                };
                if let Ok(updated) = client.update_repair(repair_id, &req).await {
                    detail.set(Some(updated));
                    on_saved2.call(());
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
                    estimated_delivery: None,
                };
                if let Ok(updated) = client.update_repair(repair_id, &req).await {
                    detail.set(Some(updated));
                    on_saved2.call(());
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
        let client = auth.api_client();
        spawn(async move {
            if let Some(client) = client {
                let req = AddRepairPartRequest {
                    name,
                    quantity: qty,
                    unit_cost: cost,
                };
                if let Ok(part) = client.add_repair_part(repair_id, &req).await {
                    let mut current = parts.read().clone();
                    current.push(part);
                    parts.set(current);
                    new_part_name.set(String::new());
                    new_part_qty.set("1".to_string());
                    new_part_cost.set(String::new());
                }
            }
        });
    };

    let remove_part = move |part_id: uuid::Uuid| {
        let client = auth.api_client();
        spawn(async move {
            if let Some(client) = client {
                if client.remove_repair_part(repair_id, part_id).await.is_ok() {
                    let current = parts.read().clone();
                    let filtered: Vec<RepairPartResponse> =
                        current.into_iter().filter(|x| x.id != part_id).collect();
                    parts.set(filtered);
                }
            }
        });
    };

    if !show {
        return rsx! {};
    }

    if *loading.read() {
        return rsx! {
            Modal {
                show: true,
                title: "Detalle de reparación".to_string(),
                on_close: move |_| on_close.call(()),
                footer: rsx! {},
                div { class: "empty-state", Spinner {} }
            }
        };
    }

    if let Some(ref err) = *error.read() {
        return rsx! {
            Modal {
                show: true,
                title: "Detalle de reparación".to_string(),
                on_close: move |_| on_close.call(()),
                footer: rsx! {},
                div { class: "alert alert-danger", "{err}" }
            }
        };
    }

    let d = match detail.read().as_ref() {
        Some(d) => d.clone(),
        None => return rsx! {},
    };

    let current_tab = active_tab.read().clone();

    rsx! {
        Modal {
            show: true,
            title: "Detalle de reparación".to_string(),
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                Button {
                    class: Some("cancel-button".to_string()),
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_close.call(()),
                    "Cancelar"
                }
            },
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
                    add_part: add_part,
                    remove_part: remove_part,
                }
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
                            estimated_delivery: None,
                        };
                        if let Ok(updated) = client.update_repair(repair_id, &req).await {
                            detail.set(Some(updated));
                            on_saved2.call(());
                        }
                    }
                });
            },
            on_cancel: move |_| show_status_confirm.set(false),
        }
    }
}

#[component]
fn GeneralTab(
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
    let status_text = status_label(&detail.repair.status);
    let estimated_cost_str = detail
        .repair
        .estimated_cost
        .map(inventory_common::money::format_clp)
        .unwrap_or_else(|| "-".to_string());
    let final_cost_str = detail
        .repair
        .final_cost
        .map(inventory_common::money::format_clp)
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

#[component]
fn StatusTab(
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
    let can_start = can_transition(&detail.repair.status, &RepairStatus::InProgress);
    let can_complete = can_transition(&detail.repair.status, &RepairStatus::Completed);
    let can_cancel = can_transition(&detail.repair.status, &RepairStatus::Cancelled);
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
                .map(status_label)
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

#[component]
fn PartsTab(
    parts: Signal<Vec<RepairPartResponse>>,
    new_part_name: Signal<String>,
    new_part_qty: Signal<String>,
    new_part_cost: Signal<String>,
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
            (p.id, p.name.clone(), p.quantity.to_string(), unit, total)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_label_covers_all_variants() {
        assert_eq!(status_label(&RepairStatus::Pending), "Pendiente");
        assert_eq!(status_label(&RepairStatus::InProgress), "En Progreso");
        assert_eq!(status_label(&RepairStatus::Completed), "Completada");
        assert_eq!(status_label(&RepairStatus::Cancelled), "Cancelada");
        assert_eq!(status_label(&RepairStatus::Deleted), "Eliminada");
    }

    #[test]
    fn can_transition_only_allows_valid_paths() {
        assert!(can_transition(
            &RepairStatus::Pending,
            &RepairStatus::InProgress
        ));
        assert!(can_transition(
            &RepairStatus::Pending,
            &RepairStatus::Cancelled
        ));
        assert!(can_transition(
            &RepairStatus::InProgress,
            &RepairStatus::Completed
        ));
        assert!(can_transition(
            &RepairStatus::InProgress,
            &RepairStatus::Cancelled
        ));
        assert!(!can_transition(
            &RepairStatus::Pending,
            &RepairStatus::Completed
        ));
        assert!(!can_transition(
            &RepairStatus::Completed,
            &RepairStatus::Cancelled
        ));
        assert!(!can_transition(
            &RepairStatus::Cancelled,
            &RepairStatus::Pending
        ));
    }
}
