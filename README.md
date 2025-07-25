[oveo](https://github.com/localvoid/oveo) is a javascript optimizer that works as a plugin for [Vite](https://vite.dev/) and [Rollup](https://rollupjs.org/). It is written in Rust and uses [oxc](https://github.com/oxc-project/oxc/) library for parsing and semantic analysis.

> **Use with caution!**
>
> Some optimizations are making assumptions that may break your code.

It is designed for bundlers that support hooks for transformations that work on [individual modules](https://rollupjs.org/plugin-development/#transform) and on [the final chunk or a bundle](https://rollupjs.org/plugin-development/#renderchunk).

## Vite Setup

- Add `@oveo/vite` package as a dev dependency to a project.
- Add oveo plugin to a Vite config:

```js
import { defineConfig } from "vite";
import { oveo } from "@oveo/vite";

export default defineConfig({
  plugins: [
    // By default, all optimizations are disabled.
    oveo({
      hoist: true,
      dedupe: true,
      globals: {
        include: ["js", "web"],
        hoist: true,
        singletons: true,
      },
      externs: {
        import: [/* */],
        inlineConstValues: true,
      },
      renameProperties: {
        pattern: "^[^_].+[^_]_$",
        map: "property-map",
      },
    }),
  ]
});
```

## Optimizations

- [Expression Hoisting](#expression-hoisting)
- [Expression Deduplication](#expression-deduplication)
- [Hoisting Globals](#hoisting-globals)
- [Singletons](#singletons)
- [Inline Extern Values](#inline-extern-values)
- [Rename Properties](#rename-properties)

### Expression Hoisting

This optimization works during module transformation phase and will try to hoist annotated expressions to the outermost valid scope.

To annotate an expression, it should be passed as an argument to the [intrinsic](#intrinsic-functions) function `hoist(expr)` or any function declared in the [externs](#externs) file.

By default, there is only one scope (program level scope). Scopes can be created with the [intrinsic](#intrinsic-functions) function `scope(() => {..})` or with a function declared in the [externs](#externs) file.

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
  },
}
```

In this [externs](#externs) example we are describing a module `@scope/modulename` that has two functions with an additional behavior: `myscope(() => {..})` and `myfunc(any, hoistable_expr)`. The first argument in the `myscope` function will behave as an expression that creates a new hoist scope. The second argument in the `myfunc` function will be hoisted to the outermost valid scope.

```js
import { myscope, myfunc } from "@scope/modulename";
import { x } from "./module.js";

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
import { myscope, myfunc } from "@scope/modulename";
import { x } from "./module.js";

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

- "Hoist Scope" - scope that can contain Hoisted Expressions. By default, there is only a program level scope. Additional scopes can be created with the intrinsic function `scope()`.
- "Hoisted Expression" - expression that should be hoisted to the outermost Hoist Scope.
- "Hoisted Expression Scope" - scopes created inside of a hoisted expression.
- "Inner Scope" - the closest Hoist Scope.
- "Outer Scopes" - scopes outside of the closest Hoist Scope.

```js
                       // outer scope (hoist scope - root scope)
{                      // outer scope
  scope((a) => {       // inner scope (hoist scope)
    return () => {     // inner scope function
      if (a) {         // conditional prevents hoisting
        hoist((i) => { // hoisted expr
                       // hoisted expr scope
          i();         // symbol from the hoisted expr scope
          a();         // symbol from the inner scope
        });
      }
    };
  })
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
  - `TaggedTemplateExpression` - ``tpl`text ${sym}` ``
- No conditionals on the path to the Hoist Scope:
  - `ConditionalExpression` - `cond ? then : else`
  - `IfStatement` - `if (cond) { .. } else { .. }`
  - `SwitchStatement` - `switch (v) { }`
- Expressions hoisted to the Inner Scope should be inside of a function scope.

To prevent an expression from hoisting, it should be wrapped in `ParenthesizedExpression`, e.g.:

```js
import { hoist } from "oveo";

const a = 1;
function test() {
  hoist((() => a));
}
```

### Expression Deduplication

This optimization works during chunk rendering phase and deduplicates expressions marked with the [intrinsic](#intrinsic-functions) function `dedupe(expr)` or when expression is [hoisted](#expression-hoisting).

- Deduped expressions shouldn't have any side effects.
- Deduped expressions doesn't provide referential equality (expressions from different chunks aren't deduplicated).

```js
import { dedupe } from "oveo";
import { externalIdentifier } from "./module.js";

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
import { externalIdentifier } from "./module.js";

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

This optimization works dunring chunk rendering phase and hoists global values and their static properties.

It hoists only predefined [globals](https://github.com/localvoid/oveo/blob/master/crates/oveo/src/globals.rs) with an assumption that they aren't mutated.

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

### Singletons

This optimization works during chunk rendering phase and deduplicates objects like `new TextEncoder()` with an assumption that there are no mutations to this objects and this objects will be referential equal when they are referenced in the chunk file.

Currently, there are only two singleton objects: `new TextEncoder()` and `new TextDecoder()`.

### Inline Extern Values

This optimization works during module transformation phase and inlines constant values declared in the [externs](#externs) file.

Inlining const values is useful in scenarios when shared constant values imported from different modules negatively affect a chunking algorithm, or when a program does a lot of string comparisons (class names in UI frameworks) and it would be more efficient to keep strings as [interned strings](https://en.wikipedia.org/wiki/String_interning).

```json
{
  "@scope/modulename": {
    "export": {
      "Button": {
        "type": "const",
        "value": "Button"
      },
      "ButtonPressed": {
        "type": "const",
        "value": "Button-pressed"
      },
      "any": {
        "type": "const",
        "value": {
          "key": "May contain any JSON values"
        }
      }
    }
  },
}
```

```jsx
import { Button, ButtonPressed, any } from "@scope/modulename";

function Button({ pressed }) {
  const cn = pressed
    ? `${Button} ${ButtonPressed}`
    : `${Button}`;
  return <button class={cn} />
}
console.log(any);
```

Will be transformed into:

```jsx
function Button({ pressed }) {
  const cn = pressed
    ? `${"Button"} ${"Button-pressed"}`
    : `${"Button"}`;
  return <button class={cn} />
}
console.log({ "key": "May contain any JSON values" });
```

And after minification (constant evaluation), class names will have an [interned](https://en.wikipedia.org/wiki/String_interning) type.

### Rename Properties

This optimization works during chunk transformation phase and renames property names that match properties from a predefined propery map.

Property map has a simple `key=value` format:

```ini
left_=a
right_=b
status_=c
```

Path to a property map file is specified in the oveo plugin options:

```js
import { oveo } from "@oveo/vite";

export default {
  input: "src/main.js",
  output: {
    file: "bundle.js",
  },
  plugins: [
    oveo({
      renameProperties: {
        pattern: "^[^_].+[^_]_$",
        map: "property-map",
      },
    }),
  ]
};
```

Some minifiers support a similar optimization:

- [Terser - Mangle Properties Options](https://terser.org/docs/options/#mangle-properties-options)
- [esbuild - Mangle props](https://esbuild.github.io/api/#mangle-props)

## Intrinsic Functions

When optimizer is disabled, intrinsic functions will work as an identity function `<T>(expr: T) => expr`.

#### `hoist(expr)`

Hoists expression to the outermost valid [hoisting scope](#scope----).

#### `scope(() => { .. })`

Creates a new hoisting scope.

#### `dedupe(expr)`

Deduplicates expressions.

## Externs

Extern files are specified in the oveo plugin options:

```js
import { oveo } from "@oveo/vite";

export default {
  input: "src/main.js",
  output: {
    file: "bundle.js",
  },
  plugins: [
    oveo({
      externs: {
        import: [
          "ivi/oveo.json", // Distributed in the 'ivi' package
          "./my-custom-extern.json",
        ],
      },
    }),
  ]
};
```

Extern file example:

```json
{
  "@scope/modulename": {
    "exports": {
      "constValue": {
        "type": "const",
        "value": { "key": "any JSON value" }
      },
      "fnWithHoistableArg": {
        "type": "function",
        "arguments": [{ "hoist": true }]
      },
      "fnWithHoistScopeArg": {
        "type": "function",
        "arguments": [{ "scope": true }]
      },
      "customNamespace": {
        "type": "namespace",
        "exports": {
          "any": { "type": "const", "value": 123 }
        }
      }
    }
  }
}
```
