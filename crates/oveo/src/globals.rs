// A lot of globals in the Web API are still missing.
// If you missing some API, submit an issue or pull request.
use std::sync::LazyLock;

use rustc_hash::FxHashMap;

static GLOBALS: LazyLock<GlobalValue> = LazyLock::new(|| {
    let mut statics = FxHashMap::default();
    add_globals_js(&mut statics);
    add_globals_console(&mut statics);
    add_globals_web(&mut statics);
    add_globals_web_typed_css(&mut statics);
    add_globals_web_background_fetch(&mut statics);
    add_globals_web_barcode(&mut statics);
    add_globals_web_battery(&mut statics);
    add_globals_web_bluetooth(&mut statics);
    add_globals_web_sync(&mut statics);
    add_globals_web_paint(&mut statics);

    GlobalValue {
        statics,
        category: GlobalCategory::ALL,
        hoist: true,
        kind: GlobalValueKind::Object,
    }
});

#[derive(Default, Clone, Copy, Debug)]
pub struct GlobalCategory(u32);

impl GlobalCategory {
    pub const ALL: Self = Self(!0);
    pub const JS: Self = Self(1 << 0);
    pub const CONSOLE: Self = Self(1 << 1);
    pub const WEB: Self = Self(1 << 2);
    pub const WEB_TYPED_CSS: Self = Self(1 << 3);
    pub const WEB_BACKGROUND_FETCH: Self = Self(1 << 4);
    pub const WEB_BARCODE: Self = Self(1 << 5);
    pub const WEB_BATTERY: Self = Self(1 << 6);
    pub const WEB_BLUETOOTH: Self = Self(1 << 7);
    pub const WEB_SYNC: Self = Self(1 << 8);
    pub const WEB_PAINT: Self = Self(1 << 9);
    pub const UNKNOWN: Self = Self(1 << 10);

    #[inline]
    pub fn matches(self, rhs: GlobalCategory) -> bool {
        self.0 & rhs.0 != 0
    }

    #[inline]
    pub fn and(self, rhs: GlobalCategory) -> GlobalCategory {
        Self(self.0 | rhs.0)
    }
}

impl<S: AsRef<str>, T: Iterator<Item = S>> From<T> for GlobalCategory {
    fn from(value: T) -> Self {
        let mut c = GlobalCategory::default();
        for i in value {
            match i.as_ref() {
                "js" => c = c.and(Self::JS),
                "console" => c = c.and(Self::CONSOLE),
                "web" => c = c.and(Self::WEB),
                "web:typed-css" => c = c.and(Self::WEB_TYPED_CSS),
                "web:background-fetch" => c = c.and(Self::WEB_BACKGROUND_FETCH),
                "web:barcode" => c = c.and(Self::WEB_BARCODE),
                "web:battery" => c = c.and(Self::WEB_BATTERY),
                "web:bluetooth" => c = c.and(Self::WEB_BLUETOOTH),
                "web:sync" => c = c.and(Self::WEB_SYNC),
                "web:paint" => c = c.and(Self::WEB_PAINT),
                _ => c = c.and(Self::UNKNOWN),
            }
        }
        c
    }
}

pub fn get_global_value(categories: GlobalCategory, name: &str) -> Option<&'static GlobalValue> {
    match name {
        "window" | "globalThis" => Some(&GLOBALS),
        _ => GLOBALS.statics.get(name).filter(|v| v.category.matches(categories)),
    }
}

#[derive(Default, Clone)]
pub struct GlobalValue {
    pub statics: FxHashMap<&'static str, GlobalValue>,
    pub category: GlobalCategory,
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
    pub singleton: bool,
    pub hoist: bool,
}

#[derive(Clone)]
pub struct GlobalFunctionArgument {
    pub hoist: bool,
}

struct GlobalObjectBuilder {
    statics: FxHashMap<&'static str, GlobalValue>,
    category: GlobalCategory,
    hoist: bool,
    kind: GlobalValueKind,
}

trait Build {
    type Output;

    fn build(self) -> Self::Output;
}

impl GlobalObjectBuilder {
    fn with_static<T: Build<Output = GlobalValue>>(mut self, name: &'static str, value: T) -> Self {
        self.statics.insert(name, value.build());
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
        GlobalValue {
            statics: self.statics,
            category: self.category,
            kind: self.kind,
            hoist: self.hoist,
        }
    }
}

