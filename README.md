# Poké Shakespeare

Web API which translates Pokémon descriptions to Shakesperean text.

## Quickstart

In order to build the application you'll need the Rust toolchain. It's recommended to setup Rust using [rustup](https://rustup.rs/). Rust nightly is required:

```shellsession
poke_shakespeare$ rustup override set nightly
```

You can run a development server with `cargo run`, though by default it will not use the Fun Translations API since its rate limits are verystrict. You can override that by setting `funtranslations.mock = false` in the `Rocket.toml` config file. For more info on configuration, see [the Rocket reference](https://rocket.rs/v0.4/guide/configuration/#environment) and [config.rs](./src/config.rs).

## Docker

A Dockerfile is also provided, a config file can be used by mounting a volume at `/usr/srv/Rocket.toml`:

```shellsession
poke_shakespeare$ docker build -t poke_shakespeare .
poke_shakespeare$ docker run \
  -p8000:8000 \
  -v/path/to/my/Rocket.toml:/usr/srv/Rocket.toml \
  -eROCKET_ENV=production \
  poke_shakespeare
```
