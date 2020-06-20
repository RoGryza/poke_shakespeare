#![feature(decl_macro)]

pub mod services;

use rocket::{get, routes, Route, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use services::{BoxedPokeApi, BoxedTranslator, Result};

#[derive(Serialize)]
pub struct Pokemon {
    pub name: String,
    pub description: String,
}

#[get("/pokemon/<name>")]
fn pokemon(
    pokeapi: State<BoxedPokeApi>,
    translator: State<BoxedTranslator>,
    name: String,
) -> Result<Json<Pokemon>> {
    let source_description = pokeapi.get_description(&name)?;
    let description = translator.translate(&source_description)?;
    Ok(Json(Pokemon { name, description }))
}

pub fn api() -> Vec<Route> {
    routes![pokemon]
}
