use std::borrow::Cow;

use rustc_hash::FxHashMap;

#[derive(Default)]
pub struct Globals {
    pub index: GlobalValue,
}

impl Globals {
    pub fn get(&self, name: &str) -> Option<&GlobalValue> {
        match name {
            "window" | "globalThis" => Some(&self.index),
            _ => self.index.statics.get(name),
        }
    }

    fn add<T: Build<Output = GlobalValue>>(&mut self, name: &'static str, value: T) {
        self.index.statics.insert(name.into(), value.build());
    }
}

#[derive(Default, Clone)]
pub struct GlobalValue {
    pub statics: FxHashMap<Cow<'static, str>, GlobalValue>,
    pub hoist: bool,
    pub kind: GlobalValueKind,
}

#[derive(Default, Clone)]
pub enum GlobalValueKind {
    #[default]
    Object,
    Func(GlobalFunction),
    Const(serde_json::Value),
}

impl GlobalValue {
    pub fn is_hoistable(&self) -> bool {
        self.hoist
    }

    pub fn is_singleton_func(&self) -> bool {
        if let GlobalValueKind::Func(f) = &self.kind {
            return f.singleton;
        }
        false
    }
}

#[derive(Clone)]
pub struct GlobalFunction {
    pub arguments: Vec<GlobalFunctionArgument>,
    /// Hash Arguments for singletons?
    pub singleton: bool,
    pub hoist: bool,
}

#[derive(Clone)]
pub struct GlobalFunctionArgument {
    pub hoist: bool,
}

struct GlobalObjectBuilder {
    statics: FxHashMap<Cow<'static, str>, GlobalValue>,
    hoist: bool,
    kind: GlobalValueKind,
}

trait Build {
    type Output;

    fn build(self) -> Self::Output;
}

impl GlobalObjectBuilder {
    fn with_static<T: Build<Output = GlobalValue>>(mut self, name: &'static str, value: T) -> Self {
        self.statics.insert(name.into(), value.build());
        self
    }

    fn with_func<T: Build<Output = GlobalFunction>>(mut self, func: T) -> Self {
        self.kind = GlobalValueKind::Func(func.build());
        self
    }
}

impl Build for GlobalObjectBuilder {
    type Output = GlobalValue;

    fn build(self) -> Self::Output {
        GlobalValue { statics: self.statics, kind: self.kind, hoist: self.hoist }
    }
}

fn object() -> GlobalObjectBuilder {
    GlobalObjectBuilder {
        statics: FxHashMap::default(),
        kind: GlobalValueKind::Object,
        hoist: true,
    }
}

struct GlobalFunctionBuilder {
    pub arguments: Vec<GlobalFunctionArgument>,
    pub singleton: bool,
    pub hoist: bool,
}

impl GlobalFunctionBuilder {
    fn singleton(mut self) -> Self {
        self.singleton = true;
        self
    }
}

impl Build for GlobalFunctionBuilder {
    type Output = GlobalFunction;

    fn build(self) -> Self::Output {
        GlobalFunction { arguments: self.arguments, singleton: self.singleton, hoist: self.hoist }
    }
}

fn func() -> GlobalFunctionBuilder {
    GlobalFunctionBuilder { arguments: Vec::default(), singleton: false, hoist: true }
}

