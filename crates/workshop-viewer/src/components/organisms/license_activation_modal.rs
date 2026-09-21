use dioxus::prelude::*;

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::molecules::modal::Modal;

#[component]
pub fn LicenseActivationModal(show: bool, on_close: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut license_key = use_signal(|| "".to_string());
    let mut error = use_signal(|| None::<String>);
    let success = use_signal(|| false);
    let loading = use_signal(|| false);

    let on_activate = move |_| {
        error.set(None);
        let key = license_key.read().trim().to_string();
        if key.is_empty() {
            error.set(Some("Ingresá tu clave de licencia".to_string()));
            return;
        }

        let mut loading_set = loading;
        let mut error_set = error;
        let mut success_set = success;
        let mut auth_set = auth;
        let on_close_clone = on_close;
        loading_set.set(true);

        spawn(async move {
            let client = match auth.api_client() {
                Some(c) => c,
                None => {
                    error_set.set(Some("No hay sesión activa".into()));
                    loading_set.set(false);
                    return;
                }
            };

            match client.activate_license(&key).await {
                Ok(info) => {
                    auth_set.license_info.set(Some(info));
                    success_set.set(true);
                    loading_set.set(false);
                    on_close_clone.call(());
                }
                Err(crate::api::ApiError::Forbidden) => {
                    error_set.set(Some("Clave de licencia inválida o revocada".to_string()));
                    loading_set.set(false);
                }
                Err(crate::api::ApiError::Network(e)) => {
                    error_set.set(Some(format!("Sin conexión al servidor: {}", e)));
                    loading_set.set(false);
                }
                Err(e) => {
                    error_set.set(Some(e.user_message().to_string()));
                    loading_set.set(false);
                }
            }
        });
    };

    rsx! {
        Modal {
            show: show,
            title: "Activar Licencia".to_string(),
            on_close: move |_| on_close.call(()),
            footer: None::<Element>,

            if *success.read() {
                div { class: "alert alert-success",
                    "Licencia activada correctamente"
                }
            } else {
                if let Some(err) = error.read().as_ref() {
                    div { class: "alert alert-danger mb-md", "{err}" }
                }

                p { class: "text-muted mb-md",
                    "Ingresá la clave de licencia que recibiste de tu proveedor."
                }

                Input {
                    label: Some("Clave de licencia".to_string()),
                    value: license_key.read().clone(),
                    oninput: move |evt: FormEvent| license_key.set(evt.value().clone()),
                    placeholder: Some("XXXX-XXXX-XXXX-XXXX".to_string()),
                    required: true,
                }

                div { class: "mt-lg",
                    Button {
                        variant: ButtonVariant::Primary,
                        class: Some("w-full".to_string()),
                        loading: *loading.read(),
                        onclick: on_activate,
                        "Activar"
                    }
                }
            }
        }
    }
}
