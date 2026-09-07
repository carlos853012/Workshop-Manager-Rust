use dioxus::prelude::*;
use inventory_common::dto::{CreateProductRequest, PosProductResponse};
use inventory_common::money::format_clp;
use inventory_common::Product;
use rust_decimal::Decimal;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::input::Input;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::modal::Modal;
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

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
    let mut error = use_signal(|| None::<String>);
    let mut refresh = use_signal(|| 0);

    let mut show_create_modal = use_signal(|| false);
    let mut show_edit_modal = use_signal(|| false);
    let mut editing_product = use_signal(|| None::<Product>);

    let mut show_stock_modal = use_signal(|| false);
    let mut stock_product = use_signal(|| None::<PosProductResponse>);

    let mut barcode_search = use_signal(|| "".to_string());

    let modal_key = if *show_edit_modal.read() {
        let ep = editing_product.read();
        match ep.as_ref() {
            Some(p) => format!("edit-{}", p.id),
            None => "edit-unknown".to_string(),
        }
    } else {
        "create".to_string()
    };

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
                                                            let p_id = p.id;
                                                            move |_| {
                                                                let client = match auth.api_client() {
                                                                    Some(c) => c,
                                                                    None => return,
                                                                };
                                                                let p_id = p_id;
                                                                spawn(async move {
                                                                    match client.delete_product(p_id).await {
                                                                        Ok(_) => {
                                                                            let current = *refresh.read();
                                                                            refresh.set(current + 1);
                                                                        }
                                                                        Err(e) => {
                                                                            error.set(Some(e.user_message().to_string()));
                                                                        }
                                                                    }
                                                                });
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
            ProductFormModal {
                key: "{modal_key}",
                show: *show_create_modal.read() || *show_edit_modal.read(),
                edit_product: if *show_edit_modal.read() { editing_product.read().clone() } else { None },
                on_close: move |_| {
                    show_create_modal.set(false);
                    show_edit_modal.set(false);
                    editing_product.set(None);
                },
                on_saved: move |_| {
                    show_create_modal.set(false);
                    show_edit_modal.set(false);
                    editing_product.set(None);
                    let current = *refresh.read();
                    refresh.set(current + 1);
                },
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
        }
    }
}

#[component]
fn ProductFormModal(
    show: bool,
    edit_product: Option<Product>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let is_edit = edit_product.is_some();

    let mut name = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default()
    });
    let mut sku = use_signal(|| {
        edit_product
            .as_ref()
            .and_then(|p| p.sku.clone())
            .unwrap_or_default()
    });
    let mut barcode = use_signal(|| {
        edit_product
            .as_ref()
            .and_then(|p| p.barcode.clone())
            .unwrap_or_default()
    });
    let mut price = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.price.to_string())
            .unwrap_or_default()
    });
    let mut cost = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.cost.to_string())
            .unwrap_or_default()
    });
    let mut stock = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.stock.to_string())
            .unwrap_or_default()
    });
    let mut min_stock = use_signal(|| {
        edit_product
            .as_ref()
            .map(|p| p.min_stock.to_string())
            .unwrap_or_default()
    });
    let saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let title = if is_edit {
        "Editar producto"
    } else {
        "Nuevo producto"
    };

    let on_submit = move |_| {
        let name_value = name.read().trim().to_string();
        if name_value.is_empty() {
            form_error.set(Some("El nombre es obligatorio".to_string()));
            return;
        }
        let price_dec = price.read().parse::<Decimal>().unwrap_or(Decimal::ZERO);
        let cost_dec = cost.read().parse::<Decimal>().unwrap_or(Decimal::ZERO);
        let stock_i = stock.read().parse::<i32>().unwrap_or(0);
        let min_stock_i = min_stock.read().parse::<i32>().unwrap_or(0);

        let request = CreateProductRequest {
            name: name_value,
            description: None,
            category: None,
            brand: None,
            model: None,
            sku: Some(sku.read().clone()).filter(|s| !s.is_empty()),
            barcode: Some(barcode.read().clone()).filter(|s| !s.is_empty()),
            price: price_dec,
            cost: cost_dec,
            stock: stock_i,
            min_stock: min_stock_i,
            location: None,
            supplier_id: None,
        };

        let client = auth.api_client();
        let mut saving_set = saving;
        let mut error_set = form_error;
        let on_saved_clone = on_saved;
        let edit_id = edit_product.as_ref().map(|p| p.id);

        saving_set.set(true);
        error_set.set(None);

        spawn(async move {
            match client {
                Some(client) => {
                    let result = if let Some(id) = edit_id {
                        client.update_product(id, &request).await
                    } else {
                        client.create_product(&request).await
                    };
                    match result {
                        Ok(_) => {
                            on_saved_clone.call(());
                        }
                        Err(e) => {
                            error_set.set(Some(e.user_message().to_string()));
                        }
                    }
                }
                None => {
                    error_set.set(Some("No hay cliente API".to_string()));
                }
            }
            saving_set.set(false);
        });
    };

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
                    "Guardar"
                }
            },
            if let Some(err) = form_error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            div { class: "form-row",
                Input {
                    label: Some("Nombre *".to_string()),
                    value: name.read().clone(),
                    oninput: move |evt: FormEvent| name.set(evt.value().clone()),
                }
                Input {
                    label: Some("SKU".to_string()),
                    value: sku.read().clone(),
                    oninput: move |evt: FormEvent| sku.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Código de barras".to_string()),
                    value: barcode.read().clone(),
                    oninput: move |evt: FormEvent| barcode.set(evt.value().clone()),
                    placeholder: Some("Vacío = auto-generar".to_string()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Precio *".to_string()),
                    r#type: "number".to_string(),
                    value: price.read().clone(),
                    oninput: move |evt: FormEvent| price.set(evt.value().clone()),
                }
                Input {
                    label: Some("Costo".to_string()),
                    r#type: "number".to_string(),
                    value: cost.read().clone(),
                    oninput: move |evt: FormEvent| cost.set(evt.value().clone()),
                }
            }
            div { class: "form-row mt-md",
                Input {
                    label: Some("Stock *".to_string()),
                    r#type: "number".to_string(),
                    value: stock.read().clone(),
                    oninput: move |evt: FormEvent| stock.set(evt.value().clone()),
                }
                Input {
                    label: Some("Stock mínimo".to_string()),
                    r#type: "number".to_string(),
                    value: min_stock.read().clone(),
                    oninput: move |evt: FormEvent| min_stock.set(evt.value().clone()),
                }
            }
        }
    }
}

