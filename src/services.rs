use std::collections::HashMap;
use std::iter::FromIterator;

pub type Error = ();
pub type Result<T> = std::result::Result<T, Error>;

pub type BoxedPokeStore = Box<dyn PokeStore + Send + Sync>;

pub trait PokeStore {
    fn get_description(&self, name: &str) -> Result<String>;
}

pub type BoxedTranslator = Box<dyn Translator + Send + Sync>;

pub trait Translator {
    fn translate(&self, source: &str) -> Result<String>;
}

#[derive(Clone, Default)]
pub struct DummyPokeStore(HashMap<String, String>);

impl DummyPokeStore {
    pub fn new(map: HashMap<String, String>) -> Self {
        map.into()
    }
}

impl FromIterator<(String, String)> for DummyPokeStore {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, String)>,
    {
        DummyPokeStore(iter.into_iter().collect())
    }
}

impl From<HashMap<String, String>> for DummyPokeStore {
    fn from(map: HashMap<String, String>) -> Self {
        DummyPokeStore(map)
    }
}

impl PokeStore for DummyPokeStore {
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
