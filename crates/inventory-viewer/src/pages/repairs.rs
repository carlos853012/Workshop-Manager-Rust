use dioxus::prelude::*;
use inventory_common::Repair;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::badge::{Badge, BadgeVariant};
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{AppShell, require_auth};

#[component]
pub fn Repairs() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let repairs = use_signal(Vec::<Repair>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);

    let load_data = move || {
        let client = auth.api_client();
        let mut repairs_set = repairs;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => match client.list_repairs(current_page, 10).await {
                    Ok(response) => {
                        repairs_set.set(response.items);
                        total_set.set(response.total);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        error_set.set(Some("Sesión expirada".to_string()));
                    }
                    Err(e) => {
                        error_set.set(Some(e.to_string()));
                    }
                },
                None => {
                    error_set.set(Some("No hay cliente API".to_string()));
                }
            }
            loading_set.set(false);
        });
    };

    use_effect(move || {
        load_data();
    });

    let columns: Vec<Column<Repair>> = vec![
        Column {
            key: "customer".to_string(),
            header: "Cliente".to_string(),
            render: |r| rsx! { span { "{r.customer_name.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "motorcycle".to_string(),
            header: "Motocicleta".to_string(),
            render: |r| rsx! { span { "{r.motorcycle.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: |r| rsx! {
                Badge {
                    variant: match r.status {
                        inventory_common::RepairStatus::Completed => BadgeVariant::Success,
                        inventory_common::RepairStatus::Cancelled => BadgeVariant::Danger,
                        inventory_common::RepairStatus::InProgress => BadgeVariant::Warning,
                        inventory_common::RepairStatus::Pending => BadgeVariant::Info,
                    },
                    "{r.status}"
                }
            },
        },
        Column {
            key: "priority".to_string(),
            header: "Prioridad".to_string(),
            render: |r| rsx! { span { "{r.priority}" } },
        },
    ];

    let rows = repairs.read().clone();

    rsx! {
        AppShell { title: "Reparaciones".to_string(), active_route: "/repairs".to_string(),
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Listado de reparaciones".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {},
                        "Nueva reparación"
                    }
                },
                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
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
        }
    }
}
