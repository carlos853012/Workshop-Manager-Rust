use dioxus::prelude::*;
use inventory_common::dto::PosProductResponse;
use inventory_common::money::format_clp;
use inventory_common::Product;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::confirm_modal::ConfirmModal;
use crate::components::organisms::product_form_modal::ProductFormModal;
use crate::components::organisms::stock_entry_modal::StockEntryModal;
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn Products() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let products = use_signal(Vec::<Product>::new);
    let mut page = use_signal(|| 1);
    let total = use_signal(|| 0);
    let loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut refresh = use_signal(|| 0);

    let mut show_create_modal = use_signal(|| false);
    let mut show_edit_modal = use_signal(|| false);
    let mut editing_product = use_signal(|| None::<Product>);

    let mut show_stock_modal = use_signal(|| false);
    let mut stock_product = use_signal(|| None::<PosProductResponse>);

    let mut barcode_search = use_signal(|| "".to_string());

    let mut show_delete_confirm = use_signal(|| false);
    let deleting_product = use_signal(|| None::<Product>);

    let load_data = move || {
        let client = auth.api_client();
        let mut products_set = products;
        let mut total_set = total;
        let mut loading_set = loading;
        let mut error_set = error;
        let current_page = *page.read();

        loading_set.set(true);
        error_set.set(None);

        let mut auth = auth;
        spawn(async move {
            match client {
                Some(client) => match client.list_products(current_page, 10).await {
                    Ok(response) => {
                        products_set.set(response.items);
                        total_set.set(response.total);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        auth.logout();
                        navigator.push(Route::Login {});
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
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
        let _ = *refresh.read();
        load_data();
    });

    let rows = products.read().clone();
    let total_pages = ((*total.read() + 9) / 10) as i32;

    rsx! {
        AppShell { title: "Productos".to_string(), active_route: Route::Products {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            div { class: "products-scan-wrapper",
                input {
                    class: "products-scan-input",
                    id: "products-barcode-input",
                    r#type: "text",
                    value: "{barcode_search}",
                    placeholder: "Escanear o escribir código...",
                    oninput: move |evt: FormEvent| barcode_search.set(evt.value().clone()),
                    onkeydown: {
                        let mut barcode_search = barcode_search;
                        let mut show_stock_modal = show_stock_modal;
                        let mut stock_product = stock_product;
                        let mut show_create_modal = show_create_modal;
                        let mut error = error;
                        move |evt: Event<KeyboardData>| {
                            if evt.key() == Key::Enter {
                                let code = barcode_search.read().trim().to_string();
                                if code.is_empty() {
                                    return;
                                }
                                let client = match auth.api_client() {
                                    Some(c) => c,
                                    None => {
                                        error.set(Some("No hay cliente API".to_string()));
                                        return;
                                    }
                                };
                                error.set(None);
                                spawn(async move {
                                    match client.lookup_product_by_barcode(&code).await {
                                        Ok(product) => {
                                            stock_product.set(Some(product));
                                            show_stock_modal.set(true);
                                            barcode_search.set(String::new());
                                        }
                                        Err(ApiError::NotFound(_)) => {
                                            barcode_search.set(String::new());
                                            show_create_modal.set(true);
                                        }
                                        Err(e) => {
                                            error.set(Some(e.user_message().to_string()));
                                            barcode_search.set(String::new());
                                        }
                                    }
                                });
                            }
                        }
                    },
                }
            }
            Card {
                title: "Listado de productos".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_create_modal.set(true),
                        "Nuevo producto"
                    }
                },
                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
                } else {
                    div { class: "products-table",
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "Nombre" }
                                    th { "SKU" }
                                    th { "Código" }
                                    th { "Precio" }
                                    th { "Stock" }
                                    th { class: "col-actions", "" }
                                }
                            }
                            tbody {
                                if rows.is_empty() {
                                    tr {
                                        td { colspan: "6", div { class: "empty-state", "No hay productos" } }
                                    }
                                } else {
                                    for p in rows.iter() {
                                        tr {
                                            td { "{p.name}" }
                                            td { class: "text-muted", "{p.sku.as_deref().unwrap_or(\"-\")}" }
                                            td { class: "text-muted", "{p.barcode.as_deref().unwrap_or(\"-\")}" }
                                            td { class: "text-right", "{format_clp(p.price)}" }
                                            td {
                                                if p.stock <= p.min_stock {
                                                    span { class: "text-danger", "{p.stock}" }
                                                } else {
                                                    span { "{p.stock}" }
                                                }
                                            }
                                            td { class: "col-actions",
                                                div { class: "table-actions",
                                                    button {
                                                        class: "btn-icon btn-edit",
                                                        title: "Editar",
                                                        onclick: {
                                                            let p = p.clone();
                                                            let mut editing_product = editing_product;
                                                            let mut show_edit_modal = show_edit_modal;
                                                            move |_| {
                                                                editing_product.set(Some(p.clone()));
                                                                show_edit_modal.set(true);
                                                            }
                                                        },
                                                        {IconName::Edit.render()}
                                                    }
                                                    button {
                                                        class: "btn-icon btn-danger",
                                                        title: "Borrar",
                                                        onclick: {
                                                            let p = p.clone();
                                                            let mut deleting_product = deleting_product;
                                                            let mut show_delete_confirm = show_delete_confirm;
                                                            move |_| {
                                                                deleting_product.set(Some(p.clone()));
                                                                show_delete_confirm.set(true);
                                                            }
                                                        },
                                                        {IconName::Trash.render()}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if *total.read() > 10 {
                            div { class: "pagination",
                                button {
                                    class: "btn btn-ghost",
                                    disabled: *page.read() <= 1,
                                    onclick: move |_| { let p = *page.read() - 1; page.set(p); },
                                    "Anterior"
                                }
                                span { class: "text-muted", "Página {page} de {total_pages}" }
                                button {
                                    class: "btn btn-ghost",
                                    disabled: *page.read() >= total_pages,
                                    onclick: move |_| { let p = *page.read() + 1; page.set(p); },
                                    "Siguiente"
                                }
                            }
                        }
                    }
                }
            }
            if *show_create_modal.read() {
                ProductFormModal {
                    show: true,
                    edit_product: None,
                    on_close: move |_| {
                        show_create_modal.set(false);
                    },
                    on_saved: move |_| {
                        show_create_modal.set(false);
                        let current = *refresh.read();
                        refresh.set(current + 1);
                    },
                }
            }
            if *show_edit_modal.read() {
                if let Some(ep) = editing_product.read().clone() {
                    ProductFormModal {
                        show: true,
                        edit_product: Some(ep),
                        on_close: move |_| {
                            show_edit_modal.set(false);
                            editing_product.set(None);
                        },
                        on_saved: move |_| {
                            show_edit_modal.set(false);
                            editing_product.set(None);
                            let current = *refresh.read();
                            refresh.set(current + 1);
                        },
                    }
                }
            }
            StockEntryModal {
                show: *show_stock_modal.read(),
                product: stock_product.read().clone(),
                on_close: move |_| {
                    show_stock_modal.set(false);
                    stock_product.set(None);
                },
                on_saved: move |_| {
                    show_stock_modal.set(false);
                    stock_product.set(None);
                    let current = *refresh.read();
                    refresh.set(current + 1);
                },
            }
            ConfirmModal {
                title: "Eliminar producto".to_string(),
                message: if let Some(p) = deleting_product.read().as_ref() {
                    format!("¿Eliminar el producto \"{}\"? Esta acción no se puede deshacer.", p.name)
                } else {
                    "¿Eliminar este producto?".to_string()
                },
                show: *show_delete_confirm.read(),
                confirm_text: Some("Eliminar".to_string()),
                on_confirm: move |_| {
                    let product = match deleting_product.read().as_ref() {
                        Some(p) => p.clone(),
                        None => return,
                    };
                    let client = match auth.api_client() {
                        Some(c) => c,
                        None => return,
                    };
                    spawn(async move {
                        match client.delete_product(product.id).await {
                            Ok(_) => {
                                show_delete_confirm.set(false);
                                let current = *refresh.read();
                                refresh.set(current + 1);
                            }
                            Err(e) => {
                                error.set(Some(e.user_message().to_string()));
                            }
                        }
                    });
                },
                on_cancel: move |_| show_delete_confirm.set(false),
            }
        }
    }
}
