//! Implementation and abstractions for external services.
use anyhow::{anyhow, Context, Result};
use log::warn;
use serde::{Deserialize, Serialize};
pub type BoxedPokeApi = Box<dyn PokeApi + Send + Sync>;
use lru::LruCache;
use reqwest::header::HeaderMap;
use std::sync::Mutex;

use crate::api::Alpha;

/// Abstraction for Poke API access.
pub trait PokeApi {
    /// Fetches Pokemon descriptions given their name. Returns `Ok(None)` when either the pokemon
    /// doesn't exist or it has no english descriptions.
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

/// Poke API accessor. Use the `Default` implementation for the public API at https://pokeapi.co.
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

/// Translation service abstraction.
pub trait Translator {
    /// Translates the given source string to Shakespearean text.
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

/// Translation service using the Fun Translations API. Use the `Default` instance to use the
/// public API at https://api.funtranslations.com.
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

pub struct Cache(Mutex<LruCache<Alpha, Option<String>>>);

impl Cache {
    pub fn new(capacity: usize) -> Self {
        Cache(Mutex::new(LruCache::new(capacity)))
    }
}

impl Cache {
    pub fn get_or_calculate<F>(&self, k: Alpha, f: F) -> Result<Option<String>>
    where
        F: FnOnce() -> Result<Option<String>>,
    {
        let mut inner = self.0.lock().unwrap();
        if let Some(v) = inner.get(&k) {
            Ok(v.clone())
        } else {
            let v = f()?;
            inner.put(k, v.clone());
            Ok(v)
        }
    }
}