fn object(category: GlobalCategory) -> GlobalObjectBuilder {
    GlobalObjectBuilder {
        statics: FxHashMap::default(),
        category,
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

fn add<T: Build<Output = GlobalValue>>(
    g: &mut FxHashMap<&'static str, GlobalValue>,
    name: &'static str,
    value: T,
) {
    g.insert(name, value.build());
}

fn add_globals_js(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(g, "AggregateError", object(GlobalCategory::JS));
    add(
        g,
        "Array",
        object(GlobalCategory::JS)
            .with_static("from", object(GlobalCategory::JS))
            .with_static("fromAsync", object(GlobalCategory::JS))
            .with_static("isArray", object(GlobalCategory::JS))
            .with_static("of", object(GlobalCategory::JS)),
    );
    add(
        g,
        "ArrayBuffer",
        object(GlobalCategory::JS).with_static("isView", object(GlobalCategory::JS)),
    );
    add(g, "AsyncDisposableStack", object(GlobalCategory::JS));
    add(g, "AsyncFunction", object(GlobalCategory::JS));
    add(g, "AsyncGenerator", object(GlobalCategory::JS));
    add(g, "AsyncGeneratorFunction", object(GlobalCategory::JS));
    add(g, "AsyncIterator", object(GlobalCategory::JS));
    add(
        g,
        "Atomics",
        object(GlobalCategory::JS)
            .with_static("add", object(GlobalCategory::JS))
            .with_static("and", object(GlobalCategory::JS))
            .with_static("compareExchange", object(GlobalCategory::JS))
            .with_static("exchange", object(GlobalCategory::JS))
            .with_static("isLockFree", object(GlobalCategory::JS))
            .with_static("load", object(GlobalCategory::JS))
            .with_static("notify", object(GlobalCategory::JS))
            .with_static("or", object(GlobalCategory::JS))
            .with_static("pause", object(GlobalCategory::JS))
            .with_static("store", object(GlobalCategory::JS))
            .with_static("sub", object(GlobalCategory::JS))
            .with_static("wait", object(GlobalCategory::JS))
            .with_static("waitAsync", object(GlobalCategory::JS))
            .with_static("xor", object(GlobalCategory::JS)),
    );
    add(
        g,
        "BigInt",
        object(GlobalCategory::JS)
            .with_static("asIntN", object(GlobalCategory::JS))
            .with_static("asUintN", object(GlobalCategory::JS)),
    );
    add(
        g,
        "BigInt64Array",
        object(GlobalCategory::JS)
            .with_static("now", object(GlobalCategory::JS))
            .with_static("parse", object(GlobalCategory::JS))
            .with_static("UTC", object(GlobalCategory::JS)),
    );
    add(g, "BigUint64Array", object(GlobalCategory::JS));
    add(g, "Boolean", object(GlobalCategory::JS));
    add(g, "DataView", object(GlobalCategory::JS));
    add(g, "Date", object(GlobalCategory::JS));
    add(g, "DisposableStack", object(GlobalCategory::JS));
    add(
        g,
        "Error",
        object(GlobalCategory::JS)
            .with_static("captureStackTrace", object(GlobalCategory::JS))
            .with_static("isError", object(GlobalCategory::JS)),
    );
    add(g, "FinalizationRegistry", object(GlobalCategory::JS));
    add(g, "Float16Array", object(GlobalCategory::JS));
    add(g, "Float32Array", object(GlobalCategory::JS));
    add(g, "Float64Array", object(GlobalCategory::JS));
    add(g, "Function", object(GlobalCategory::JS));
    add(g, "Generator", object(GlobalCategory::JS));
    add(g, "GeneratorFunction", object(GlobalCategory::JS));
    add(g, "Infinity", object(GlobalCategory::JS));
    add(g, "Int8Array", object(GlobalCategory::JS));
    add(g, "Int16Array", object(GlobalCategory::JS));
    add(g, "Int32Array", object(GlobalCategory::JS));
    add(
        g,
        "Intl",
        object(GlobalCategory::JS)
            .with_static("getCanonicalLocales", object(GlobalCategory::JS))
            .with_static("supportedValuesOf", object(GlobalCategory::JS)),
    );
    add(g, "Iterator", object(GlobalCategory::JS).with_static("from", object(GlobalCategory::JS)));
    add(
        g,
        "JSON",
        object(GlobalCategory::JS)
            .with_static("isRawJSON", object(GlobalCategory::JS))
            .with_static("parse", object(GlobalCategory::JS))
            .with_static("rawJSON", object(GlobalCategory::JS))
            .with_static("stringify", object(GlobalCategory::JS)),
    );
    add(g, "Map", object(GlobalCategory::JS).with_static("groupBy", object(GlobalCategory::JS)));
    add(
        g,
        "Math",
        object(GlobalCategory::JS)
            .with_static("abs", object(GlobalCategory::JS))
            .with_static("acos", object(GlobalCategory::JS))
            .with_static("acosh", object(GlobalCategory::JS))
            .with_static("asin", object(GlobalCategory::JS))
            .with_static("asinh", object(GlobalCategory::JS))
            .with_static("atan", object(GlobalCategory::JS))
            .with_static("atan2", object(GlobalCategory::JS))
            .with_static("atanh", object(GlobalCategory::JS))
            .with_static("cbrt", object(GlobalCategory::JS))
            .with_static("ceil", object(GlobalCategory::JS))
            .with_static("clz32", object(GlobalCategory::JS))
            .with_static("cos", object(GlobalCategory::JS))
            .with_static("cosh", object(GlobalCategory::JS))
            .with_static("exp", object(GlobalCategory::JS))
            .with_static("expm1", object(GlobalCategory::JS))
            .with_static("f16round", object(GlobalCategory::JS))
            .with_static("floor", object(GlobalCategory::JS))
            .with_static("fround", object(GlobalCategory::JS))
            .with_static("hypot", object(GlobalCategory::JS))
            .with_static("imul", object(GlobalCategory::JS))
            .with_static("log", object(GlobalCategory::JS))
            .with_static("log1p", object(GlobalCategory::JS))
            .with_static("log2", object(GlobalCategory::JS))
            .with_static("log10", object(GlobalCategory::JS))
            .with_static("max", object(GlobalCategory::JS))
            .with_static("min", object(GlobalCategory::JS))
            .with_static("pow", object(GlobalCategory::JS))
            .with_static("random", object(GlobalCategory::JS))
            .with_static("round", object(GlobalCategory::JS))
            .with_static("sign", object(GlobalCategory::JS))
            .with_static("sin", object(GlobalCategory::JS))
            .with_static("sinh", object(GlobalCategory::JS))
            .with_static("sqrt", object(GlobalCategory::JS))
            .with_static("sumPrecise", object(GlobalCategory::JS))
            .with_static("tan", object(GlobalCategory::JS))
            .with_static("tanh", object(GlobalCategory::JS))
            .with_static("trunc", object(GlobalCategory::JS))
            // Constants
            .with_static("E", object(GlobalCategory::JS))
            .with_static("LN2", object(GlobalCategory::JS))
            .with_static("LN10", object(GlobalCategory::JS))
            .with_static("LOG2E", object(GlobalCategory::JS))
            .with_static("LOG10E", object(GlobalCategory::JS))
            .with_static("PI", object(GlobalCategory::JS))
            .with_static("SQRT1_2", object(GlobalCategory::JS))
            .with_static("SQRT2", object(GlobalCategory::JS)),
    );
    add(g, "NaN", object(GlobalCategory::JS));
    add(
        g,
        "Number",
        object(GlobalCategory::JS)
            .with_static("isFinite", object(GlobalCategory::JS))
            .with_static("isInteger", object(GlobalCategory::JS))
            .with_static("isNaN", object(GlobalCategory::JS))
            .with_static("isSafeInteger", object(GlobalCategory::JS))
            .with_static("parseFloat", object(GlobalCategory::JS))
            .with_static("parseInt", object(GlobalCategory::JS))
            // Constants
            .with_static("EPSILON", object(GlobalCategory::JS))
            .with_static("MAX_SAFE_INTEGER", object(GlobalCategory::JS))
            .with_static("MAX_VALUE", object(GlobalCategory::JS))
            .with_static("MIN_SAFE_INTEGER", object(GlobalCategory::JS))
            .with_static("MIN_VALUE", object(GlobalCategory::JS))
            .with_static("NaN", object(GlobalCategory::JS))
            .with_static("NEGATIVE_INFINITY", object(GlobalCategory::JS))
            .with_static("POSITIVE_INFINITY", object(GlobalCategory::JS)),
    );
    add(
        g,
        "Object",
        object(GlobalCategory::JS)
            .with_static(
                "prototype",
                object(GlobalCategory::JS)
                    .with_static("hasOwnProperty", object(GlobalCategory::JS))
                    .with_static("isPrototypeOf", object(GlobalCategory::JS))
                    .with_static("propertyIsEnumerable", object(GlobalCategory::JS)),
            )
            .with_static("assign", object(GlobalCategory::JS))
            .with_static("create", object(GlobalCategory::JS))
            .with_static("defineProperties", object(GlobalCategory::JS))
            .with_static("defineProperty", object(GlobalCategory::JS))
            .with_static("entries", object(GlobalCategory::JS))
            .with_static("freeze", object(GlobalCategory::JS))
            .with_static("fromEntries", object(GlobalCategory::JS))
            .with_static("getOwnPropertyDescriptor", object(GlobalCategory::JS))
            .with_static("getOwnPropertyDescriptors", object(GlobalCategory::JS))
            .with_static("getOwnPropertyNames", object(GlobalCategory::JS))
            .with_static("getOwnPropertySymbols", object(GlobalCategory::JS))
            .with_static("getPrototypeOf", object(GlobalCategory::JS))
            .with_static("groupBy", object(GlobalCategory::JS))
            .with_static("hasOwn", object(GlobalCategory::JS))
            .with_static("is", object(GlobalCategory::JS))
            .with_static("isExtensible", object(GlobalCategory::JS))
            .with_static("isFrozen", object(GlobalCategory::JS))
            .with_static("isSealed", object(GlobalCategory::JS))
            .with_static("keys", object(GlobalCategory::JS))
            .with_static("preventExtensions", object(GlobalCategory::JS))
            .with_static("seal", object(GlobalCategory::JS))
            .with_static("setPrototypeOf", object(GlobalCategory::JS))
            .with_static("values", object(GlobalCategory::JS)),
    );
    add(
        g,
        "Promise",
        object(GlobalCategory::JS)
            .with_static("all", object(GlobalCategory::JS))
            .with_static("allSettled", object(GlobalCategory::JS))
            .with_static("any", object(GlobalCategory::JS))
            .with_static("race", object(GlobalCategory::JS))
            .with_static("reject", object(GlobalCategory::JS))
            .with_static("resolve", object(GlobalCategory::JS))
            .with_static("try", object(GlobalCategory::JS))
            .with_static("withResolvers", object(GlobalCategory::JS)),
    );
    add(g, "Proxy", object(GlobalCategory::JS));
    add(g, "RangeError", object(GlobalCategory::JS));
    add(g, "ReferenceError", object(GlobalCategory::JS));
    add(
        g,
        "Reflect",
        object(GlobalCategory::JS)
            .with_static("apply", object(GlobalCategory::JS))
            .with_static("construct", object(GlobalCategory::JS))
            .with_static("defineProperty", object(GlobalCategory::JS))
            .with_static("deleteProperty", object(GlobalCategory::JS))
            .with_static("get", object(GlobalCategory::JS))
            .with_static("getOwnPropertyDescriptor", object(GlobalCategory::JS))
            .with_static("getPrototypeOf", object(GlobalCategory::JS))
            .with_static("has", object(GlobalCategory::JS))
            .with_static("isExtensible", object(GlobalCategory::JS))
            .with_static("ownKeys", object(GlobalCategory::JS))
            .with_static("preventExtensions", object(GlobalCategory::JS))
            .with_static("set", object(GlobalCategory::JS))
            .with_static("setPrototypeOf", object(GlobalCategory::JS)),
    );
    add(g, "RegExp", object(GlobalCategory::JS).with_static("escape", object(GlobalCategory::JS)));
    add(g, "Set", object(GlobalCategory::JS));
    add(g, "SharedArrayBuffer", object(GlobalCategory::JS));
    add(
        g,
        "String",
        object(GlobalCategory::JS)
            .with_static("fromCharCode", object(GlobalCategory::JS))
            .with_static("fromCodePoint", object(GlobalCategory::JS))
            .with_static("raw", object(GlobalCategory::JS)),
    );
    add(
        g,
        "Symbol",
        object(GlobalCategory::JS)
            .with_static("asyncDispose", object(GlobalCategory::JS))
            .with_static("dispose", object(GlobalCategory::JS))
            .with_static("for", object(GlobalCategory::JS))
            .with_static("keyFor", object(GlobalCategory::JS))
            .with_static("asyncIterator", object(GlobalCategory::JS))
            .with_static("hasInstance", object(GlobalCategory::JS))
            .with_static("isConcatSpreadable", object(GlobalCategory::JS))
            .with_static("iterator", object(GlobalCategory::JS))
            .with_static("match", object(GlobalCategory::JS))
            .with_static("matchAll", object(GlobalCategory::JS))
            .with_static("replace", object(GlobalCategory::JS))
            .with_static("search", object(GlobalCategory::JS))
            .with_static("species", object(GlobalCategory::JS))
            .with_static("split", object(GlobalCategory::JS))
            .with_static("toPrimitive", object(GlobalCategory::JS))
            .with_static("toStringTag", object(GlobalCategory::JS))
            .with_static("unscopables", object(GlobalCategory::JS)),
    );
    add(g, "SyntaxError", object(GlobalCategory::JS));
    add(g, "TextDecoder", object(GlobalCategory::JS).with_func(func().singleton()));
    add(g, "TextEncoder", object(GlobalCategory::JS).with_func(func().singleton()));
    add(
        g,
        "TypedArray",
        object(GlobalCategory::JS)
            .with_static("from", object(GlobalCategory::JS))
            .with_static("of", object(GlobalCategory::JS))
            .with_static("BYTES_PER_ELEMENT", object(GlobalCategory::JS)),
    );
    add(g, "TypeError", object(GlobalCategory::JS));
    add(g, "Uint8Array", object(GlobalCategory::JS));
    add(g, "Uint8ClampedArray", object(GlobalCategory::JS));
    add(g, "Uint16Array", object(GlobalCategory::JS));
    add(g, "Uint32Array", object(GlobalCategory::JS));
    add(g, "URIError", object(GlobalCategory::JS));
    add(g, "URLPattern", object(GlobalCategory::JS));
    add(g, "WeakMap", object(GlobalCategory::JS));
    add(g, "WeakRef", object(GlobalCategory::JS));
    add(g, "WeakSet", object(GlobalCategory::JS));
    add(g, "decodeURI", object(GlobalCategory::JS));
    add(g, "decodeURIComponent", object(GlobalCategory::JS));
    add(g, "encodeURI", object(GlobalCategory::JS));
    add(g, "encodeURIComponent", object(GlobalCategory::JS));
    add(g, "isFinite", object(GlobalCategory::JS));
    add(g, "isNaN", object(GlobalCategory::JS));
    add(g, "parseFloat", object(GlobalCategory::JS));
    add(g, "parseInt", object(GlobalCategory::JS));
    add(g, "undefined", object(GlobalCategory::JS));
}

fn add_globals_console(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(
        g,
        "console",
        object(GlobalCategory::CONSOLE)
            .with_static("assert", object(GlobalCategory::CONSOLE))
            .with_static("clear", object(GlobalCategory::CONSOLE))
            .with_static("countReset", object(GlobalCategory::CONSOLE))
            .with_static("count", object(GlobalCategory::CONSOLE))
            .with_static("debug", object(GlobalCategory::CONSOLE))
            .with_static("dir", object(GlobalCategory::CONSOLE))
            .with_static("dirxml", object(GlobalCategory::CONSOLE))
            .with_static("error", object(GlobalCategory::CONSOLE))
            .with_static("groupCollapsed", object(GlobalCategory::CONSOLE))
            .with_static("groupEnd", object(GlobalCategory::CONSOLE))
            .with_static("group", object(GlobalCategory::CONSOLE))
            .with_static("info", object(GlobalCategory::CONSOLE))
            .with_static("log", object(GlobalCategory::CONSOLE))
            .with_static("profileEnd", object(GlobalCategory::CONSOLE))
            .with_static("profile", object(GlobalCategory::CONSOLE))
            .with_static("table", object(GlobalCategory::CONSOLE))
            .with_static("timeEnd", object(GlobalCategory::CONSOLE))
            .with_static("timeLog", object(GlobalCategory::CONSOLE))
            .with_static("timeStamp", object(GlobalCategory::CONSOLE))
            .with_static("time", object(GlobalCategory::CONSOLE))
            .with_static("trace", object(GlobalCategory::CONSOLE))
            .with_static("warn", object(GlobalCategory::CONSOLE)),
    );
}

fn add_globals_web(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(g, "Navigator", object(GlobalCategory::WEB));
    add(g, "Window", object(GlobalCategory::WEB));
    add(g, "Document", object(GlobalCategory::WEB));
    add(g, "URLSearchParams", object(GlobalCategory::WEB));
    add(g, "Range", object(GlobalCategory::WEB));

    add(g, "Element", object(GlobalCategory::WEB));
    add(g, "HTMLDocument", object(GlobalCategory::WEB));
    add(g, "HTMLCollection", object(GlobalCategory::WEB));
    add(g, "HTMLFormControlsCollection", object(GlobalCategory::WEB));
    add(g, "HTMLOptionsCollection", object(GlobalCategory::WEB));
    add(g, "HTMLElement", object(GlobalCategory::WEB));
    add(g, "HTMLAreaElement", object(GlobalCategory::WEB));
    add(g, "HTMLAnchorElement", object(GlobalCategory::WEB));
    add(g, "HTMLAudioElement", object(GlobalCategory::WEB));
    add(g, "HTMLBaseElement", object(GlobalCategory::WEB));
    add(g, "HTMLBodyElement", object(GlobalCategory::WEB));
    add(g, "HTMLBRElement", object(GlobalCategory::WEB));
    add(g, "HTMLButtonElement", object(GlobalCategory::WEB));
    add(g, "HTMLCanvasElement", object(GlobalCategory::WEB));
    add(g, "HTMLDataElement", object(GlobalCategory::WEB));
    add(g, "HTMLDataListElement", object(GlobalCategory::WEB));
    add(g, "HTMLDetailsElement", object(GlobalCategory::WEB));
    add(g, "HTMLDialogElement", object(GlobalCategory::WEB));
    add(g, "HTMLDivElement", object(GlobalCategory::WEB));
    add(g, "HTMLDListElement", object(GlobalCategory::WEB));
    add(g, "HTMLEmbedElement", object(GlobalCategory::WEB));
    add(g, "HTMLFencedFrameElement", object(GlobalCategory::WEB));
    add(g, "HTMLFieldSetElement", object(GlobalCategory::WEB));
    add(g, "HTMLFormElement", object(GlobalCategory::WEB));
    add(g, "HTMLHeadElement", object(GlobalCategory::WEB));
    add(g, "HTMLHeadingElement", object(GlobalCategory::WEB));
    add(g, "HTMLHRElement", object(GlobalCategory::WEB));
    add(g, "HTMLHtmlElement", object(GlobalCategory::WEB));
    add(g, "HTMLIFrameElement", object(GlobalCategory::WEB));
    add(g, "HTMLImageElement", object(GlobalCategory::WEB));
    add(g, "HTMLInputElement", object(GlobalCategory::WEB));
    add(g, "HTMLLabelElement", object(GlobalCategory::WEB));
    add(g, "HTMLLegendElement", object(GlobalCategory::WEB));
    add(g, "HTMLLIElement", object(GlobalCategory::WEB));
    add(g, "HTMLLinkElement", object(GlobalCategory::WEB));
    add(g, "HTMLMapElement", object(GlobalCategory::WEB));
    add(g, "HTMLMediaElement", object(GlobalCategory::WEB));
    add(g, "HTMLMenuElement", object(GlobalCategory::WEB));
    add(g, "HTMLMetaElement", object(GlobalCategory::WEB));
    add(g, "HTMLMeterElement", object(GlobalCategory::WEB));
    add(g, "HTMLModElement", object(GlobalCategory::WEB));
    add(g, "HTMLObjectElement", object(GlobalCategory::WEB));
    add(g, "HTMLOListElement", object(GlobalCategory::WEB));
    add(g, "HTMLOptGroupElement", object(GlobalCategory::WEB));
    add(g, "HTMLOptionElement", object(GlobalCategory::WEB));
    add(g, "HTMLOutputElement", object(GlobalCategory::WEB));
    add(g, "HTMLParagraphElement", object(GlobalCategory::WEB));
    add(g, "HTMLPictureElement", object(GlobalCategory::WEB));
    add(g, "HTMLPreElement", object(GlobalCategory::WEB));
    add(g, "HTMLProgressElement", object(GlobalCategory::WEB));
    add(g, "HTMLQuoteElement", object(GlobalCategory::WEB));
    add(g, "HTMLScriptElement", object(GlobalCategory::WEB));
    add(g, "HTMLSelectElement", object(GlobalCategory::WEB));
    add(g, "HTMLSlotElement", object(GlobalCategory::WEB));
    add(g, "HTMLSourceElement", object(GlobalCategory::WEB));
    add(g, "HTMLSpanElement", object(GlobalCategory::WEB));
    add(g, "HTMLStyleElement", object(GlobalCategory::WEB));
    add(g, "HTMLTableCaptionElement", object(GlobalCategory::WEB));
    add(g, "HTMLTableCellElement", object(GlobalCategory::WEB));
    add(g, "HTMLTableColElement", object(GlobalCategory::WEB));
    add(g, "HTMLTableElement", object(GlobalCategory::WEB));
    add(g, "HTMLTableRowElement", object(GlobalCategory::WEB));
    add(g, "HTMLTableSectionElement", object(GlobalCategory::WEB));
    add(g, "HTMLTemplateElement", object(GlobalCategory::WEB));
    add(g, "HTMLTextAreaElement", object(GlobalCategory::WEB));
    add(g, "HTMLTimeElement", object(GlobalCategory::WEB));
    add(g, "HTMLTitleElement", object(GlobalCategory::WEB));
    add(g, "HTMLTrackElement", object(GlobalCategory::WEB));
    add(g, "HTMLUListElement", object(GlobalCategory::WEB));
    add(g, "HTMLUnknownElement", object(GlobalCategory::WEB));
    add(g, "HTMLVideoElement", object(GlobalCategory::WEB));
    add(g, "SVGElement", object(GlobalCategory::WEB));

    add(g, "AnimationEvent", object(GlobalCategory::WEB));
    add(g, "AnimationPlaybackEvent", object(GlobalCategory::WEB));
    add(g, "BeforeUnloadEvent", object(GlobalCategory::WEB));
    add(g, "CloseEvent", object(GlobalCategory::WEB));
    add(g, "CommandEvent", object(GlobalCategory::WEB));
    add(g, "CompositionEvent", object(GlobalCategory::WEB));
    add(g, "CustomEvent", object(GlobalCategory::WEB));
    add(g, "DragEvent", object(GlobalCategory::WEB));
    add(g, "ErrorEvent", object(GlobalCategory::WEB));
    add(g, "FetchEvent", object(GlobalCategory::WEB));
    add(g, "FocusEvent", object(GlobalCategory::WEB));
    add(g, "FontFaceSetLoadEvent", object(GlobalCategory::WEB));
    add(g, "FormDataEvent", object(GlobalCategory::WEB));
    add(g, "GamepadEvent", object(GlobalCategory::WEB));
    add(g, "HashChangeEvent", object(GlobalCategory::WEB));
    add(g, "InputEvent", object(GlobalCategory::WEB));
    add(g, "InstallEvent", object(GlobalCategory::WEB));
    add(g, "KeyboardEvent", object(GlobalCategory::WEB));
    add(g, "MessageEvent", object(GlobalCategory::WEB));
    add(g, "MouseEvent", object(GlobalCategory::WEB));
    add(g, "PointerEvent", object(GlobalCategory::WEB));
    add(g, "ProgressEvent", object(GlobalCategory::WEB));
    add(g, "PromiseRejectionEvent", object(GlobalCategory::WEB));
    add(g, "SubmitEvent", object(GlobalCategory::WEB));
    add(g, "TimeEvent", object(GlobalCategory::WEB));
    add(g, "ToggleEvent", object(GlobalCategory::WEB));
    add(g, "TouchEvent", object(GlobalCategory::WEB));
    add(g, "TrackEvent", object(GlobalCategory::WEB));
    add(g, "UIEvent", object(GlobalCategory::WEB));
    add(g, "WheelEvent", object(GlobalCategory::WEB));

    add(g, "navigator", object(GlobalCategory::WEB));
    add(g, "document", object(GlobalCategory::WEB));
    add(g, "crypto", object(GlobalCategory::WEB));
    add(g, "crossOriginIsolated", object(GlobalCategory::WEB));
    add(g, "customElements", object(GlobalCategory::WEB));
    add(g, "frameElement", object(GlobalCategory::WEB));
    add(g, "history", object(GlobalCategory::WEB));
    add(g, "isSecureContext", object(GlobalCategory::WEB));
    add(g, "localStorage", object(GlobalCategory::WEB));
    add(g, "sessionStorage", object(GlobalCategory::WEB));
    add(g, "trustedTypes", object(GlobalCategory::WEB));
    add(g, "setTimeout", object(GlobalCategory::WEB));
    add(g, "clearTimeout", object(GlobalCategory::WEB));
    add(g, "queueMicrotask", object(GlobalCategory::WEB));
    add(g, "performance", object(GlobalCategory::WEB));

    add(g, "MessageChannel", object(GlobalCategory::WEB));
    add(g, "MessagePort", object(GlobalCategory::WEB));
    add(g, "BroadcastChannel", object(GlobalCategory::WEB));

    add(g, "requestAnimationFrame", object(GlobalCategory::WEB));
    add(g, "cancelAnimationFrame", object(GlobalCategory::WEB));

    add(g, "Blob", object(GlobalCategory::WEB));
    add(g, "FormData", object(GlobalCategory::WEB));
    add(g, "XMLHttpRequest", object(GlobalCategory::WEB));
    add(g, "Request", object(GlobalCategory::WEB));
    add(g, "fetch", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/Streams_API
    add(g, "ReadableStream", object(GlobalCategory::WEB));
    add(g, "ReadableStreamDefaultReader", object(GlobalCategory::WEB));
    add(g, "ReadableStreamDefaultController", object(GlobalCategory::WEB));
    add(g, "WritableStream", object(GlobalCategory::WEB));
    add(g, "WritableStreamDefaultWriter", object(GlobalCategory::WEB));
    add(g, "WritableStreamDefaultController", object(GlobalCategory::WEB));
    add(g, "TransformStream", object(GlobalCategory::WEB));
    add(g, "TransformStreamDefaultController", object(GlobalCategory::WEB));
    add(g, "ByteLengthQueuingStrategy", object(GlobalCategory::WEB));
    add(g, "CountQueuingStrategy", object(GlobalCategory::WEB));
    add(g, "ReadableStreamBYOBReader", object(GlobalCategory::WEB));
    add(g, "ReadableByteStreamController", object(GlobalCategory::WEB));
    add(g, "ReadableStreamBYOBRequest", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/Clipboard_API
    add(g, "Clipboard", object(GlobalCategory::WEB));
    add(g, "ClipboardEvent", object(GlobalCategory::WEB));
    add(g, "ClipboardItem", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/CSS_Object_Model
    add(g, "AnimationEvent", object(GlobalCategory::WEB));
    add(g, "CaretPosition", object(GlobalCategory::WEB));
    add(
        g,
        "CSS",
        object(GlobalCategory::WEB)
            .with_static("highlights", object(GlobalCategory::WEB))
            .with_static("registerProperty", object(GlobalCategory::WEB_TYPED_CSS))
            .with_static("highlights", object(GlobalCategory::WEB_TYPED_CSS)),
    );
    add(g, "CSSConditionRule", object(GlobalCategory::WEB));
    add(g, "CSSCounterStyleRule", object(GlobalCategory::WEB));
    add(g, "CSSFontFaceRule", object(GlobalCategory::WEB));
    add(g, "CSSFontFeatureValuesMap", object(GlobalCategory::WEB));
    add(g, "CSSFontFeatureValuesRule", object(GlobalCategory::WEB));
    add(g, "CSSGroupingRule", object(GlobalCategory::WEB));
    add(g, "CSSImportRule", object(GlobalCategory::WEB));
    add(g, "CSSKeyframeRule", object(GlobalCategory::WEB));
    add(g, "CSSKeyframesRule", object(GlobalCategory::WEB));
    add(g, "CSSMarginRule", object(GlobalCategory::WEB));
    add(g, "CSSMediaRule", object(GlobalCategory::WEB));
    add(g, "CSSNamespaceRule", object(GlobalCategory::WEB));
    add(g, "CSSPageRule", object(GlobalCategory::WEB));
    add(g, "CSSPositionTryRule", object(GlobalCategory::WEB));
    add(g, "CSSPositionTryDescriptors", object(GlobalCategory::WEB));
    add(g, "CSSRule", object(GlobalCategory::WEB));
    add(g, "CSSRuleList", object(GlobalCategory::WEB));
    add(g, "CSSStartingStyleRule", object(GlobalCategory::WEB));
    add(g, "CSSStyleDeclaration", object(GlobalCategory::WEB));
    add(g, "CSSStyleSheet", object(GlobalCategory::WEB));
    add(g, "CSSStyleRule", object(GlobalCategory::WEB));
    add(g, "CSSSupportRule", object(GlobalCategory::WEB));
    add(g, "CSSNestedDeclarations", object(GlobalCategory::WEB));
    add(g, "FontFace", object(GlobalCategory::WEB));
    add(g, "FontFaceSet", object(GlobalCategory::WEB));
    add(g, "FontFaceSetLoadEvent", object(GlobalCategory::WEB));
    add(g, "MediaList", object(GlobalCategory::WEB));
    add(g, "MediaQueryList", object(GlobalCategory::WEB));
    add(g, "MediaQueryListEvent", object(GlobalCategory::WEB));
    add(g, "Screen", object(GlobalCategory::WEB));
    add(g, "StyleSheet", object(GlobalCategory::WEB));
    add(g, "StyleSheetList", object(GlobalCategory::WEB));
    add(g, "TransitionEvent", object(GlobalCategory::WEB));
    add(g, "VisualViewport", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/MutationObserver
    add(g, "MutationObserver", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
    add(g, "CanvasRenderingContext2D", object(GlobalCategory::WEB));
    add(g, "CanvasGradient", object(GlobalCategory::WEB));
    add(g, "CanvasPattern", object(GlobalCategory::WEB));
    add(g, "ImageBitmap", object(GlobalCategory::WEB));
    add(g, "ImageData", object(GlobalCategory::WEB));
    add(g, "TextMetrics", object(GlobalCategory::WEB));
    add(g, "OffscreenCanvas", object(GlobalCategory::WEB));
    add(g, "Path2D", object(GlobalCategory::WEB)); // Experimental
    add(g, "ImageBitmapRenderingContext", object(GlobalCategory::WEB)); // Experimental

    // https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API
    add(g, "IDBFactory", object(GlobalCategory::WEB));
    add(g, "IDBOpenDBRequest", object(GlobalCategory::WEB));
    add(g, "IDBDatabase", object(GlobalCategory::WEB));
    add(g, "IDBTransaction", object(GlobalCategory::WEB));
    add(g, "IDBRequest", object(GlobalCategory::WEB));
    add(g, "IDBObjectStore", object(GlobalCategory::WEB));
    add(g, "IDBIndex", object(GlobalCategory::WEB));
    add(g, "IDBCursor", object(GlobalCategory::WEB));
    add(g, "IDBCursorWithValue", object(GlobalCategory::WEB));
    add(g, "IDBKeyRange", object(GlobalCategory::WEB));
    add(g, "IDBVersionChangeEvent", object(GlobalCategory::WEB));
    add(g, "indexedDB", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/Credential_Management_API
    add(g, "Credential", object(GlobalCategory::WEB));
    add(g, "CredentialsContainer", object(GlobalCategory::WEB));
    add(g, "FederatedCredential", object(GlobalCategory::WEB));
    add(g, "PasswordCredential", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API
    add(g, "WorkerNavigator", object(GlobalCategory::WEB));
    add(g, "WorkerGlobalScope", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API
    add(g, "Cache", object(GlobalCategory::WEB));
    add(g, "CacheStorage", object(GlobalCategory::WEB));
    add(g, "Client", object(GlobalCategory::WEB));
    add(g, "Clients", object(GlobalCategory::WEB));
    add(g, "ExtendableEvent", object(GlobalCategory::WEB));
    add(g, "ExtendableMessageEvent", object(GlobalCategory::WEB));
    add(g, "FetchEvent", object(GlobalCategory::WEB));
    add(g, "InstallEvent", object(GlobalCategory::WEB));
    add(g, "NavigationPreloadManager", object(GlobalCategory::WEB));
    add(g, "ServiceWorker", object(GlobalCategory::WEB));
    add(g, "ServiceWorkerContainer", object(GlobalCategory::WEB));
    add(g, "ServiceWorkerGlobalScope", object(GlobalCategory::WEB));
    add(g, "ServiceWorkerRegistration", object(GlobalCategory::WEB));
    add(g, "WindowClient", object(GlobalCategory::WEB));
    add(g, "caches", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/Cookie_Store_API
    add(g, "cookieStore", object(GlobalCategory::WEB)); // Experimental
    add(g, "CookieStore", object(GlobalCategory::WEB)); // Experimental
    add(g, "cookieStoreManager", object(GlobalCategory::WEB)); // Experimental
    add(g, "CookieChangeEvent", object(GlobalCategory::WEB)); // Experimental
    add(g, "ExtendableCookieChangeEvent", object(GlobalCategory::WEB)); // Experimental

    // https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices
    add(g, "MediaDevices", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API
    add(g, "IntersectionObserver", object(GlobalCategory::WEB));
    add(g, "IntersectionObserverEntry", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/Idle_Detection_API
    add(g, "IdleDeadline", object(GlobalCategory::WEB)); // Experimental
    add(g, "requestIdleCallback", object(GlobalCategory::WEB)); // Experimental
    add(g, "cancelIdleCallback", object(GlobalCategory::WEB)); // Experimental

    // https://developer.mozilla.org/en-US/docs/Web/API/Scheduler
    add(g, "Scheduler", object(GlobalCategory::WEB)); // Experimental
    add(g, "scheduler", object(GlobalCategory::WEB)); // Experimental

    // https://developer.mozilla.org/en-US/docs/Web/API/CSS_Custom_Highlight_API
    add(g, "Highlight", object(GlobalCategory::WEB));
    add(g, "HighlightRegistry", object(GlobalCategory::WEB));

    // https://developer.mozilla.org/en-US/docs/Web/API/EditContext_API
    add(g, "EditContext", object(GlobalCategory::WEB));
    add(g, "TextFormat", object(GlobalCategory::WEB));
    add(g, "TextUpdateEvent", object(GlobalCategory::WEB));
    add(g, "TextFormatUpdateEvent", object(GlobalCategory::WEB));
    add(g, "CharacterBoundsUpdateEvent", object(GlobalCategory::WEB));
}

fn add_globals_web_typed_css(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(g, "CSSPropertyRule", object(GlobalCategory::WEB_TYPED_CSS));
    add(
        g,
        "CSSStyleValue",
        object(GlobalCategory::WEB_TYPED_CSS)
            .with_static("parseWEB_TYPED_CSS", object(GlobalCategory::WEB_TYPED_CSS))
            .with_static("parse", object(GlobalCategory::WEB_TYPED_CSS)),
    );
    add(g, "CSSImageValue", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSKeywordValue", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSMathValue", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSMathInvert", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSMathMax", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSMathMin", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSMathNegate", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSMathProduct", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSMathSum", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSNumericValue", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSNumericArray", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSPerspective", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSPositionValue", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSRotate", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSScale", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSSkew", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSSkewX", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSSkewY", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSTransformValue", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSTransformComponent", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSTranslate", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSUnitValue", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSUnparsedValue", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "CSSVariableReferenceValue", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "StylePropertyMap", object(GlobalCategory::WEB_TYPED_CSS));
    add(g, "StylePropertyMapReadOnly", object(GlobalCategory::WEB_TYPED_CSS));
}

// https://developer.mozilla.org/en-US/docs/Web/API/Background_Fetch_API
fn add_globals_web_background_fetch(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(g, "BackgroundFetchManager", object(GlobalCategory::WEB_BACKGROUND_FETCH));
    add(g, "BackgroundFetchRegistration", object(GlobalCategory::WEB_BACKGROUND_FETCH));
    add(g, "BackgroundFetchRecord", object(GlobalCategory::WEB_BACKGROUND_FETCH));
    add(g, "BackgroundFetchEvent", object(GlobalCategory::WEB_BACKGROUND_FETCH));
    add(g, "BackgroundFetchUIEvent", object(GlobalCategory::WEB_BACKGROUND_FETCH));
}

// https://developer.mozilla.org/en-US/docs/Web/API/Background_Synchronization_API
fn add_globals_web_sync(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(g, "SyncManager", object(GlobalCategory::WEB_SYNC));
    add(g, "SyncEvent", object(GlobalCategory::WEB_SYNC));
}

// https://developer.mozilla.org/en-US/docs/Web/API/Battery_Status_API
fn add_globals_web_battery(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(g, "BatteryManager", object(GlobalCategory::WEB_BATTERY));
}

// https://developer.mozilla.org/en-US/docs/Web/API/Barcode_Detection_API
fn add_globals_web_barcode(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(g, "BarcodeDetector", object(GlobalCategory::WEB_BARCODE));
}

// https://developer.mozilla.org/en-US/docs/Web/API/Web_Bluetooth_API
fn add_globals_web_bluetooth(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(g, "Bluetooth", object(GlobalCategory::WEB_BLUETOOTH));
    add(g, "BluetoothCharacteristicProperties", object(GlobalCategory::WEB_BLUETOOTH));
    add(g, "BluetoothDevice", object(GlobalCategory::WEB_BLUETOOTH));
    add(g, "BluetoothRemoteGATTCharacteristic", object(GlobalCategory::WEB_BLUETOOTH));
    add(g, "BluetoothRemoteGATTDescriptor", object(GlobalCategory::WEB_BLUETOOTH));
    add(g, "BluetoothRemoteGATTServer", object(GlobalCategory::WEB_BLUETOOTH));
    add(g, "BluetoothRemoteGATTService", object(GlobalCategory::WEB_BLUETOOTH));
}

// https://developer.mozilla.org/en-US/docs/Web/API/CSS_Painting_API
fn add_globals_web_paint(g: &mut FxHashMap<&'static str, GlobalValue>) {
    add(g, "PaintWorkletGlobalScope", object(GlobalCategory::WEB_PAINT));
    add(g, "PaintRenderingContext2D", object(GlobalCategory::WEB_PAINT));
    add(g, "PaintSize", object(GlobalCategory::WEB_PAINT));
}
