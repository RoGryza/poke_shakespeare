use poke_shakespeare_lib::poke_shakespeare;

fn main() {
    poke_shakespeare(|_: &str| Ok(None), |s: &str| Ok(s.to_string())).launch();
}
