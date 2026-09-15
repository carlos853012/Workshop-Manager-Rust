use dioxus::prelude::*;
use rust_decimal::Decimal;
use workshop_common::Priority;

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::molecules::modal::Modal;

#[component]
pub fn RepairFormModal(
    show: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
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

        let request = workshop_common::dto::CreateRepairRequest {
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
