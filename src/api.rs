use std::ops::Deref;

use log::error;
use rocket::http::{RawStr, Status};
use rocket::request::FromParam;
use rocket::response::{status, Responder, Result as ResponseResult};
use rocket::Request;
use rocket_contrib::json::Json;
use serde_json::json;

pub type Result<T> = std::result::Result<Json<T>, Error>;

#[derive(Debug)]
pub enum Error {
    Status(Status),
    Other(anyhow::Error),
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(e: E) -> Self {
        Error::Other(e.into())
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, request: &Request) -> ResponseResult<'r> {
        let status = match self {
            Error::Status(s) => s,
            Error::Other(e) => {
                error!("{}", e);
                Status::InternalServerError
            }
        };
        status::Custom(status, Json(json!({ "error": status.reason }))).respond_to(request)
    }
}

#[derive(Clone, Debug)]
pub struct Alpha(String);

impl Into<String> for Alpha {
    fn into(self) -> String {
        self.0
    }
}

impl Deref for Alpha {
    type Target = str;

    fn deref(&self) -> &str {
        self.0.deref()
    }
}

impl<'r> FromParam<'r> for Alpha {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> std::result::Result<Self, Self::Error> {
        String::from_param(param).and_then(|s| {
            if s.chars().all(char::is_alphabetic) {
                Ok(Alpha(s))
            } else {
                Err(param)
            }
        })
    }
}
