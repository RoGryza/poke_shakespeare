use anyhow::Result;

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
