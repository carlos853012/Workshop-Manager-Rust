use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::pages::home::{Dashboard, Root};
use crate::pages::login::Login;
use crate::pages::products::Products;
use crate::pages::repairs::Repairs;
use crate::pages::reports::Reports;
use crate::pages::sales::Sales;
use crate::pages::suppliers::Suppliers;
use crate::pages::users::Users;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Root {},

    #[route("/login")]
    Login {},

    #[route("/dashboard")]
    Dashboard {},

    #[route("/products")]
    Products {},

    #[route("/sales")]
    Sales {},

    #[route("/repairs")]
    Repairs {},

    #[route("/suppliers")]
    Suppliers {},

    #[route("/reports")]
    Reports {},

    #[route("/users")]
    Users {},
}
