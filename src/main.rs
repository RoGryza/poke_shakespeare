use poke_shakespeare_lib::poke_shakespeare;
use poke_shakespeare_lib::services::{FunTranslationsApi, PokeApiClient};

fn main() {
    poke_shakespeare(PokeApiClient::default(), FunTranslationsApi::default()).launch();
}
