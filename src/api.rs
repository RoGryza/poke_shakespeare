//! API and Rocket-related types
use std::ops::Deref;

use log::error;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{ContentType, RawStr, Status, StatusClass};
use rocket::request::FromParam;
use rocket::response::{status, Responder, Result as ResponseResult};
use rocket::{Request, Response};
use rocket_contrib::json::Json;
use serde::de::{Deserializer, Error as _, Unexpected};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// JSON payload sent by the server on HTTP errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub error: String,
}

impl ErrorPayload {
    pub fn new<S: Into<String>>(s: S) -> Self {
        ErrorPayload { error: s.into() }
    }
}

impl From<Status> for ErrorPayload {
    fn from(status: Status) -> Self {
        ErrorPayload::new(status.reason)
    }
}

/// `Fairing` which serializes all 4xx and 5xx HTTP errors as JSON. Formats the body as
/// `JsonPayload`.
#[derive(Clone, Copy, Debug)]
pub struct SerializeErrors;

impl Fairing for SerializeErrors {
    fn info(&self) -> Info {
        Info {
            name: "Serialize Errors",
            kind: Kind::Response,
        }
    }

    fn on_response(&self, _: &Request, response: &mut Response) {
        match response.status().class() {
            StatusClass::ClientError | StatusClass::ServerError
                if response.content_type() != Some(ContentType::JSON) =>
            {
                response.set_header(ContentType::JSON);
                let body = serde_json::to_vec(&ErrorPayload::from(response.status()))
                    .expect("ErrorPayload should be serializable");
                response.set_sized_body(Cursor::new(body));
            }
            _ => (),
        }
    }
}

pub type Result<T> = std::result::Result<Json<T>, Error>;

/// API error response type. Use `Status` for user-facing errors and `Other` for internal errors.
/// `Other` errors are logged.
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
        status::Custom(status, Json(ErrorPayload::new(status.reason))).respond_to(request)
    }
}

/// String containing only alphabetic characters.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Alpha(String);

impl Alpha {
    /// Validates a string and constructs a new `Alpha`, returns `None` if the input is invalid.
    ///
    /// This type implements `FromParam` and `Deserialize`, so it can be used for validation in
    /// rocket and serde, respectively.
    pub fn try_new(s: String) -> Option<Self> {
        if !s.is_empty() && s.chars().all(char::is_alphabetic) {
            Some(Alpha(s))
        } else {
            None
        }
    }
}

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
        String::from_param(param).and_then(|s| Alpha::try_new(s).ok_or(param))
    }
}

impl<'de> Deserialize<'de> for Alpha {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Alpha, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|s| {
            Alpha::try_new(s.clone()).ok_or(D::Error::invalid_value(
                Unexpected::Str(&s),
                &"an alpha string",
            ))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::anyhow;
    use rocket::local::Client;
    use rocket::{get, routes};

    #[test]
    fn test_alpha_parse_ok() {
        Alpha::from_param("foo".into()).unwrap();
    }

    #[test]
    fn test_alpha_parse_invalid() {
        Alpha::from_param("".into()).unwrap_err();
        Alpha::from_param("123".into()).unwrap_err();
        Alpha::from_param(".".into()).unwrap_err();
    }

    #[test]
    fn test_serialize_errors() {
        #[get("/status?<code>")]
        fn status(code: u16) -> Status {
            Status::from_code(code).unwrap()
        }

        #[get("/err?<code>")]
        fn err(code: Option<u16>) -> Error {
            code.and_then(Status::from_code)
                .map_or(Error::Other(anyhow!("Some error")), Error::Status)
        }

        let rocket = rocket::ignite()
            .attach(SerializeErrors)
            .mount("/", routes![status, err]);
        let client = Client::new(rocket).unwrap();
        for (status, endpoint) in vec![
            (Status::NotFound, "/foo"),
            (Status::BadRequest, "/status?code=400"),
            (Status::InternalServerError, "/status?code=500"),
            (Status::InternalServerError, "/err"),
            (Status::BadRequest, "/err?code=400"),
        ] {
            let mut response = client.get(endpoint).dispatch();
            assert_eq!(response.content_type(), Some(ContentType::JSON));
            assert_eq!(response.status(), status);
            let bytes = response.body_bytes().expect("Body should not be empty");
            let payload = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(ErrorPayload::from(status), payload);
        }
    }
}
