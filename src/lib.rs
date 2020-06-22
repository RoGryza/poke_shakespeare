#![feature(decl_macro)]

mod api;
pub mod services;

use rocket::http::Status;
use rocket::{get, routes, Route, State};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

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

pub fn api() -> Vec<Route> {
    routes![pokemon]
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::{bail, Result};
    use api::ErrorPayload;
    use rocket::http::ContentType;
    use rocket::local::Client;
    use serde::de::DeserializeOwned;
    use services::{DummyPokeApi, DummyTranslator, PokeApi};

    #[test]
    fn test_pokemon_ok() {
        let rocket = rocket::ignite()
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
    fn test_errors_sent_as_json() {
        struct TestPokeApi;
        impl PokeApi for TestPokeApi {
            fn get_description(&self, name: &str) -> Result<Option<String>> {
                match name {
                    "notfound" => Ok(None),
                    _ => bail!("Never works!"),
                }
            }
        }

        let rocket = rocket::ignite()
            .manage(BoxedPokeApi::from(Box::new(TestPokeApi)))
            .manage(BoxedTranslator::from(Box::new(DummyTranslator::new())))
            .mount("/", api());
        let client = Client::new(rocket).unwrap();

        json_get_err(&client, "/pokemon/notfound", Status::NotFound);
        json_get_err(&client, "/foo", Status::NotFound);
        json_get_err(&client, "/pokemon", Status::NotFound);
        json_get_err(&client, "/pokemon/", Status::NotFound);
        json_get_err(&client, "/pokemon/12", Status::BadRequest);
        json_get_err(&client, "/pokemon/foo", Status::InternalServerError);
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

    fn json_get_err(client: &Client, endpoint: &str, status: Status) {
        let (st, payload) = json_get(client, endpoint);
        assert_eq!(st, status);
        assert_eq!(ErrorPayload::new(st.reason), payload);
    }
}
