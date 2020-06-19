#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![feature(decl_macro)]

use rocket::{get, routes, Route};
use rocket::response::content;
use serde::Serialize;

#[derive(Serialize)]
pub struct Pokemon {
    pub name: String,
    pub description: String,
}

#[get("/pokemon/<name>")]
fn pokemon(name: String) -> content::Json<String> {
    content::Json(serde_json::to_string(&Pokemon {
        name,
        description: "FooBar".into(),
    }).unwrap())
}

pub fn api() -> Vec<Route> {
    routes![pokemon]
}
