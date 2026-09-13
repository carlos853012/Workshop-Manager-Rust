use dioxus::prelude::*;
use inventory_common::dto::{CreateUserRequest, UpdateUserRequest};
use inventory_common::User;

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::molecules::modal::Modal;

#[component]
pub fn UserFormModal(
    show: bool,
    edit_user: Option<User>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let is_edit = edit_user.is_some();

    let mut email = use_signal(|| {
        edit_user
            .as_ref()
            .map(|u| u.email.clone())
            .unwrap_or_default()
    });
    let mut name = use_signal(|| {
        edit_user
            .as_ref()
            .and_then(|u| u.display_name.clone())
            .unwrap_or_default()
    });
    let mut password = use_signal(String::new);
    let mut role = use_signal(|| {
        edit_user
            .as_ref()
            .map(|u| format!("{}", u.role))
            .unwrap_or_else(|| "seller".to_string())
    });
    let mut saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let on_submit = move |_| {
        let email_val = email.read().trim().to_string();
        let name_val = name.read().trim().to_string();
        let password_val = password.read().clone();

        if email_val.is_empty() {
            form_error.set(Some("El email es obligatorio".to_string()));
            return;
        }
        if !email_val.contains('@') || !email_val.contains('.') {
            form_error.set(Some("Email inválido".to_string()));
            return;
        }
        if name_val.is_empty() {
            form_error.set(Some("El nombre es obligatorio".to_string()));
            return;
        }
        if !is_edit && password_val.len() < 6 {
            form_error.set(Some("La contraseña debe tener al menos 6 caracteres".to_string()));
            return;
        }

        let client = match auth.api_client() {
            Some(c) => c,
            None => {
                form_error.set(Some("No hay cliente API".to_string()));
                return;
            }
        };

        let role_val = role.read().clone();
        saving.set(true);
        form_error.set(None);

        if is_edit {
            let user = edit_user.clone().unwrap();
            let req = UpdateUserRequest {
                display_name: if name_val.is_empty() { None } else { Some(name_val) },
                role: Some(role_val),
                status: None,
            };
            spawn(async move {
                match client.update_user(user.id, &req).await {
                    Ok(_) => on_saved.call(()),
                    Err(e) => form_error.set(Some(e.user_message().to_string())),
                }
                saving.set(false);
            });
        } else {
            let req = CreateUserRequest {
                email: email_val,
                display_name: if name_val.is_empty() { None } else { Some(name_val) },
                password: password_val,
                role: role_val,
            };
            spawn(async move {
                match client.create_user(&req).await {
                    Ok(_) => on_saved.call(()),
                    Err(e) => form_error.set(Some(e.user_message().to_string())),
                }
                saving.set(false);
            });
        }
    };

    let title = if is_edit { "Editar usuario" } else { "Nuevo usuario" };

    rsx! {
        Modal {
            show: show,
            title: title.to_string(),
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
                    if is_edit { "Guardar" } else { "Crear" }
                }
            },
            if let Some(err) = form_error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            div { class: "form-grid",
                if !is_edit {
                    Input {
                        label: Some("Email".to_string()),
                        value: email.read().clone(),
                        oninput: move |evt: FormEvent| email.set(evt.value()),
                    }
                }
                Input {
                    label: Some("Nombre".to_string()),
                    value: name.read().clone(),
                    oninput: move |evt: FormEvent| name.set(evt.value()),
                }
                if !is_edit {
                    Input {
                        label: Some("Contraseña".to_string()),
                        r#type: "password".to_string(),
                        value: password.read().clone(),
                        oninput: move |evt: FormEvent| password.set(evt.value()),
                    }
                }
                div { class: "form-group",
                    label { class: "form-label", "Rol" }
                    select {
                        class: "input",
                        value: role.read().clone(),
                        onchange: move |evt: FormEvent| role.set(evt.value()),
                        option { value: "admin", "Admin" }
                        option { value: "mechanic", "Mecánico" }
                        option { value: "seller", "Vendedor" }
                    }
                }
            }
        }
    }
}
