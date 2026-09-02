use dioxus::prelude::*;
use inventory_common::Repair;

use crate::components::atoms::badge::{Badge, BadgeVariant};
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{require_auth, AppShell};

#[component]
pub fn Repairs() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let repairs = use_signal(Vec::<Repair>::new);
    let page = use_signal(|| 1);
    let total = use_signal(|| 0);

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
            render: |r| {
                rsx! {
                    Badge {
                        variant: match r.status {
                            inventory_common::RepairStatus::Completed => BadgeVariant::Success,
                            inventory_common::RepairStatus::Cancelled => BadgeVariant::Danger,
                            inventory_common::RepairStatus::InProgress => BadgeVariant::Warning,
                            inventory_common::RepairStatus::Pending => BadgeVariant::Info,
                        },
                        "{r.status}"
                    }
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
            Card {
                title: "Listado de reparaciones".to_string(),
                footer: rsx! {
                    Button { variant: ButtonVariant::Primary, "Nueva reparación" }
                },
                DataTable {
                    columns: columns.clone(),
                    rows: rows.clone(),
                    page: *page.read(),
                    total: *total.read(),
                }
            }
        }
    }
}
