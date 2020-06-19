use poke_shakespeare_lib::api;

fn main() {
    rocket::ignite().mount("/", api()).launch();
}