#[component]
fn StockEntryModal(
    show: bool,
    product: Option<PosProductResponse>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let mut quantity = use_signal(|| "".to_string());
    let saving = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let has_product = product.is_some();
    let product_name_display = product.as_ref().map(|p| p.name.clone()).unwrap_or_default();
    let current_stock = product.as_ref().map(|p| p.stock).unwrap_or(0);
    let product_id = product.as_ref().map(|p| p.product_id);
    let submit_sku = product.as_ref().and_then(|p| p.sku.clone());
    let submit_barcode = product.as_ref().and_then(|p| p.barcode.clone());
    let submit_price = product.as_ref().map(|p| p.price).unwrap_or_default();

    let on_submit = {
        let product_name_display = product_name_display.clone();
        move |_| {
            let qty = match quantity.read().parse::<i32>() {
                Ok(q) if q > 0 => q,
                _ => {
                    form_error.set(Some("Ingrese una cantidad válida".to_string()));
                    return;
                }
            };
            let pid = match product_id {
                Some(id) => id,
                None => return,
            };
            let client = match auth.api_client() {
                Some(c) => c,
                None => {
                    form_error.set(Some("No hay cliente API".to_string()));
                    return;
                }
            };
            let product_name = product_name_display.clone();
            let sku = submit_sku.clone();
            let barcode = submit_barcode.clone();
            let price = submit_price;
            let mut saving_clone = saving;
            let mut error_clone = form_error;
            let on_saved_clone = on_saved;

            saving_clone.set(true);
            error_clone.set(None);

            spawn(async move {
                let new_stock = current_stock + qty;
                let request = CreateProductRequest {
                    name: product_name,
                    description: None,
                    category: None,
                    brand: None,
                    model: None,
                    sku,
                    barcode,
                    price,
                    cost: Decimal::ZERO,
                    stock: new_stock,
                    min_stock: 0,
                    location: None,
                    supplier_id: None,
                };
                match client.update_product(pid, &request).await {
                    Ok(_) => {
                        on_saved_clone.call(());
                    }
                    Err(e) => {
                        error_clone.set(Some(e.user_message().to_string()));
                    }
                }
                saving_clone.set(false);
            });
        }
    };

    if !show || !has_product {
        return rsx! {};
    }

    rsx! {
        Modal {
            show: show,
            title: "Ingreso de stock".to_string(),
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
                    "Confirmar"
                }
            },
            if let Some(err) = form_error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            div { class: "stock-entry-info",
                p { "Producto: " strong { "{product_name_display}" } }
                p { "Stock actual: " strong { "{current_stock}" } }
            }
            div { class: "mt-md",
                Input {
                    label: Some("Cantidad a ingresar *".to_string()),
                    r#type: "number".to_string(),
                    value: quantity.read().clone(),
                    oninput: move |evt: FormEvent| quantity.set(evt.value().clone()),
                    placeholder: Some("Ej: 10".to_string()),
                }
            }
        }
    }
}
