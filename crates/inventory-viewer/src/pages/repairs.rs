use dioxus::prelude::*;
use inventory_common::dto::CreateRepairRequest;
use inventory_common::{Priority, Repair, RepairStatus};
use rust_decimal::Decimal;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::badge::{Badge, BadgeVariant};
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::modal::Modal;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{AppShell, require_auth};

#[component]
pub fn Repairs() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let repairs = use_signal(Vec::<Repair>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let mut show_modal = use_signal(|| false);
    let mut refresh = use_signal(|| 0);

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
            match client {
                Some(client) => match client.list_repairs(current_page, 10).await {
                    Ok(response) => {
                        repairs_set.set(response.items);
                        total_set.set(response.total);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        error_set.set(Some("Sesión expirada".to_string()));
                    }
                    Err(e) => {
                        error_set.set(Some(e.to_string()));
                    }
                },
                None => {
                    error_set.set(Some("No hay cliente API".to_string()));
                }
            }
            loading_set.set(false);
        });
    };

    use_effect(move || {
        let _ = *refresh.read();
        load_data();
    });

    let columns: Vec<Column<Repair>> = vec![
        Column {
            key: "customer".to_string(),
            header: "Cliente".to_string(),
            render: |r| rsx! { span { "{r.customer_name.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "motorcycle".to_string(),
            header: "Motocicleta".to_string(),
            render: |r| rsx! { span { "{r.motorcycle.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: |r| rsx! {
                Badge {
                    variant: match r.status {
                        RepairStatus::Completed => BadgeVariant::Success,
                        RepairStatus::Cancelled => BadgeVariant::Danger,
                        RepairStatus::InProgress => BadgeVariant::Warning,
                        RepairStatus::Pending => BadgeVariant::Info,
                    },
                    "{r.status}"
                }
            },
        },
        Column {
            key: "priority".to_string(),
            header: "Prioridad".to_string(),
            render: |r| rsx! { span { "{r.priority}" } },
        },
    ];

    let rows = repairs.read().clone();

    rsx! {
        AppShell { title: "Reparaciones".to_string(), active_route: "/repairs".to_string(),
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Listado de reparaciones".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_modal.set(true),
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
                show: *show_modal.read(),
                on_close: move |_| show_modal.set(false),
                on_saved: move |_| {
                    show_modal.set(false);
                    let current = *refresh.read();
                    refresh.set(current + 1);
                },
            }
        }
    }
}

#[component]
fn RepairFormModal(show: bool, on_close: EventHandler<()>, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut customer_name = use_signal(|| "".to_string());
    let mut customer_email = use_signal(|| "".to_string());
    let mut motorcycle = use_signal(|| "".to_string());
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

        let request = CreateRepairRequest {
            customer_name: Some(name_value),
            customer_email: Some(customer_email.read().clone()).filter(|s| !s.is_empty()),
            customer_phone: None,
            motorcycle: Some(motorcycle.read().clone()).filter(|s| !s.is_empty()),
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
            match client {
                Some(client) => match client.create_repair(&request).await {
                    Ok(_) => {
                        on_saved_clone.call(());
                    }
                    Err(e) => {
                        error_set.set(Some(e.to_string()));
                    }
                },
                None => {
                    error_set.set(Some("No hay cliente API".to_string()));
                }
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
                    label: Some("Motocicleta".to_string()),
                    value: motorcycle.read().clone(),
                    oninput: move |evt: FormEvent| motorcycle.set(evt.value().clone()),
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
