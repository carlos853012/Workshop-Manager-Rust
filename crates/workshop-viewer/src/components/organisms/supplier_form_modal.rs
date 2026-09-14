use dioxus::prelude::*;
use workshop_common::dto::CreateSupplierRequest;

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::molecules::modal::Modal;

#[component]
pub fn SupplierFormModal(
    show: bool,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let mut name = use_signal(|| "".to_string());
    let mut contact_person = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut phone = use_signal(|| "".to_string());
    let mut address = use_signal(|| "".to_string());
    let mut tax_id = use_signal(|| "".to_string());
    let mut payment_terms = use_signal(|| "".to_string());
    let saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let on_submit = move |_| {
        let name_value = name.read().trim().to_string();
        if name_value.is_empty() {
            form_error.set(Some("El nombre es obligatorio".to_string()));
            return;
        }

        let request = CreateSupplierRequest {
            name: name_value,
            contact_person: Some(contact_person.read().clone()).filter(|s| !s.is_empty()),
            email: Some(email.read().clone()).filter(|s| !s.is_empty()),
            phone: Some(phone.read().clone()).filter(|s| !s.is_empty()),
            address: Some(address.read().clone()).filter(|s| !s.is_empty()),
            tax_id: Some(tax_id.read().clone()).filter(|s| !s.is_empty()),
            payment_terms: Some(payment_terms.read().clone()).filter(|s| !s.is_empty()),
        };

        let client = auth.api_client();
        let mut saving_set = saving;
        let mut error_set = form_error;
        let on_saved_clone = on_saved;

        saving_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => match client.create_supplier(&request).await {
                    Ok(_) => {
                        on_saved_clone.call(());
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
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
            title: "Nuevo proveedor".to_string(),
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
                    label: Some("Nombre *".to_string()),
                    value: name.read().clone(),
                    oninput: move |evt: FormEvent| name.set(evt.value().clone()),
                }
                Input {
                    label: Some("Contacto".to_string()),
                    value: contact_person.read().clone(),
                    oninput: move |evt: FormEvent| contact_person.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Email".to_string()),
                    r#type: "email".to_string(),
                    value: email.read().clone(),
                    oninput: move |evt: FormEvent| email.set(evt.value().clone()),
                }
                Input {
                    label: Some("Teléfono".to_string()),
                    value: phone.read().clone(),
                    oninput: move |evt: FormEvent| phone.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Dirección".to_string()),
                    value: address.read().clone(),
                    oninput: move |evt: FormEvent| address.set(evt.value().clone()),
                }
                Input {
                    label: Some("CUIT/RUC".to_string()),
                    value: tax_id.read().clone(),
                    oninput: move |evt: FormEvent| tax_id.set(evt.value().clone()),
                }
            }
            div { class: "mt-md" }
            Input {
                label: Some("Términos de pago".to_string()),
                value: payment_terms.read().clone(),
                oninput: move |evt: FormEvent| payment_terms.set(evt.value().clone()),
            }
        }
    }
}
