# @oveo/optimizer

NAPI-RS bindings for the [oveo](https://github.com/localvoid/oveo) JavaScript optimizer (Rust core in `crates/oveo`, powered by [oxc](https://github.com/oxc-project/oxc/)).

Most app developers should use [`@oveo/rolldown`](https://github.com/localvoid/oveo#quick-setup) instead of this package directly. It exposes `Optimizer` with `transform()` (per-module) and `renderChunk()` (per-chunk) plus `importExterns` / `importPropertyMap` / `updatePropertyMap` helpers.

See the [main README](https://github.com/localvoid/oveo) for options, safety notes, and examples. Requires Node `>= 20`; prebuilt binaries ship per platform under `@oveo/optimizer-*`.
