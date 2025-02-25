# Rust10x MangoDB Web App Blueprint for Production Coding

Discord of Jeremy Chone: https://discord.gg/XuKWrNGKpC

## This is work in Progress and not production ready.

## Needs a local MangoDB instance without auth. If yor want the test to work you first need to place data in the DB.

## Known issues:
- No Tenants DB Layer
- No Role Features
- Objects in DB can be similar besides the ObjectID
- Error handling is not really done but good enough for dev.
- Response to clients wrong cause of point 4.


## Dev (watch)

> NOTE: Install cargo watch with `cargo install cargo-watch`.

```sh
# Terminal 1 - To run the server.

cargo watch -q -c -w crates/services/web-server/src/ -w crates/libs/ -w .cargo/ -x "run -p web-server"
# Terminal 2 - To run the quick_dev.
cargo watch -q -c -w crates/services/web-server/examples/ -x "run -p web-server --example quick_dev"
```

## Dev

```sh
# Terminal 1 - To run the server.
cargo run -p web-server

# Terminal 2 - To run the tests.
cargo run -p web-server --example quick_dev
```

## Tools

```sh
cargo run -p gen-key
```

<br />

---

More resources for [Rust for Production Coding](https://rust10x.com)
