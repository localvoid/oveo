# oveo

[oveo](https://github.com/localvoid/oveo) is a JavaScript optimizer that works as a plugin for [Vite](https://vite.dev/) and [Rolldown](https://rolldown.rs/). It is written in Rust and uses the [oxc](https://github.com/oxc-project/oxc/) library for parsing and semantic analysis.

It shrinks and speeds up production bundles by hoisting/deduplicating repeated expressions, hoisting global lookups (`Array.isArray` → cached local), deduplicating singletons (`new TextEncoder()`), shortening property names, and rewriting `new URL(..., import.meta.url)` asset references into absolute URLs.

> **Use with caution!**
>
> Some optimizations make assumptions that may break your code (see [Caveats and safety](#caveats-and-safety) and the assumptions listed under each optimization). All optimizations are **disabled by default** — enable them one at a time and validate your build output.

## Contents

- [When to use oveo](#when-to-use-oveo)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick setup](#quick-setup)
  - [Vite](#vite)
  - [Rolldown](#rolldown)
- [Plugin options](#plugin-options)
- [How it works](#how-it-works)
- [Optimizations](#optimizations)
  - [Expression Hoisting](#expression-hoisting)
  - [Expression Deduplication](#expression-deduplication)
  - [Hoisting Globals](#hoisting-globals)
  - [Singletons](#singletons)
  - [Rename Properties](#rename-properties)
  - [Absolute URLs](#absolute-urls)
- [Annotating expressions](#annotating-expressions)
- [Externs](#externs)
- [Caveats and safety](#caveats-and-safety)
- [Troubleshooting](#troubleshooting)
- [Development](#development)
- [License](#license)

## When to use oveo

Use oveo when you ship a Vite/Rolldown production build and want smaller/faster output beyond minification:

- Repeated inline callbacks/objects/templates created inside components or hot functions.
- Hot paths calling `Array.isArray`, `Object.hasOwn`, `console.*`, `fetch`, etc.
- `new TextEncoder()` / `new TextDecoder()` scattered across chunks.
- Libraries like [ivi](https://github.com/localvoid/ivi) that already emit hoist/dedupe annotations.
- Projects that can enforce a property-renaming convention (e.g. trailing `_`) and asset `base`.

Skip or be extra careful if you mutate globals (`Array.isArray = ...`), rely on `new TextEncoder() !== new TextEncoder()`, rely on object identity across chunks for deduped values, or cannot validate renamed properties end-to-end.

## Requirements

- Node.js `>= 20`
- Vite (build mode only) or Rolldown
- Modules processed by the `transform` step: `js`, `jsx`, `ts`, `tsx` (configurable via `filter`, see [Plugin options](#plugin-options))

## Installation

```sh
npm install --save-dev @oveo/rolldown
# pnpm add -D @oveo/rolldown
# yarn add -D @oveo/rolldown
# bun add -d @oveo/rolldown
```

If your code (or a library like `ivi`) uses intrinsic calls such as `hoist()` / `dedupe()`, also install the identity fallback package (no-op at runtime, real behavior comes from the optimizer):

```sh
npm install oveo
```

## Quick setup

### Vite

```js
import { defineConfig } from 'vite';
import { oveo } from '@oveo/rolldown';

export default defineConfig({
  plugins: [
    // By default, all optimizations are disabled.
    oveo({
      dedupe: true,
      globals: true,
      url: true, // auto-detect base from Vite `base`
    }),
  ],
});
```

Full example with every feature turned on:

```js
import { defineConfig } from 'vite';
import { oveo } from '@oveo/rolldown';

export default defineConfig({
  base: '/assets/',
  plugins: [
    oveo({
      hoist: true,
      dedupe: true,
      // `globals: true` is a shorthand for everything enabled:
      // {
      //   include: ['js', 'console', 'web', 'electron', 'tauri'],
      //   hoist: true,
      //   singletons: true,
      // }
      globals: true,
      externs: {
        import: ['./my-custom-extern.json'],
      },
      renameProperties: {
        pattern: '^[^_].+[^_]_$',
        map: 'property-map',
      },
      url: {
        baseURL: '/assets/', // must end with '/'; or `url: true` to auto-detect from Vite `base`
      },
    }),
  ],
});
```

### Rolldown

```js
import { oveo } from '@oveo/rolldown';

export default {
  input: 'src/main.js',
  output: {
    file: 'bundle.js',
  },
  plugins: [
    oveo({
      dedupe: true,
      globals: {
        include: ['js', 'web'],
        hoist: true,
        singletons: true,
      },
      url: {
        baseURL: '/assets/',
      },
    }),
  ],
};
```

See [`examples/vite`](examples/vite) for a runnable Vite setup (`vite.config.mjs`, `externs.json`, `properties.ini`).

## Plugin options

All fields are optional and default to disabled.

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| `hoist` | `boolean` | `false` | Enable [Expression Hoisting](#expression-hoisting) in `transform`. |
| `dedupe` | `boolean` | `false` | Enable [Expression Deduplication](#expression-deduplication) in `renderChunk`. Hoisted expressions are deduped automatically. |
| `globals` | `boolean \| { include?, hoist?, singletons? }` | disabled | `true` = all namespaces + hoist + singletons. `include` is a subset of `['js', 'console', 'web', 'electron', 'tauri']`. See [Hoisting Globals](#hoisting-globals) and [Singletons](#singletons). |
| `externs.import` | `string[]` | `[]` | Paths/ids of [extern files](#externs) to load in `buildStart` (watched via `addWatchFile`). |
| `externs.inlineConstValues` | `boolean` | `false` | Inline `{"type": "const", "value": ...}` exports from extern files. |
| `renameProperties.pattern` | `string` | unset | RegExp source for property names to rename. When set, new matches are appended to the map in `writeBundle`. See [Rename Properties](#rename-properties). |
| `renameProperties.map` | `string` | unset | Path to `key=value` property map file. Loaded in `buildStart`, updated in `writeBundle` when `pattern` is set. Watched via `addWatchFile`. |
| `url` | `boolean \| { baseURL? }` | disabled | `false`/omitted = off. `true` or `{}` = auto-detect from Vite `base`. `{ baseURL }` = explicit base (non-empty, must end with `/`). See [Absolute URLs](#absolute-urls). |
| `filter` | Rolldown `HookFilter` | `{ moduleType: ['js', 'jsx', 'ts', 'tsx'] }` | Which modules go through `transform`. |

TypeScript shape (from `@oveo/rolldown` / `@oveo/optimizer`):

```ts
interface PluginOptions {
  hoist?: boolean;
  dedupe?: boolean;
  globals?:
    | boolean
    | {
        include?: Array<'js' | 'console' | 'web' | 'electron' | 'tauri'>;
        hoist?: boolean;
        singletons?: boolean;
      };
  externs?: {
    inlineConstValues?: boolean;
    import?: string[];
  };
  renameProperties?: {
    pattern?: string;
    map?: string;
  };
  url?: boolean | { baseURL?: string };
  filter?: HookFilter;
}
```

## How it works

oveo is designed for bundlers with per-module and per-chunk hooks ([`transform`](https://rollupjs.org/plugin-development/#transform) and [`renderChunk`](https://rollupjs.org/plugin-development/#renderchunk)).

| Optimization | Phase | Needs annotations? |
| --- | --- | --- |
| [Expression Hoisting](#expression-hoisting) | `transform` (module) | Yes (`hoist`/`scope` or comments/externs) |
| [Expression Deduplication](#expression-deduplication) | `renderChunk` (chunk) | Yes (`dedupe`/`@__CONST__`), hoisted exprs included automatically |
| [Hoisting Globals](#hoisting-globals) | `renderChunk` (chunk) | No (namespace allowlist) |
| [Singletons](#singletons) | `renderChunk` (chunk) | No (`TextEncoder`/`TextDecoder` only) |
| [Rename Properties](#rename-properties) | `transform` + `writeBundle` | No (pattern + map) |
| [Absolute URLs](#absolute-urls) | `renderChunk` (chunk) | No (`new URL(..., import.meta.url)` patterns) |

Sourcemaps are preserved (`transform`/`renderChunk` return `{ code, map }`). Transform failures report `Unable to transform module '<id>'`; chunk failures report `Unable to optimize chunk file`.

## Optimizations

- [Expression Hoisting](#expression-hoisting)
- [Expression Deduplication](#expression-deduplication)
- [Hoisting Globals](#hoisting-globals)
- [Singletons](#singletons)
- [Rename Properties](#rename-properties)
- [Absolute URLs](#absolute-urls)

### Expression Hoisting

Works during module transformation. Tries to hoist annotated expressions to the outermost valid scope.

Annotate with comment `/*@__HOIST__*/expr` (preferred) or intrinsic `hoist(expr)` (see [Annotating expressions](#annotating-expressions)):

```js
function test() {
  const x = /*@__HOIST__*/ (c) => a;
  return x;
}
```

Comment annotations are matched by substring (block or line comments in leading position, e.g. `/* note @__HOIST__ */` also works). Annotation comments are removed from the output.

By default there is only one scope (program-level scope). Create scopes with `/*@__SCOPE__*/(() => {..})` (or `scope(() => {..})`), or with a function declared in the [externs](#externs) file.

```json
{
  "@scope/modulename": {
    "exports": {
      "myscope": {
        "arguments": [{ "scope": true }]
      },
      "myfunc": {
        "arguments": [{}, { "hoist": true }]
      }
    }
  }
}
```

In this [externs](#externs) example we describe module `@scope/modulename` with two functions: `myscope(() => {..})` and `myfunc(any, hoistable_expr)`. The first argument of `myscope` behaves as an expression that creates a new hoist scope. The second argument of `myfunc` is hoisted to the outermost valid scope.

```js
import { myscope, myfunc } from '@scope/modulename';
import { x } from './module.js';

const fn = myscope((inner_0) => {
  myfunc(1, (inner_1) => {
    x(inner_1);
    myfunc(2, () => {
      x(inner_0);
    });
    myfunc(3, (inner_3) => {
      x(inner_3);
    });
  });
});
```

Will be transformed into:

```js
import { myscope, myfunc } from '@scope/modulename';
import { x } from './module.js';

const _HOIST_3 = (inner_3) => {
  x(inner_3);
};
const fn = myscope((inner_0) => {
  const _HOIST_2 = () => {
    x(inner_0);
  };
  myfunc(1, (inner_1) => {
    x(inner_1);
    myfunc(2, _HOIST_2);
    myfunc(3, _HOIST_3);
  });
});
```

#### Real World Usage Example

```js
import { component, getProps, html } from "ivi";
import { type Action, dispatch, select } from "./actions.js";

const Button = component((c) => {
  return ({ text }) => html`
    <button @click=${() => { dispatch(c, select(getProps(c).entry)); }}}>
      ${text}
    </button>
  `;
});
```

In the example above, `component(() => {})` behaves as a hoisting scope (declared in the externs file) and [ivi](https://github.com/localvoid/ivi) template compiler annotates event handlers as hoistable expressions.

After template compilation and oveo optimizations the generated code will look like:

```js
import { component, getProps, _T, _t } from "ivi";
import { type Action, dispatch, select } from "./actions.js";

const _TPL_ = _T(/* template strings and opcodes */);
const Button = component((c) => {
  const _HOISTED_ = () => { dispatch(c, select(getProps(c).entry)); };
  return ({ text }) => _t(_TPL_, [_HOISTED_, text]);
});
```

#### Hoisting Heuristics

Terminology:

- "Hoist Scope" - scope that can contain Hoisted Expressions. By default, there is only a program level scope. Additional scopes can be created with `/*@__SCOPE__*/`.
- "Hoisted Expression" - expression that should be hoisted to the outermost Hoist Scope.
- "Hoisted Expression Scope" - scopes created inside of a hoisted expression.
- "Inner Scope" - the closest Hoist Scope.
- "Outer Scopes" - scopes outside of the closest Hoist Scope.

```js
// outer scope (hoist scope - root scope)
{
  // outer scope
  scope((a) => {
    // inner scope (hoist scope)
    return () => {
      // inner scope function
      if (a) {
        // conditional prevents hoisting
        hoist((i) => {
          // hoisted expr
          // hoisted expr scope
          i(); // symbol from the hoisted expr scope
          a(); // symbol from the inner scope
        });
      }
    };
  });
}
```

Hoisting heuristics are quite conservative:

- All symbols should be accessible from the Hoist Scope.
- Hoisted expression should have a type:
  - `ArrowFunctionExpression` - `() => {}`
  - `FunctionExpression` - `function () {}`
  - `CallExpression` - `fn()`
  - `NewExpression` - `new C()`
  - `ObjectExpression` - `{ key: value }`
  - `ArrayExpression` - `[a, b, c]`
  - `TemplateLiteral` - `` `text ${sym}` ``
  - `TaggedTemplateExpression` - ``tpl`text ${sym}` ``
- No conditionals on the path to the Hoist Scope:
  - `ConditionalExpression` - `cond ? then : else`
  - `IfStatement` - `if (cond) { .. } else { .. }`
  - `SwitchStatement` - `switch (v) { }`
- Expressions hoisted to the Inner Scope should be inside of a function scope.

### Expression Deduplication

Works during chunk rendering. Deduplicates expressions marked with `/*@__CONST__*/expr` (or `dedupe(expr)`), or expressions already [hoisted](#expression-hoisting).

- Deduped expressions shouldn't have any side effects.
- Deduped expressions don't provide referential equality across chunks (dedup is chunk-local).

```js
import { dedupe } from 'oveo';
import { externalIdentifier } from './module.js';

const obj1 = dedupe({
  global: Number,
  identifier: externalIdentifier,
  array: [1, 2, 3],
  literal: 1,
});
function Scope1() {
  const obj2 = dedupe({
    global: Number,
    identifier: externalIdentifier,
    array: [1, 2, 3],
    literal: 1,
  });
  const scoped1 = dedupe({ array: [1, 2, 3] });
}
function Scope2() {
  const scoped2 = dedupe({ array: [1, 2, 3] });
}
const arr1 = dedupe([1, 2, 3]);
```

Will be transformed into:

```js
import { externalIdentifier } from './module.js';

const _DEDUPE_ = [1, 2, 3];
const obj1 = {
  global: Number,
  identifier: externalIdentifier,
  array: _DEDUPE_,
  literal: 1,
};
function Scope1() {
  const obj2 = obj1;
  const scoped1 = { array: _DEDUPE_ };
}
function Scope2() {
  const scoped2 = { array: _DEDUPE_ };
}
const arr1 = _DEDUPE_;
```

### Hoisting Globals

Works during chunk rendering. Hoists global values and their static properties.

It hoists only predefined [globals](crates/oveo/src/globals.rs) with an assumption that they aren't mutated.

```js
function isArray(data) {
  if (Array.isArray(data)) {
    // ...
  }
}
function from(data) {
  if (Array.from(data)) {
    // ...
  }
}
```

Will be transformed into:

```js
const _GLOBAL_1 = Array;
const _GLOBAL_2 = _GLOBAL_1.isArray;
const _GLOBAL_3 = _GLOBAL_1.from;
function isArray(data) {
  if (_GLOBAL_2(data)) {
    // ...
  }
}
function from(data) {
  if (_GLOBAL_3(data)) {
    // ...
  }
}
```

Configure with `globals: true` (all of `['js', 'console', 'web', 'electron', 'tauri']` + `hoist` + `singletons`) or granularly:

```js
oveo({
  globals: {
    include: ['js', 'web'],
    hoist: true,
    singletons: true,
  },
});
```

### Singletons

Works during chunk rendering. Deduplicates objects like `new TextEncoder()` with an assumption that there are no mutations to these objects and that these objects are referentially equal when referenced in the chunk file.

Currently only two singletons: `new TextEncoder()` and `new TextDecoder()`. Enabled via `globals: true` or `globals: { singletons: true }`.

### Rename Properties

Works during chunk transformation. Renames property names matching a RegExp pattern or listed in a property map.

When bundler finishes building all chunks, it adds new properties matching the RegExp pattern to the property map.

Property map has a simple `key=value` format:

```ini
left_=a
right_=b
status_=c
```

Path to the property map file is specified in the oveo plugin options:

```js
import { oveo } from '@oveo/rolldown';

export default {
  input: 'src/main.js',
  output: {
    file: 'bundle.js',
  },
  plugins: [
    oveo({
      renameProperties: {
        pattern: '^[^_].+[^_]_$',
        map: 'property-map',
      },
    }),
  ],
};
```

Workflow:

1. First build with `pattern` + `map`: matching properties are renamed and the map file is created/updated in `writeBundle`.
2. Commit the map file. Subsequent builds reuse stable short names.
3. To rename-only (no new names), set `map` without `pattern`.

To mark a single string literal as a property name, use `key('prop_')` (see [Annotating expressions](#annotating-expressions)).

Some minifiers support a similar optimization:

- [Terser - Mangle Properties Options](https://terser.org/docs/options/#mangle-properties-options)
- [esbuild - Mangle props](https://esbuild.github.io/api/#mangle-props)
- [oxc](https://oxc.rs/docs/guide/usage/minifier/mangling.html)

Since oxc recently added support for property mangling, this optimization will be removed in the future versions.

### Absolute URLs

By default, when Rollup and Rolldown generate URLs to different assets, they generate relative URLs like `new URL("./asset", import.meta.url).href`.

This optimization rewrites relative URLs into absolute URLs, e.g.:

```js
function test() {
  return new URL('./relative.css', import.meta.url).href;
}
```

Will be transformed into:

```js
function test() {
  return '/base-url/relative.css';
}
```

- Rollup supports [`resolveFileUrl`](https://rollupjs.org/plugin-development/#resolvefileurl) hook that can be used instead of this optimization.
- Rolldown currently doesn't support `resolveFileUrl` hook: [issue#1010](https://github.com/rolldown/rolldown/issues/1010).

Supported patterns (first argument must be a string literal or a substitution-free template literal, second argument must be `import.meta.url`):

- `new URL('./asset', import.meta.url).href`
- `new URL('./asset', import.meta.url).pathname`
- `new URL('./asset', import.meta.url)["href"]` / `["pathname"]`
- `new URL('./asset', import.meta.url).toString()` (zero arguments, also `["toString"]()`)

Skipped (left untouched): absolute URLs (`https:…`, `data:…`), root-absolute (`/…`), query/hash-only (`?…`, `#…`), empty strings, and shadowed (non-global) `URL` constructors. `baseURL` must be non-empty and end with `'/'`. In plugin options, `url: true` auto-detects the base from Vite `base`.

## Annotating expressions

| Goal | Comment (preferred) | Intrinsic (from `oveo`) |
| --- | --- | --- |
| Hoist to outer [hoisting scope](#expression-hoisting) | `/*@__HOIST__*/expr` | `hoist(expr)` |
| Create hoisting scope | `/*@__SCOPE__*/(() => {..})` | `scope(() => {..})` |
| Deduplicate | `/*@__CONST__*/expr` | `dedupe(expr)` |
| Rename string as property | — | `key(string_literal)` |

When the optimizer is disabled, intrinsic functions work as identity functions `<T>(expr: T) => expr`.

#### `hoist(expr)`

Hoists expression to the outermost valid [hoisting scope](#expression-hoisting).

#### `scope(() => { .. })`

Creates a new hoisting scope.

#### `dedupe(expr)`

Deduplicates expressions.

#### `key(string_literal)`

Renames string literal as a property name.

## Externs

Extern files describe third-party modules so oveo can treat their functions as hoist/scope annotations, or inline constants — without changing that library's source. Paths are specified in plugin options:

```js
import { oveo } from '@oveo/rolldown';

export default {
  input: 'src/main.js',
  output: {
    file: 'bundle.js',
  },
  plugins: [
    oveo({
      externs: {
        import: [
          'ivi/oveo.json', // Distributed in the 'ivi' package
          './my-custom-extern.json',
        ],
      },
    }),
  ],
};
```

Extern file example:

```json
{
  "@scope/modulename": {
    "exports": {
      "fnWithHoistableArg": {
        "type": "function",
        "arguments": [{ "hoist": true }]
      },
      "fnWithHoistScopeArg": {
        "type": "function",
        "arguments": [{ "scope": true }]
      }
    }
  }
}
```

Supported export descriptors:

- `{ "type": "function", "arguments": [{ "hoist"?: true, "scope"?: true }] }` — per-argument hoist/scope behavior.
- `{ "type": "const", "value": ... }` — inline constant (requires `externs: { inlineConstValues: true }`). Example in [`examples/vite/externs.json`](examples/vite/externs.json).

## Caveats and safety

- **Globals:** assumes globals and their static properties are never reassigned/monkey-patched. Don't enable `globals.hoist` if you (or a dependency) mutate `Array`, `Object`, `console`, `window`, etc. Limit `include` to namespaces you control (e.g. `['js']`).
- **Singletons:** assumes `new TextEncoder()` / `new TextDecoder()` instances are never mutated and can share identity within a chunk.
- **Dedupe:** only for side-effect-free expressions; equality is chunk-local, not cross-chunk.
- **Hoist:** heuristic is conservative (see [Hoisting Heuristics](#expression-hoisting)); symbols must be reachable from the hoist scope, no conditionals on the hoist path. Opt out with extra parens (`hoist((() => a))` stays put).
- **Rename properties:** irreversible across builds without the map file — commit `renameProperties.map`, review new entries, and avoid overly broad `pattern` regexes.
- **URLs:** only the patterns listed in [Absolute URLs](#absolute-urls) are rewritten; `base` / `baseURL` (must end with `/`). Relative `./` Vite `base` disables auto-detection with a warning.

## Troubleshooting

- `oveo: url.baseURL must end with '/'` — append trailing slash or use `url: true` with a proper Vite `base` (not `'./'` or `''`).
- ``oveo: ignoring unsupported Vite `base` ...`` — auto-detection disabled; set `url: { baseURL: '/...' }` explicitly.
- `Unable to read property map file` — normal on first build when `pattern` is set and the file doesn't exist yet; it will be created in `writeBundle`. Without `pattern`, ensure the path exists.
- `Invalid property map file` / `Unable to import extern file` — check `key=value` format and that extern paths resolve (they go through Rolldown `resolve` + `addWatchFile`).
- Nothing happens — remember all optimizations default to off; set at least `dedupe: true` or `globals: true`. Also check `filter` covers your `moduleType` and that Vite runs in build mode (`apply: 'build'` means `vite dev` is untouched).

## Development

```sh
bun install
bun run napi-build   # napi build
bun run build        # tsc -b
bun run test         # bun test ./tests/
bun run format       # oxfmt
bun run check        # oxlint
```

NAPI binding lives in `packages/@oveo/optimizer` (`bun run napi-build`). JS plugin lives in `packages/@oveo/rolldown/src/index.ts`. Intrinsics live in `packages/oveo`.

## License

MIT — see [LICENSE](LICENSE).
