use anyhow::{anyhow, Context, Result};

use log::warn;
use serde::{Deserialize, Serialize};
pub type BoxedPokeApi = Box<dyn PokeApi + Send + Sync>;
use reqwest::header::HeaderMap;

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
    pub url: String,
}

impl Default for PokeApiClient {
    fn default() -> Self {
        PokeApiClient {
            url: "https://pokeapi.co/api/v2/pokemon-species/".into(),
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

        let resp = reqwest::blocking::get(&format!("{}{}", self.url, name))
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

pub struct FunTranslationsApi {
    pub url: String,
    pub api_key: Option<String>,
    client: reqwest::blocking::Client,
}

impl Default for FunTranslationsApi {
    fn default() -> Self {
        FunTranslationsApi {
            url: "https://api.funtranslations.com/translate/shakespeare".into(),
            api_key: None,
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Translator for FunTranslationsApi {
    fn translate<'s>(&self, source: &'s str) -> Result<String> {
        #[derive(Serialize)]
        struct Request<'s> {
            text: &'s str,
        }

        #[derive(Deserialize)]
        struct Response {
            contents: Contents,
        }

        #[derive(Deserialize)]
        struct Contents {
            translated: String,
        }

        let mut headers = HeaderMap::new();
        if let Some(ref api_key) = self.api_key {
            headers.insert(
                "X-FunTranslations-Api-Secret",
                api_key.parse().expect("Invalid Fun Translations API key"),
            );
        }
        let resp = self
            .client
            .post(&self.url)
            .headers(headers)
            .form(&Request { text: source })
            .send()
            .context("Failed Fun Translations request")?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                let data: Response = resp
                    .json()
                    .context("Fun Translations responded with invalid JSON")?;
                Ok(data.contents.translated)
            }
            s => Err(anyhow!(
                "Fun Translations responded with {}: {}",
                s,
                resp.text()
                    .unwrap_or("<API responded with empty body or unformattable text>".into())
            )),
        }
    }
}
