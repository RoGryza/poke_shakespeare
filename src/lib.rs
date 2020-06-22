#![feature(decl_macro)]

mod api;
mod config;
pub mod services;

use rocket::http::{RawStr, Status};
use rocket::response::status;
use rocket::{get, routes, Rocket, State};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use api::{Alpha, Error as ApiError, Result as ApiResult, SerializeErrors};
use config::ReadConfig;
use services::{BoxedPokeApi, BoxedTranslator, PokeApi, Translator};

pub trait RocketExt {
    fn poke_shakespeare(self) -> Self;
    fn poke_shakespeare_custom<P, T>(self, pokeapi: P, translator: T) -> Self
    where
        P: 'static + PokeApi + Send + Sync,
        T: 'static + Translator + Send + Sync;
}

impl RocketExt for Rocket {
    fn poke_shakespeare(self) -> Self {
        self.attach(SerializeErrors)
            .attach(ReadConfig)
            .mount("/", routes![pokemon, pokemon_badrequest])
    }

    fn poke_shakespeare_custom<P, T>(self, pokeapi: P, translator: T) -> Self
    where
        P: 'static + PokeApi + Send + Sync,
        T: 'static + Translator + Send + Sync,
    {
        self.attach(SerializeErrors)
            .manage(BoxedPokeApi::from(Box::new(pokeapi)))
            .manage(BoxedTranslator::from(Box::new(translator)))
            .mount("/", routes![pokemon, pokemon_badrequest])
    }
}

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

#[cfg(test)]
mod test {
    use super::*;

    use rocket::config::{Config, Environment};
    use rocket::http::ContentType;
    use rocket::local::Client;
    use serde::de::DeserializeOwned;

    #[test]
    fn test_pokemon_ok() {
        let rocket = rocket::custom(Config::new(Environment::Development)).poke_shakespeare_custom(
            |name: &str| match name {
                "foo" => Ok(Some("desc foo".to_string())),
                "bar" => Ok(Some("my name is bar".to_string())),
                _ => Ok(None),
            },
            |source: &str| Ok(format!("TRANSLATED: {}", source)),
        );
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
        let rocket = rocket::custom(Config::new(Environment::Development))
            .poke_shakespeare_custom(|_: &str| Ok(None), |s: &str| Ok(s.to_string()));
        let client = Client::new(rocket).unwrap();
        let response = client.get("/pokemon/12").dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let response = client.get("/pokemon/foo&20bar").dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    #[ignore]
    fn test_api_integration() {
        let rocket = rocket::custom(Config::new(Environment::Development)).poke_shakespeare();
        let client = Client::new(rocket).unwrap();
        let response = client.get("/pokemon/notfound").dispatch();
        assert_eq!(response.status(), Status::NotFound);
        let response = client.get("/pokemon/butterfree").dispatch();
        assert_eq!(response.status(), Status::Ok);
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
