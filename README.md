# Some Bot
This repository provides a scaffold for QQ bots in Rust.

Thanks [ricq](https://github.com/lz1998/ricq) for the amazing work.

## Getting Started
### Usage
`UIN=$UIN PASSWORD=$PASSWORD cargo run .`

### Register Modules
1. Implement module struct in `src/handler/$NAME.rs`. Structs should implement `Module` trait.
2. Import modules in `src/handler/mod.rs`.
3. Register modules in `src/main.rs` with `register_module` chaining functions.

