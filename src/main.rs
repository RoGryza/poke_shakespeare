#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use poke_shakespeare_lib::api;
use poke_shakespeare_lib::services::{
    BoxedPokeStore, BoxedTranslator, DummyPokeStore, DummyTranslator,
};

fn main() {
    rocket::ignite()
        .manage(BoxedPokeStore::from(Box::new(
            vec![
                ("a".to_string(), "b".to_string()),
                ("c".to_string(), "d".to_string()),
            ]
            .into_iter()
            .collect::<DummyPokeStore>(),
        )))
        .manage(BoxedTranslator::from(Box::new(DummyTranslator::new())))
        .mount("/", api())
        .launch();
}
