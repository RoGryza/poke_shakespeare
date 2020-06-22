//! This module handles application-specific configuration in the Rocket.toml file. See
//! `ReadConfig`.
use std::collections::HashMap;

use log::error;
use rocket::config::ConfigError;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;
use serde::de::Deserializer;
use serde::Deserialize;

use crate::api::Alpha;
use crate::services::{BoxedPokeApi, BoxedTranslator, Cache, FunTranslationsApi, PokeApiClient};

/// Fairing which parses extra configuration on launch and instantiates the necessary services. The
/// following config keys are defined:
///
/// * cache_size(integer): Max translations to keep cached, defaults to 4096.
/// * pokeapi.mock(table): Mapping of pokemon names to descriptions. If specified, the application
/// references this table instead of fetching descriptions from PokeAPI.
/// * pokeapi.url(string): Pokemon species endpoint, defaults to
/// https://pokeapi.co/api/v2/pokemon-species/.
/// * funtranslations.mock(boolean): if true, the application will do mock translations instead of
/// accessing the Fun Translations API.
/// * funtranslations.url(string): Shakespeare translation endpoint, defaults to
/// https://api.funtranslations.com/translate/shakespeare/.
/// * funtranslations.api_key(string): Secret to authenticate the Fun Translations API with. If
/// unspecified, API calls will be unauthenticated. Note that unauthenticated calls are
/// rate-limited.
pub struct ReadConfig;

impl Fairing for ReadConfig {
    fn info(&self) -> Info {
        Info {
            name: "Read Config",
            kind: Kind::Attach,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let cfg = rocket.config();

        let cache_size = cfg.get_int("cache_size").unwrap_or(4096);
        if cache_size <= 0 {
            error!("Invalid cache size {}", cache_size);
            return Err(rocket);
        }
        let cache: Cache = Cache::new(cache_size as usize);

        let pokeapi = match cfg.get_extra("pokeapi").and_then(|v| {
            v.clone().try_into::<PokeApiConfig>().map_err(|e| {
                ConfigError::ParseError(
                    "".into(),
                    "Rocket.toml".into(),
                    e.to_string(),
                    e.line_col(),
                )
            })
        }) {
            Ok(cfg) => cfg.into_client(),
            Err(ConfigError::Missing(_)) => Box::new(PokeApiClient::default()),
            Err(e) => {
                error!("Configuration error: {}", e);
                return Err(rocket);
            }
        };

        let translator = match cfg.get_extra("funtranslations").and_then(|v| {
            v.clone().try_into::<TranslatorConfig>().map_err(|e| {
                ConfigError::ParseError(
                    "".into(),
                    "Rocket.toml".into(),
                    e.to_string(),
                    e.line_col(),
                )
            })
        }) {
            Ok(cfg) => cfg.into_translator(),
            Err(ConfigError::Missing(_)) => Box::new(FunTranslationsApi::default()),
            Err(e) => {
                error!("Configuration error: {}", e);
                return Err(rocket);
            }
        };

        Ok(rocket.manage(cache).manage(pokeapi).manage(translator))
    }
}

#[derive(Clone, Debug)]
pub enum PokeApiConfig {
    Mock(HashMap<String, String>),
    Concrete { url: Option<String> },
}

impl PokeApiConfig {
    pub fn into_client(self) -> BoxedPokeApi {
        match self {
            PokeApiConfig::Mock(map) => Box::new(move |s: &str| Ok(map.get(s).cloned())),
            PokeApiConfig::Concrete { url } => {
                let mut api = PokeApiClient::default();
                if let Some(u) = url {
                    api.url = u;
                }
                Box::new(api)
            }
        }
    }
}

impl<'de> Deserialize<'de> for PokeApiConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawConfig {
            #[serde(default)]
            mock: Option<HashMap<Alpha, String>>,
            #[serde(default)]
            url: Option<String>,
        }

        let raw = RawConfig::deserialize(deserializer)?;
        match raw.mock {
            Some(map) => Ok(PokeApiConfig::Mock(
                map.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            )),
            None => Ok(PokeApiConfig::Concrete { url: raw.url }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TranslatorConfig {
    Mock,
    Concrete {
        url: Option<String>,
        api_key: Option<String>,
    },
}

impl TranslatorConfig {
    pub fn into_translator(self) -> BoxedTranslator {
        match self {
            TranslatorConfig::Mock => Box::new(|s: &str| Ok(format!("MOCKED TRANSLATION: {}", s))),
            TranslatorConfig::Concrete { url, api_key } => {
                let mut api = FunTranslationsApi::default();
                api.api_key = api_key;
                if let Some(u) = url {
                    api.url = u;
                }
                Box::new(api)
            }
        }
    }
}

impl<'de> Deserialize<'de> for TranslatorConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawConfig {
            mock: bool,
            #[serde(default)]
            url: Option<String>,
            #[serde(default)]
            api_key: Option<String>,
        }

        let raw = RawConfig::deserialize(deserializer)?;
        if raw.mock {
            Ok(TranslatorConfig::Mock)
        } else {
            Ok(TranslatorConfig::Concrete {
                url: raw.url,
                api_key: raw.api_key,
            })
        }
    }
}
