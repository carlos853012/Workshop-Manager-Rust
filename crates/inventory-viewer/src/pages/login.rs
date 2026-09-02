use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::routes::Route;

#[component]
pub fn Login() -> Element {
    let mut auth = use_auth();
    let mut email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let navigator = use_navigator();

    let on_submit = move |_| {
        error.set(None);
        loading.set(true);

        let email_value = email.read().clone();
        let password_value = password.read().clone();

        spawn(async move {
            let client = match crate::api::ApiClient::new(None, true) {
                Ok(c) => c,
                Err(e) => {
                    error.set(Some(e.to_string()));
                    loading.set(false);
                    return;
                }
            };

            match client.login(&email_value, &password_value).await {
                Ok(response) => {
                    auth.login(response.token.clone(), response.user.email.clone());
                    navigator.push(Route::Dashboard {});
                }
                Err(ApiError::Unauthorized) => {
                    error.set(Some("Credenciales inválidas".to_string()));
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
            loading.set(false);
        });
    };

    rsx! {
        div { class: "login-page",
            div { class: "card login-card",
                div { class: "card-body",
                    h1 { class: "text-2xl font-semibold text-center mb-lg", "WorkshopManager" }
                    p { class: "text-muted text-center mb-lg", "Iniciá sesión para continuar" }

                    if let Some(err) = error.read().as_ref() {
                        div { class: "alert alert-danger mb-md", "{err}" }
                    }

                    Input {
                        label: Some("Email".to_string()),
                        r#type: "email".to_string(),
                        value: email.read().clone(),
                        oninput: move |evt: FormEvent| email.set(evt.value().clone()),
                        placeholder: Some("usuario@taller.com".to_string()),
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
                    div { class: "mt-lg" }
                    Button {
                        variant: ButtonVariant::Primary,
                        class: Some("w-full".to_string()),
                        loading: *loading.read(),
                        onclick: on_submit,
                        "Ingresar"
                    }
                }
            }
        }
    }
}
