# Litentry Node
[![Actions Status](https://github.com/litentry/litentry-node/workflows/Rust/badge.svg)](https://github.com/litentry/litentry-node/actions)


Litentry node built with Substrate.

## Build from source code.


## Set up

```
  rustup update
  rustup target add wasm32-unknown-unknown --toolchain nightly
  rustup default nightly
  cargo clean && cargo build
```


## Debug

If you want to use your local copy of pallets implementation instead of the snapshots from github, you can configure it in your `.cargo/config` file.

```
  mkdir -p .cargo
  mv cargoconfig.example .cargo/config
  vi .cargo/config
```


## License
Apache-2.0
