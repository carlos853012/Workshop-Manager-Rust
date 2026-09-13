use dioxus::prelude::*;
use inventory_common::dto::DeviceKeySummary;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::confirm_modal::ConfirmModal;
use crate::components::molecules::modal::Modal;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

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
    let success = use_signal(|| None::<String>);
    let mut refresh_token = use_signal(|| 0u32);

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
        let _ = refresh_token.read();
        load_data();
    });

    let on_generate = move |_| {
        let client = auth.api_client();
        let mut error_set = error;
        let mut success_set = success;
        let mut new_key_value_set = new_key_value;
        let mut show_new_key_modal_set = show_new_key_modal;

        spawn(async move {
            if let Some(client) = client {
                match client.generate_device_key().await {
                    Ok(key) => {
                        new_key_value_set.set(Some(key));
                        show_new_key_modal_set.set(true);
                        success_set.set(Some("Clave generada correctamente".to_string()));
                        let current = *refresh_token.read();
                        refresh_token.set(current + 1);
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
        let mut success_set = success;
        let mut show_revoke_modal_set = show_revoke_modal;

        if let Some(id) = key_id {
            spawn(async move {
                if let Some(client) = client {
                    match client.revoke_device_key(id).await {
                        Ok(()) => {
                            success_set.set(Some("Clave revocada".to_string()));
                            show_revoke_modal_set.set(false);
                            let current = *refresh_token.read();
                            refresh_token.set(current + 1);
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
        let mut success_set = success;

        spawn(async move {
            if let Some(client) = client {
                match client.unbind_device_key(key_id).await {
                    Ok(()) => {
                        success_set.set(Some("Clave desvinculada".to_string()));
                        let current = *refresh_token.read();
                        refresh_token.set(current + 1);
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
            }
        });
    };

    let keys_snapshot = keys.read().clone();
    let is_loading = *loading.read();
    let err_snapshot = error.read().clone();
    let success_snapshot = success.read().clone();
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
            if success_snapshot.is_some() {
                div { class: "alert alert-success", "{success_snapshot.as_deref().unwrap_or_default()}" }
            }

            div { class: "flex justify-end mb-4",
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: on_generate,
                    "Generar Clave"
                }
            }

            Card {
                if is_loading {
                    div { class: "empty-state", Spinner {} }
                } else if keys_snapshot.is_empty() {
                    div { class: "empty-state", "No hay claves de dispositivo registradas" }
                } else {
                    div { class: "data-table-wrapper",
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "ID" }
                                    th { "IP Vinculada" }
                                    th { "Estado" }
                                    th { "Creada" }
                                    th { "Último uso" }
                                    th { "Acciones" }
                                }
                            }
                            tbody {
                                for dk in keys_snapshot.iter() {
                                    KeyRow {
                                        device_key: dk.clone(),
                                        on_revoke: {
                                            let key_id = dk.id;
                                            move |_| {
                                                revoking_key_id.set(Some(key_id));
                                                show_revoke_modal.set(true);
                                            }
                                        },
                                        on_unbind: {
                                            let key_id = dk.id;
                                            move |_| on_unbind(key_id)
                                        },
                                    }
                                }
                            }
                        }
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

#[component]
fn KeyRow(
    device_key: DeviceKeySummary,
    on_revoke: EventHandler<MouseEvent>,
    on_unbind: EventHandler<MouseEvent>,
) -> Element {
    let id_str = device_key.id.to_string();
    let is_active = device_key.active;
    let has_ip = device_key.bound_ip.is_some();
    let created_str = device_key.created_at.format("%d/%m/%Y %H:%M").to_string();
    let last_seen_str = match device_key.last_seen_at {
        Some(dt) => dt.format("%d/%m/%Y %H:%M").to_string(),
        None => "—".to_string(),
    };
    let ip_str = device_key.bound_ip.unwrap_or_else(|| "—".to_string());

    rsx! {
        tr {
            td { span { class: "text-mono text-xs", "{id_str}" } }
            td { span { class: "text-muted", "{ip_str}" } }
            td {
                if is_active {
                    span { class: "text-success", "Activa" }
                } else {
                    span { class: "text-danger", "Revocada" }
                }
            }
            td { span { class: "text-muted", "{created_str}" } }
            td { span { class: "text-muted", "{last_seen_str}" } }
            td {
                div { class: "flex gap-2",
                    if is_active {
                        Button { variant: ButtonVariant::Danger, onclick: on_revoke, "Revocar" }
                    }
                    if has_ip && is_active {
                        Button { variant: ButtonVariant::Secondary, onclick: on_unbind, "Desvincular" }
                    }
                }
            }
        }
    }
}
