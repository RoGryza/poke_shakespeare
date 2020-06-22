#![feature(decl_macro)]

mod api;
pub mod services;

use rocket::http::{RawStr, Status};
use rocket::response::status;
use rocket::{get, routes, Route, State};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

pub use api::SerializeErrors;
use api::{Alpha, Error as ApiError, Result as ApiResult};
use services::{BoxedPokeApi, BoxedTranslator};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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

#[get("/pokemon/<_name>", rank = 2)]
fn pokemon_badrequest(_name: &RawStr) -> status::BadRequest<()> {
    status::BadRequest(None)
}

pub fn api() -> Vec<Route> {
    routes![pokemon, pokemon_badrequest]
}

#[cfg(test)]
mod test {
    use super::*;

    use rocket::http::ContentType;
    use rocket::local::Client;
    use serde::de::DeserializeOwned;
    use services::{DummyPokeApi, DummyTranslator};

    #[test]
    fn test_pokemon_ok() {
        let rocket = rocket::ignite()
            .attach(SerializeErrors)
            .manage(BoxedPokeApi::from(Box::new(
                vec![
                    ("foo".to_string(), "desc foo".to_string()),
                    ("bar".to_string(), "my name is bar".to_string()),
                ]
                .into_iter()
                .collect::<DummyPokeApi>(),
            )))
            .manage(BoxedTranslator::from(Box::new(DummyTranslator::new())))
            .mount("/", api());
        let client = Client::new(rocket).unwrap();

        assert_eq!(
            (
                Status::Ok,
                Pokemon {
                    name: "foo".into(),
                    description: "TRANSLATED: desc foo".into(),
                }
            ),
            json_get(&client, "/pokemon/foo"),
        );

        assert_eq!(
            (
                Status::Ok,
                Pokemon {
                    name: "bar".into(),
                    description: "TRANSLATED: my name is bar".into(),
                }
            ),
            json_get(&client, "/pokemon/bar"),
        );
    }

    #[test]
    fn test_invalid_param_responds_bad_request() {
        let rocket = rocket::ignite()
            .attach(SerializeErrors)
            .manage(BoxedPokeApi::from(Box::new(DummyPokeApi::default())))
            .manage(BoxedTranslator::from(Box::new(DummyTranslator::new())))
            .mount("/", api());
        let client = Client::new(rocket).unwrap();
        let response = client.get("/pokemon/12").dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let response = client.get("/pokemon/foo&20bar").dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }

    fn json_get<T>(client: &Client, endpoint: &str) -> (Status, T)
    where
        T: DeserializeOwned,
    {
        let mut response = client.get(endpoint).dispatch();
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let bytes = response.body_bytes().expect("Body must not be empty");
        (response.status(), serde_json::from_slice(&bytes).unwrap())
    }
}
