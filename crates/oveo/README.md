# oveo (Rust core)

Rust core of the [oveo](https://github.com/localvoid/oveo) JavaScript optimizer, powered by [oxc](https://github.com/oxc-project/oxc/).

Exposes `optimize_module()` (hoisting) and `optimize_chunk()` (dedupe, globals, singletons, rename-properties, URLs). Usually consumed via the `@oveo/optimizer` NAPI bindings and the `@oveo/rolldown` plugin — see the [main README](https://github.com/localvoid/oveo) for user docs.

Develop with `cargo test` / `cargo clippy`; global tables live in `src/globals.rs`.
