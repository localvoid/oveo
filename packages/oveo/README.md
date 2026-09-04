# oveo

Identity fallbacks for the [oveo](https://github.com/localvoid/oveo) JavaScript optimizer intrinsics (`hoist`, `scope`, `dedupe`, `key`).

When the optimizer plugin (`@oveo/rolldown`) is enabled, annotated calls are hoisted/deduped/renamed at build time. When it is disabled (e.g. `vite dev`), these are plain identity functions, so code keeps working.

> Prefer comment annotations (`/*@__HOIST__*/`, `/*@__SCOPE__*/`, `/*@__CONST__*/`) in new code — call intrinsics are deprecated and will be removed in the next major version.

See the [main README](https://github.com/localvoid/oveo#annotating-expressions) for annotation docs and [plugin options](https://github.com/localvoid/oveo#plugin-options).
