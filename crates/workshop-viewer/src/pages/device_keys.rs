use dioxus::prelude::*;
use workshop_common::dto::DeviceKeySummary;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
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
pub fn DeviceKeys() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let keys = use_signal(Vec::<DeviceKeySummary>::new);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let mut refresh = use_signal(|| 0u32);

    let mut show_revoke_modal = use_signal(|| false);
    let mut revoking_key_id = use_signal(|| None::<uuid::Uuid>);

    let mut show_new_key_modal = use_signal(|| false);
    let mut new_key_value = use_signal(|| None::<String>);

    let load_data = move || {
        let client = auth.api_client();
        let mut keys_set = keys;
        let mut loading_set = loading;
        let mut error_set = error;
        loading_set.set(true);
        error_set.set(None);

        let mut auth = auth;
        spawn(async move {
            if let Some(client) = client {
                match client.list_device_keys().await {
                    Ok(data) => {
                        keys_set.set(data);
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
        let _ = refresh.read();
        load_data();
    });

    let on_generate = move |_| {
        let client = auth.api_client();
        let mut error_set = error;
        let mut new_key_value_set = new_key_value;
        let mut show_new_key_modal_set = show_new_key_modal;

        spawn(async move {
            if let Some(client) = client {
                match client.generate_device_key().await {
                    Ok(key) => {
                        new_key_value_set.set(Some(key));
                        show_new_key_modal_set.set(true);
                        let current = *refresh.read();
                        refresh.set(current + 1);
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
            }
        });
    };

    let on_confirm_revoke = move |_| {
        let client = auth.api_client();
        let key_id = *revoking_key_id.read();
        let mut error_set = error;
        let mut show_revoke_modal_set = show_revoke_modal;

        if let Some(id) = key_id {
            spawn(async move {
                if let Some(client) = client {
                    match client.revoke_device_key(id).await {
                        Ok(()) => {
                            show_revoke_modal_set.set(false);
                            let current = *refresh.read();
                            refresh.set(current + 1);
                        }
                        Err(e) => {
                            error_set.set(Some(e.user_message().to_string()));
                        }
                    }
                }
            });
        }
    };

    let on_cancel_revoke = move |_| show_revoke_modal.set(false);

    let on_unbind = move |key_id: uuid::Uuid| {
        let client = auth.api_client();
        let mut error_set = error;

        spawn(async move {
            if let Some(client) = client {
                match client.unbind_device_key(key_id).await {
                    Ok(()) => {
                        let current = *refresh.read();
                        refresh.set(current + 1);
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
            }
        });
    };

    let columns: Vec<Column<DeviceKeySummary>> = vec![
        Column {
            key: "id".to_string(),
            header: "ID".to_string(),
            render: Rc::new(|dk: &DeviceKeySummary| {
                let id_str = dk.id.to_string();
                rsx! { span { class: "text-mono text-xs", "{id_str}" } }
            }),
        },
        Column {
            key: "ip".to_string(),
            header: "IP Vinculada".to_string(),
            render: Rc::new(|dk: &DeviceKeySummary| {
                let ip = dk.bound_ip.as_deref().unwrap_or("—");
                rsx! { span { class: "text-muted", "{ip}" } }
            }),
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: Rc::new(|dk: &DeviceKeySummary| {
                if dk.active {
                    rsx! { span { class: "text-success", "Activa" } }
                } else {
                    rsx! { span { class: "text-danger", "Revocada" } }
                }
            }),
        },
        Column {
            key: "created".to_string(),
            header: "Creada".to_string(),
            render: Rc::new(|dk: &DeviceKeySummary| {
                let ts = dk.created_at.format("%d/%m/%Y %H:%M").to_string();
                rsx! { span { class: "text-muted", "{ts}" } }
            }),
        },
        Column {
            key: "last_seen".to_string(),
            header: "Último uso".to_string(),
            render: Rc::new(|dk: &DeviceKeySummary| {
                let ts = match dk.last_seen_at {
                    Some(dt) => dt.format("%d/%m/%Y %H:%M").to_string(),
                    None => "—".to_string(),
                };
                rsx! { span { class: "text-muted", "{ts}" } }
            }),
        },
        Column {
            key: "actions".to_string(),
            header: String::new(),
            render: {
                Rc::new(move |dk: &DeviceKeySummary| {
                    rsx! {
                        div { class: "table-actions",
                            if dk.active {
                                button {
                                    class: "btn-icon btn-danger",
                                    title: "Revocar",
                                    onclick: {
                                        let key_id = dk.id;
                                        move |_| {
                                            revoking_key_id.set(Some(key_id));
                                            show_revoke_modal.set(true);
                                        }
                                    },
                                    {IconName::Ban.render()}
                                }
                            }
                            if dk.active && dk.bound_ip.is_some() {
                                button {
                                    class: "btn-icon btn-edit",
                                    title: "Desvincular",
                                    onclick: {
                                        let key_id = dk.id;
                                        move |_| on_unbind(key_id)
                                    },
                                    {IconName::Unlink.render()}
                                }
                            }
                        }
                    }
                })
            },
        },
    ];

    let rows = keys.read().clone();
    let err_snapshot = error.read().clone();
    let show_revoke = *show_revoke_modal.read();
    let show_new_key = *show_new_key_modal.read();
    let new_key_snapshot = new_key_value.read().clone();
    let new_key_display = new_key_snapshot.unwrap_or_default();

    rsx! {
        AppShell {
            title: "Claves de Dispositivo".to_string(),
            active_route: Route::DeviceKeys {},

            if err_snapshot.is_some() {
                div { class: "alert alert-danger mb-md", "{err_snapshot.as_deref().unwrap_or_default()}" }
            }

            Card {
                title: "Claves de Dispositivo".to_string(),
                header_action: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: on_generate,
                        "Generar Clave"
                    }
                },
                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
                } else {
                    DataTable {
                        columns: columns.clone(),
                        rows: rows.clone(),
                    }
                }
            }
        }

        ConfirmModal {
            title: "Revocar clave".to_string(),
            message: "La clave será desactivada permanentemente. ¿Continuar?".to_string(),
            show: show_revoke,
            confirm_text: Some("Revocar".to_string()),
            on_confirm: on_confirm_revoke,
            on_cancel: on_cancel_revoke,
        }

        if show_new_key && !new_key_display.is_empty() {
            Modal {
                show: true,
                title: "Clave Generada".to_string(),
                on_close: move |_| {
                    show_new_key_modal.set(false);
                    new_key_value.set(None);
                },
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {
                            show_new_key_modal.set(false);
                            new_key_value.set(None);
                        },
                        "Cerrar"
                    }
                },
                p { class: "text-muted", "Copia esta clave y guárdala en un lugar seguro. No se volverá a mostrar." }
                div { class: "mt-md",
                    input {
                        class: "input",
                        value: "{new_key_display}",
                        readonly: true,
                    }
                }
            }
        }
    }
}
