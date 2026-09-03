use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::api::{ApiClient, ApiError};
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::routes::Route;

#[component]
pub fn Setup() -> Element {
    let auth = use_auth();
    let mut email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut confirm_password = use_signal(|| "".to_string());
    let mut error = use_signal(|| None::<String>);
    let loading = use_signal(|| false);
    let navigator = use_navigator();

    let on_submit = move |_| {
        error.set(None);

        if *password.read() != *confirm_password.read() {
            error.set(Some("Las contraseñas no coinciden".to_string()));
            return;
        }

        if password.read().len() < 8 {
            error.set(Some("La contraseña debe tener al menos 8 caracteres".to_string()));
            return;
        }

        let email_value = email.read().clone();
        let password_value = password.read().clone();
        let mut error_set = error;
        let mut loading_set = loading;
        let mut auth_set = auth;
        let navigator_set = navigator;

        loading_set.set(true);

        spawn(async move {
            let client = match ApiClient::new(None, true) {
                Ok(c) => c,
                Err(e) => {
                    error_set.set(Some(e.to_string()));
                    loading_set.set(false);
                    return;
                }
            };

            match client.register(&email_value, &password_value).await {
                Ok(response) => {
                    auth_set.login(response.token.clone(), response.user.email.clone());
                    navigator_set.push(Route::Dashboard {});
                }
                Err(ApiError::Forbidden) => {
                    navigator_set.push(Route::Login {});
                }
                Err(e) => {
                    error_set.set(Some(e.to_string()));
                }
            }
            loading_set.set(false);
        });
    };

    rsx! {
        div { class: "login-page",
            div { class: "card login-card",
                div { class: "card-body",
                    h1 { class: "text-2xl font-semibold text-center mb-lg", "Configuración inicial" }
                    p { class: "text-muted text-center mb-lg", "Creá la cuenta de administrador" }

                    if let Some(err) = error.read().as_ref() {
                        div { class: "alert alert-danger mb-md", "{err}" }
                    }

                    Input {
                        label: Some("Email".to_string()),
                        r#type: "email".to_string(),
                        value: email.read().clone(),
                        oninput: move |evt: FormEvent| email.set(evt.value().clone()),
                        placeholder: Some("admin@taller.com".to_string()),
                        required: true,
                    }
                    div { class: "mt-md" }
                    Input {
                        label: Some("Contraseña".to_string()),
                        r#type: "password".to_string(),
                        value: password.read().clone(),
                        oninput: move |evt: FormEvent| password.set(evt.value().clone()),
                        placeholder: Some("••••••••".to_string()),
                        required: true,
                    }
                    div { class: "mt-md" }
                    Input {
                        label: Some("Confirmar contraseña".to_string()),
                        r#type: "password".to_string(),
                        value: confirm_password.read().clone(),
                        oninput: move |evt: FormEvent| confirm_password.set(evt.value().clone()),
                        placeholder: Some("••••••••".to_string()),
                        required: true,
                    }
                    div { class: "mt-lg" }
                    Button {
                        variant: ButtonVariant::Primary,
                        class: Some("w-full".to_string()),
                        loading: *loading.read(),
                        onclick: on_submit,
                        "Crear cuenta admin"
                    }
                    div { class: "mt-md text-center",
                        a {
                            href: "/login",
                            class: "text-muted",
                            "Ya tengo una cuenta"
                        }
                    }
                }
            }
        }
    }
}
