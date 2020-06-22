#![feature(decl_macro)]

pub mod services;

use log::error;
use rocket::http::Status;
use rocket::response::{status, Responder, Result as ResponseResult};
use rocket::{get, routes, Request, Route, State};
use rocket_contrib::json::Json;
use serde::Serialize;
use serde_json::json;

use services::{BoxedPokeApi, BoxedTranslator};

#[derive(Serialize)]
pub struct Pokemon {
    pub name: String,
    pub description: String,
}

type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Debug)]
enum ApiError {
    Status(Status),
    Other(anyhow::Error),
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(e: E) -> Self {
        ApiError::Other(e.into())
    }
}

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, request: &Request) -> ResponseResult<'r> {
        let status = match self {
            ApiError::Status(s) => s,
            ApiError::Other(e) => {
                error!("{}", e);
                Status::InternalServerError
            }
        };
        status::Custom(status, Json(json!({ "error": status.reason }))).respond_to(request)
    }
}

#[get("/pokemon/<name>")]
fn pokemon(
    pokeapi: State<BoxedPokeApi>,
    translator: State<BoxedTranslator>,
    name: String,
) -> ApiResult<Pokemon> {
    match pokeapi.get_description(&name)? {
        Some(source_description) => {
            let description = translator.translate(&source_description)?;
            Ok(Json(Pokemon { name, description }))
        }
        None => Err(ApiError::Status(Status::NotFound)),
    }
}

pub fn api() -> Vec<Route> {
    routes![pokemon]
}
