use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::api::{ApiClient, ApiError};
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::organisms::server_settings::ConnectionSettingsButton;
use crate::routes::Route;

#[component]
pub fn Setup() -> Element {
    let auth = use_auth();
    let mut workshop_name = use_signal(|| "".to_string());
    let mut workshop_address = use_signal(|| "".to_string());
    let mut workshop_city = use_signal(|| "".to_string());
    let mut admin_name = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut confirm_password = use_signal(|| "".to_string());
    let mut error = use_signal(|| None::<String>);
    let loading = use_signal(|| false);
    let navigator = use_navigator();

    let on_submit = move |_| {
        error.set(None);

        if workshop_name.read().trim().is_empty()
            || workshop_address.read().trim().is_empty()
            || workshop_city.read().trim().is_empty()
            || admin_name.read().trim().is_empty()
        {
            error.set(Some(
                "Completa todos los datos del taller y del administrador".to_string(),
            ));
            return;
        }

        if *password.read() != *confirm_password.read() {
            error.set(Some("Las contraseñas no coinciden".to_string()));
            return;
        }

        if password.read().len() < 8 {
            error.set(Some(
                "La contraseña debe tener al menos 8 caracteres".to_string(),
            ));
            return;
        }

        let email_value = email.read().clone();
        let password_value = password.read().clone();
        let workshop_name_value = workshop_name.read().trim().to_string();
        let workshop_address_value = workshop_address.read().trim().to_string();
        let workshop_city_value = workshop_city.read().trim().to_string();
        let admin_name_value = admin_name.read().trim().to_string();
        let mut error_set = error;
        let mut loading_set = loading;
        let mut auth_set = auth;
        let navigator_set = navigator;

        loading_set.set(true);

        spawn(async move {
            let cfg = crate::config::config();
            let client = match ApiClient::new(
                None,
                cfg.server.api_key.clone(),
                cfg.server.tls_accept_invalid_certs,
            ) {
                Ok(c) => c,
                Err(e) => {
                    error_set.set(Some(e.user_message().to_string()));
                    loading_set.set(false);
                    return;
                }
            };

            match client
                .register(
                    &workshop_name_value,
                    &workshop_address_value,
                    &workshop_city_value,
                    &admin_name_value,
                    &email_value,
                    &password_value,
                )
                .await
            {
                Ok(response) => {
                    auth_set.login(
                        response.token.clone(),
                        response.user.email.clone(),
                        response.user.display_name.clone(),
                        response.user.role.clone(),
                        response.workshop.clone(),
                    );
                    navigator_set.push(Route::Dashboard {});
                }
                Err(ApiError::Forbidden) => {
                    error_set.set(Some(
                        "Ya existe un usuario administrador. Usá Login para ingresar.".to_string(),
                    ));
                }
                Err(e) => {
                    error_set.set(Some(e.user_message().to_string()));
                }
            }
            loading_set.set(false);
        });
    };

    rsx! {
        div { class: "login-page",
            div { class: "card login-card",
                div { class: "card-body",
                    div { class: "login-header",
                        h1 { class: "text-2xl font-semibold", "Configuración inicial" }
                        ConnectionSettingsButton { class: Some("login-settings-button".to_string()) }
                    }
                    p { class: "text-muted text-center mb-lg", "Creá la cuenta de administrador" }

                    if let Some(err) = error.read().as_ref() {
                        div { class: "alert alert-danger mb-md", "{err}" }
                    }

                    Input {
                        label: Some("Nombre del taller".to_string()),
                        value: workshop_name.read().clone(),
                        oninput: move |evt: FormEvent| workshop_name.set(evt.value().clone()),
                        placeholder: Some("Taller Moto Racing".to_string()),
                        required: true,
                    }
                    div { class: "mt-md" }
                    Input {
                        label: Some("Dirección".to_string()),
                        value: workshop_address.read().clone(),
                        oninput: move |evt: FormEvent| workshop_address.set(evt.value().clone()),
                        placeholder: Some("Av. Principal 123".to_string()),
                        required: true,
                    }
                    div { class: "mt-md" }
                    Input {
                        label: Some("Ciudad".to_string()),
                        value: workshop_city.read().clone(),
                        oninput: move |evt: FormEvent| workshop_city.set(evt.value().clone()),
                        placeholder: Some("Madrid".to_string()),
                        required: true,
                    }
                    div { class: "mt-md" }
                    Input {
                        label: Some("Nombre del administrador".to_string()),
                        value: admin_name.read().clone(),
                        oninput: move |evt: FormEvent| admin_name.set(evt.value().clone()),
                        placeholder: Some("Carlos García".to_string()),
                        required: true,
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
                }
            }
        }
    }
}
