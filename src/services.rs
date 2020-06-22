use anyhow::{anyhow, Context, Result};

use log::warn;
use serde::Deserialize;
pub type BoxedPokeApi = Box<dyn PokeApi + Send + Sync>;

pub trait PokeApi {
    fn get_description(&self, name: &str) -> Result<Option<String>>;
}

impl<F> PokeApi for F
where
    F: Fn(&str) -> Result<Option<String>>,
{
    fn get_description(&self, name: &str) -> Result<Option<String>> {
        self(name)
    }
}

pub struct PokeApiClient {
    pub base_url: String,
}

impl Default for PokeApiClient {
    fn default() -> Self {
        PokeApiClient {
            base_url: "https://pokeapi.co/api/v2/".into(),
        }
    }
}

impl PokeApi for PokeApiClient {
    fn get_description(&self, name: &str) -> Result<Option<String>> {
        #[derive(Deserialize)]
        struct Species {
            flavor_text_entries: Vec<FlavorText>,
        }

        #[derive(Deserialize)]
        struct FlavorText {
            flavor_text: String,
            language: NamedResource,
        }

        #[derive(Deserialize)]
        struct NamedResource {
            name: String,
        }

        let resp = reqwest::blocking::get(&format!("{}pokemon-species/{}", self.base_url, name))
            .context("Failed PokeAPI request")?;
        match resp.status() {
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            reqwest::StatusCode::OK => {
                let species: Species =
                    resp.json().context("PokeAPI responded with invalid JSON")?;
                match species
                    .flavor_text_entries
                    .into_iter()
                    .find(|e| e.language.name == "en")
                {
                    Some(e) => Ok(Some(e.flavor_text)),
                    None => {
                        warn!("Pokemon {} has no english flavor text available", name);
                        Ok(None)
                    }
                }
            }
            s => Err(anyhow!(
                "PokeAPI responded with {}: {}",
                s,
                resp.text()
                    .unwrap_or("<API responded with empty body or unformattable text>".into())
            )),
        }
    }
}

pub type BoxedTranslator = Box<dyn Translator + Send + Sync>;

pub trait Translator {
    fn translate(&self, source: &str) -> Result<String>;
}

impl<F> Translator for F
where
    F: Fn(&str) -> Result<String>,
{
    fn translate(&self, source: &str) -> Result<String> {
        self(source)
    }
}