pub fn add_default_globals(g: &mut Globals) {
    g.add("AggregateError", object());
    g.add(
        "Array",
        object()
            .with_static("from", object())
            .with_static("fromAsync", object())
            .with_static("isArray", object())
            .with_static("of", object()),
    );
    g.add("ArrayBuffer", object().with_static("isView", object()));
    g.add("AsyncDisposableStack", object());
    g.add("AsyncFunction", object());
    g.add("AsyncGenerator", object());
    g.add("AsyncGeneratorFunction", object());
    g.add("AsyncIterator", object());
    g.add(
        "Atomics",
        object()
            .with_static("add", object())
            .with_static("and", object())
            .with_static("compareExchange", object())
            .with_static("exchange", object())
            .with_static("isLockFree", object())
            .with_static("load", object())
            .with_static("notify", object())
            .with_static("or", object())
            .with_static("pause", object())
            .with_static("store", object())
            .with_static("sub", object())
            .with_static("wait", object())
            .with_static("waitAsync", object())
            .with_static("xor", object()),
    );
    g.add("BigInt", object().with_static("asIntN", object()).with_static("asUintN", object()));
    g.add(
        "BigInt64Array",
        object()
            .with_static("now", object())
            .with_static("parse", object())
            .with_static("UTC", object()),
    );
    g.add("BigUint64Array", object());
    g.add("Boolean", object());
    g.add("DataView", object());
    g.add("Date", object());
    g.add("DisposableStack", object());
    g.add(
        "Error",
        object().with_static("captureStackTrace", object()).with_static("isError", object()),
    );
    g.add("FinalizationRegistry", object());
    g.add("Float16Array", object());
    g.add("Float32Array", object());
    g.add("Float64Array", object());
    g.add("Function", object());
    g.add("Generator", object());
    g.add("GeneratorFunction", object());
    g.add("Infinity", object());
    g.add("Int8Array", object());
    g.add("Int16Array", object());
    g.add("Int32Array", object());
    g.add(
        "Intl",
        object()
            .with_static("getCanonicalLocales", object())
            .with_static("supportedValuesOf", object()),
    );
    g.add("Iterator", object().with_static("from", object()));
    g.add(
        "JSON",
        object()
            .with_static("isRawJSON", object())
            .with_static("parse", object())
            .with_static("rawJSON", object())
            .with_static("stringify", object()),
    );
    g.add("Map", object().with_static("groupBy", object()));
    g.add(
        "Math",
        object()
            .with_static("abs", object())
            .with_static("acos", object())
            .with_static("acosh", object())
            .with_static("asin", object())
            .with_static("asinh", object())
            .with_static("atan", object())
            .with_static("atan2", object())
            .with_static("atanh", object())
            .with_static("cbrt", object())
            .with_static("ceil", object())
            .with_static("clz32", object())
            .with_static("cos", object())
            .with_static("cosh", object())
            .with_static("exp", object())
            .with_static("expm1", object())
            .with_static("f16round", object())
            .with_static("floor", object())
            .with_static("fround", object())
            .with_static("hypot", object())
            .with_static("imul", object())
            .with_static("log", object())
            .with_static("log1p", object())
            .with_static("log2", object())
            .with_static("log10", object())
            .with_static("max", object())
            .with_static("min", object())
            .with_static("pow", object())
            .with_static("random", object())
            .with_static("round", object())
            .with_static("sign", object())
            .with_static("sin", object())
            .with_static("sinh", object())
            .with_static("sqrt", object())
            .with_static("sumPrecise", object())
            .with_static("tan", object())
            .with_static("tanh", object())
            .with_static("trunc", object()),
    );
    g.add("NaN", object());
    g.add(
        "Number",
        object()
            .with_static("isFinite", object())
            .with_static("isInteger", object())
            .with_static("isNaN", object())
            .with_static("isSafeInteger", object())
            .with_static("parseFloat", object())
            .with_static("parseInt", object())
            .with_static("EPSILON", object())
            .with_static("MAX_SAFE_INTEGER", object())
            .with_static("MAX_VALUE", object())
            .with_static("MIN_SAFE_INTEGER", object())
            .with_static("MIN_VALUE", object())
            .with_static("NaN", object())
            .with_static("NEGATIVE_INFINITY", object())
            .with_static("POSITIVE_INFINITY", object()),
    );
    g.add(
        "Object",
        object()
            .with_static("prototype", object())
            .with_static("assign", object())
            .with_static("create", object())
            .with_static("defineProperties", object())
            .with_static("defineProperty", object())
            .with_static("entries", object())
            .with_static("freeze", object())
            .with_static("fromEntries", object())
            .with_static("getOwnPropertyDescriptor", object())
            .with_static("getOwnPropertyDescriptors", object())
            .with_static("getOwnPropertyNames", object())
            .with_static("getOwnPropertySymbols", object())
            .with_static("getPrototypeOf", object())
            .with_static("groupBy", object())
            .with_static("hasOwn", object())
            .with_static("is", object())
            .with_static("isExtensible", object())
            .with_static("isFrozen", object())
            .with_static("isSealed", object())
            .with_static("keys", object())
            .with_static("preventExtensions", object())
            .with_static("seal", object())
            .with_static("setPrototypeOf", object())
            .with_static("values", object()),
    );
    g.add(
        "Promise",
        object()
            .with_static("all", object())
            .with_static("allSettled", object())
            .with_static("any", object())
            .with_static("race", object())
            .with_static("reject", object())
            .with_static("resolve", object())
            .with_static("try", object())
            .with_static("withResolvers", object()),
    );
    g.add("Proxy", object());
    g.add("RangeError", object());
    g.add("ReferenceError", object());
    g.add(
        "Reflect",
        object()
            .with_static("apply", object())
            .with_static("construct", object())
            .with_static("defineProperty", object())
            .with_static("deleteProperty", object())
            .with_static("get", object())
            .with_static("getOwnPropertyDescriptor", object())
            .with_static("getPrototypeOf", object())
            .with_static("has", object())
            .with_static("isExtensible", object())
            .with_static("ownKeys", object())
            .with_static("preventExtensions", object())
            .with_static("set", object())
            .with_static("setPrototypeOf", object()),
    );
    g.add("RegExp", object().with_static("escape", object()));
    g.add("Set", object());
    g.add("SharedArrayBuffer", object());
    g.add(
        "String",
        object()
            .with_static("fromCharCode", object())
            .with_static("fromCodePoint", object())
            .with_static("raw", object()),
    );
    g.add(
        "Symbol",
        object()
            .with_static("asyncDispose", object())
            .with_static("dispose", object())
            .with_static("for", object())
            .with_static("keyFor", object())
            .with_static("asyncIterator", object())
            .with_static("hasInstance", object())
            .with_static("isConcatSpreadable", object())
            .with_static("iterator", object())
            .with_static("match", object())
            .with_static("matchAll", object())
            .with_static("replace", object())
            .with_static("search", object())
            .with_static("species", object())
            .with_static("split", object())
            .with_static("toPrimitive", object())
            .with_static("toStringTag", object())
            .with_static("unscopables", object()),
    );
    g.add("SyntaxError", object());
    g.add("TextDecoder", object().with_func(func().singleton()));
    g.add("TextEncoder", object().with_func(func().singleton()));
    g.add(
        "TypedArray",
        object()
            .with_static("from", object())
            .with_static("of", object())
            .with_static("BYTES_PER_ELEMENT", object()),
    );
    g.add("TypeError", object());
    g.add("Uint8Array", object());
    g.add("Uint8ClampedArray", object());
    g.add("Uint16Array", object());
    g.add("Uint32Array", object());
    g.add("URIError", object());
    g.add("URLPattern", object());
    g.add("WeakMap", object());
    g.add("WeakRef", object());
    g.add("WeakSet", object());
    g.add("decodeURI", object());
    g.add("decodeURIComponent", object());
    g.add("encodeURI", object());
    g.add("encodeURIComponent", object());
    g.add("isFinite", object());
    g.add("isNaN", object());
    g.add("parseFloat", object());
    g.add("parseInt", object());
    g.add("undefined", object());
    g.add("document", object());
    g.add("caches", object());
    g.add("cookieStore", object());
    g.add("crossOriginIsolated", object());
    g.add("crypto", object());
    g.add("customElements", object());
    g.add("frameElement", object());
    g.add("history", object());
    g.add("indexedDB", object());
    g.add("isSecureContext", object());
    g.add("localStorage", object());
    g.add("navigator", object());
    g.add("scheduler", object());
    g.add("sessionStorage", object());
    g.add("speechSynthesis", object());
    g.add("trustedTypes", object());
    g.add("setTimeout", object());
    g.add("clearTimeout", object());
    g.add("performance", object());
    g.add("queueMicrotask", object());
    g.add("requestAnimationFrame", object());
}
