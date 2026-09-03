use dioxus::prelude::*;
use inventory_common::Product;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::badge::{Badge, BadgeVariant};
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{AppShell, require_auth};

#[component]
pub fn Products() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let products = use_signal(Vec::<Product>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);

    let load_data = move || {
        let client = auth.api_client();
        let mut products_set = products;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => match client.list_products(current_page, 10).await {
                    Ok(response) => {
                        products_set.set(response.items);
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

    let columns: Vec<Column<Product>> = vec![
        Column {
            key: "name".to_string(),
            header: "Nombre".to_string(),
            render: |p| rsx! { span { "{p.name}" } },
        },
        Column {
            key: "sku".to_string(),
            header: "SKU".to_string(),
            render: |p| rsx! { span { class: "text-muted", "{p.sku.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "price".to_string(),
            header: "Precio".to_string(),
            render: |p| rsx! { span { "${p.price}" } },
        },
        Column {
            key: "stock".to_string(),
            header: "Stock".to_string(),
            render: |p| rsx! {
                if p.stock <= p.min_stock {
                    Badge { variant: BadgeVariant::Danger, "{p.stock}" }
                } else {
                    span { "{p.stock}" }
                }
            },
        },
    ];

    let rows = products.read().clone();

    rsx! {
        AppShell { title: "Productos".to_string(), active_route: "/products".to_string(),
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            Card {
                title: "Listado de productos".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {},
                        "Nuevo producto"
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
