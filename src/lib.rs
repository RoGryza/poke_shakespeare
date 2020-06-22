#![feature(decl_macro)]

mod api;
pub mod services;

use rocket::http::Status;
use rocket::{get, routes, Route, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use api::{Alpha, Error as ApiError, Result as ApiResult};
use services::{BoxedPokeApi, BoxedTranslator};

#[derive(Clone, Debug, Serialize)]
pub struct Pokemon {
    pub name: String,
    pub description: String,
}

#[get("/pokemon/<name>")]
fn pokemon(
    pokeapi: State<BoxedPokeApi>,
    translator: State<BoxedTranslator>,
    name: Alpha,
) -> ApiResult<Pokemon> {
    match pokeapi.get_description(&name)? {
        Some(source_description) => {
            let description = translator.translate(&source_description)?;
            Ok(Json(Pokemon {
                name: name.into(),
                description,
            }))
        }
        None => Err(ApiError::Status(Status::NotFound)),
    }
}

pub fn api() -> Vec<Route> {
    routes![pokemon]
}
