use poke_shakespeare_lib::api;
use poke_shakespeare_lib::services::{
    BoxedPokeApi, BoxedTranslator, DummyPokeApi, DummyTranslator,
};

fn main() {
    rocket::ignite()
        .manage(BoxedPokeApi::from(Box::new(
            vec![
                ("a".to_string(), "b".to_string()),
                ("c".to_string(), "d".to_string()),
            ]
            .into_iter()
            .collect::<DummyPokeApi>(),
        )))
        .manage(BoxedTranslator::from(Box::new(DummyTranslator::new())))
        .mount("/", api())
        .launch();
}
