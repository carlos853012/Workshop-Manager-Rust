use dioxus::prelude::*;
use workshop_common::User;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::confirm_modal::ConfirmModal;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::components::organisms::user_form_modal::UserFormModal;
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;
use std::rc::Rc;

#[component]
pub fn Users() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let users = use_signal(Vec::<User>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut refresh = use_signal(|| 0);

    let mut show_form_modal = use_signal(|| false);
    let mut editing_user = use_signal(|| None::<User>);

    let mut show_delete_modal = use_signal(|| false);
    let deleting_user = use_signal(|| None::<User>);

    let load_data = move || {
        let client = auth.api_client();
        let mut users_set = users;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        let mut auth = auth;
        spawn(async move {
            if let Some(client) = client {
                match client.list_users(current_page, 10).await {
                    Ok(response) => {
                        users_set.set(response.items);
                        total_set.set(response.total);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        auth.logout();
                        navigator.push(Route::Login {});
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

    let role_text = |role: &workshop_common::UserRole| -> String {
        match role {
            workshop_common::UserRole::Admin => "Admin".to_string(),
            workshop_common::UserRole::Mechanic => "Mecánico".to_string(),
            workshop_common::UserRole::Seller => "Vendedor".to_string(),
        }
    };

    let columns: Vec<Column<User>> = vec![
        Column {
            key: "email".to_string(),
            header: "Email".to_string(),
            render: Rc::new(|u: &User| rsx! { span { "{u.email}" } }),
        },
        Column {
            key: "name".to_string(),
            header: "Nombre".to_string(),
            render: Rc::new(
                |u: &User| rsx! { span { class: "text-muted", "{u.display_name.as_deref().unwrap_or(\"-\")}" } },
            ),
        },
        Column {
            key: "role".to_string(),
            header: "Rol".to_string(),
            render: {
                Rc::new(move |u: &User| {
                    let text = role_text(&u.role);
                    rsx! { span { "{text}" } }
                })
            },
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: Rc::new(|u: &User| {
                let text = if u.status == "active" { "Activo" } else { "Inactivo" };
                rsx! { span { "{text}" } }
            }),
        },
        Column {
            key: "actions".to_string(),
            header: String::new(),
            render: {
                Rc::new(move |u: &User| {
                    rsx! {
                        div { class: "table-actions",
                            button {
                                class: "btn-icon btn-edit",
                                title: "Editar",
                                onclick: {
                                    let u = u.clone();
                                    let mut show_form_modal = show_form_modal;
                                    let mut editing_user = editing_user;
                                    move |_| {
                                        editing_user.set(Some(u.clone()));
                                        show_form_modal.set(true);
                                    }
                                },
                                {IconName::Edit.render()}
                            }
                            button {
                                class: "btn-icon btn-danger",
                                title: "Eliminar",
                                onclick: {
                                    let u = u.clone();
                                    let mut show_delete_modal = show_delete_modal;
                                    let mut deleting_user = deleting_user;
                                    move |_| {
                                        deleting_user.set(Some(u.clone()));
                                        show_delete_modal.set(true);
                                    }
                                },
                                {IconName::Trash.render()}
                            }
                        }
                    }
                })
            },
        },
    ];

    let rows = users.read().clone();

    rsx! {
        AppShell { title: "Usuarios".to_string(), active_route: Route::Users {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Usuarios".to_string(),
                header_action: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {
                            editing_user.set(None);
                            show_form_modal.set(true);
                        },
                        "Nuevo usuario"
                    }
                },
                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
                } else if rows.is_empty() {
                    div { class: "empty-state", "No hay usuarios registrados" }
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

            if *show_form_modal.read() {
                UserFormModal {
                    show: true,
                    edit_user: editing_user.read().clone(),
                    on_close: move |_| show_form_modal.set(false),
                    on_saved: move |_| {
                        show_form_modal.set(false);
                        let current = *refresh.read();
                        refresh.set(current + 1);
                    },
                }
            }

            ConfirmModal {
                title: "Eliminar usuario".to_string(),
                message: if let Some(user) = deleting_user.read().as_ref() {
                    format!("¿Eliminar a {}?", user.email)
                } else {
                    "¿Eliminar este usuario?".to_string()
                },
                show: *show_delete_modal.read(),
                confirm_text: Some("Eliminar".to_string()),
                on_confirm: move |_| {
                    let user = match deleting_user.read().as_ref() {
                        Some(u) => u.clone(),
                        None => return,
                    };
                    let client = match auth.api_client() {
                        Some(c) => c,
                        None => return,
                    };
                    spawn(async move {
                        match client.delete_user(user.id).await {
                            Ok(_) => {
                                show_delete_modal.set(false);
                                let current = *refresh.read();
                                refresh.set(current + 1);
                            }
                            Err(e) => {
                                error.set(Some(e.user_message().to_string()));
                            }
                        }
                    });
                },
                on_cancel: move |_| show_delete_modal.set(false),
            }
        }
    }
}
