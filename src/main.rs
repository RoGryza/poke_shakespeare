use poke_shakespeare_lib::poke_shakespeare;
use poke_shakespeare_lib::services::PokeApiClient;

fn main() {
    poke_shakespeare(PokeApiClient::default(), |s: &str| Ok(s.to_string())).launch();
}
