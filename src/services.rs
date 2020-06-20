use std::collections::HashMap;
use std::iter::FromIterator;

pub type Error = ();
pub type Result<T> = std::result::Result<T, Error>;

pub type BoxedPokeApi = Box<dyn PokeApi + Send + Sync>;

pub trait PokeApi {
    fn get_description(&self, name: &str) -> Result<String>;
}

pub type BoxedTranslator = Box<dyn Translator + Send + Sync>;

pub trait Translator {
    fn translate(&self, source: &str) -> Result<String>;
}

#[derive(Clone, Default)]
pub struct DummyPokeApi(HashMap<String, String>);

impl DummyPokeApi {
    pub fn new(map: HashMap<String, String>) -> Self {
        map.into()
    }
}

impl FromIterator<(String, String)> for DummyPokeApi {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, String)>,
    {
        DummyPokeApi(iter.into_iter().collect())
    }
}

impl From<HashMap<String, String>> for DummyPokeApi {
    fn from(map: HashMap<String, String>) -> Self {
        DummyPokeApi(map)
    }
}

impl PokeApi for DummyPokeApi {
    fn get_description(&self, name: &str) -> Result<String> {
        match self.0.get(name) {
            Some(s) => Ok(s.into()),
            None => Err(()),
        }
    }
}

#[derive(Clone, Default)]
pub struct DummyTranslator;

impl DummyTranslator {
    pub fn new() -> Self {
        DummyTranslator::default()
    }
}

impl Translator for DummyTranslator {
    fn translate(&self, source: &str) -> Result<String> {
        Ok(format!("TRANSLATED: {}", source))
    }
}
