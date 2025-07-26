// A lot of globals in the Web API are still missing.
// If you missing some API, submit an issue or pull request.
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

    pub fn add<'a, I: Iterator<Item = &'a str>>(&mut self, iter: I) {
        for name in iter {
            match name {
                "js" => add_globals_js(self),
                "console" => add_globals_console(self),
                "web" => add_globals_web(self),
                "web:typed-css" => add_globals_web_typed_css(self),
                "web:background-fetch" => add_globals_web_background_fetch(self),
                "web:barcode" => add_globals_web_barcode(self),
                "web:battery" => add_globals_web_battery(self),
                "web:bluetooth" => add_globals_web_bluetooth(self),
                "web:sync" => add_globals_web_sync(self),
                "web:paint" => add_globals_web_paint(self),
                _ => {}
            }
        }
    }

    fn set<T: Build<Output = GlobalValue>>(&mut self, name: &'static str, value: T) {
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

fn add_globals_js(g: &mut Globals) {
    g.set("AggregateError", object());
    g.set(
        "Array",
        object()
            .with_static("from", object())
            .with_static("fromAsync", object())
            .with_static("isArray", object())
            .with_static("of", object()),
    );
    g.set("ArrayBuffer", object().with_static("isView", object()));
    g.set("AsyncDisposableStack", object());
    g.set("AsyncFunction", object());
    g.set("AsyncGenerator", object());
    g.set("AsyncGeneratorFunction", object());
    g.set("AsyncIterator", object());
    g.set(
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
    g.set("BigInt", object().with_static("asIntN", object()).with_static("asUintN", object()));
    g.set(
        "BigInt64Array",
        object()
            .with_static("now", object())
            .with_static("parse", object())
            .with_static("UTC", object()),
    );
    g.set("BigUint64Array", object());
    g.set("Boolean", object());
    g.set("DataView", object());
    g.set("Date", object());
    g.set("DisposableStack", object());
    g.set(
        "Error",
        object().with_static("captureStackTrace", object()).with_static("isError", object()),
    );
    g.set("FinalizationRegistry", object());
    g.set("Float16Array", object());
    g.set("Float32Array", object());
    g.set("Float64Array", object());
    g.set("Function", object());
    g.set("Generator", object());
    g.set("GeneratorFunction", object());
    g.set("Infinity", object());
    g.set("Int8Array", object());
    g.set("Int16Array", object());
    g.set("Int32Array", object());
    g.set(
        "Intl",
        object()
            .with_static("getCanonicalLocales", object())
            .with_static("supportedValuesOf", object()),
    );
    g.set("Iterator", object().with_static("from", object()));
    g.set(
        "JSON",
        object()
            .with_static("isRawJSON", object())
            .with_static("parse", object())
            .with_static("rawJSON", object())
            .with_static("stringify", object()),
    );
    g.set("Map", object().with_static("groupBy", object()));
    g.set(
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
            .with_static("trunc", object())
            // Constants
            .with_static("E", object())
            .with_static("LN2", object())
            .with_static("LN10", object())
            .with_static("LOG2E", object())
            .with_static("LOG10E", object())
            .with_static("PI", object())
            .with_static("SQRT1_2", object())
            .with_static("SQRT2", object()),
    );
    g.set("NaN", object());
    g.set(
        "Number",
        object()
            .with_static("isFinite", object())
            .with_static("isInteger", object())
            .with_static("isNaN", object())
            .with_static("isSafeInteger", object())
            .with_static("parseFloat", object())
            .with_static("parseInt", object())
            // Constants
            .with_static("EPSILON", object())
            .with_static("MAX_SAFE_INTEGER", object())
            .with_static("MAX_VALUE", object())
            .with_static("MIN_SAFE_INTEGER", object())
            .with_static("MIN_VALUE", object())
            .with_static("NaN", object())
            .with_static("NEGATIVE_INFINITY", object())
            .with_static("POSITIVE_INFINITY", object()),
    );
    g.set(
        "Object",
        object()
            .with_static(
                "prototype",
                object()
                    .with_static("hasOwnProperty", object())
                    .with_static("isPrototypeOf", object())
                    .with_static("propertyIsEnumerable", object()),
            )
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
    g.set(
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
    g.set("Proxy", object());
    g.set("RangeError", object());
    g.set("ReferenceError", object());
    g.set(
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
    g.set("RegExp", object().with_static("escape", object()));
    g.set("Set", object());
    g.set("SharedArrayBuffer", object());
    g.set(
        "String",
        object()
            .with_static("fromCharCode", object())
            .with_static("fromCodePoint", object())
            .with_static("raw", object()),
    );
    g.set(
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
    g.set("SyntaxError", object());
    g.set("TextDecoder", object().with_func(func().singleton()));
    g.set("TextEncoder", object().with_func(func().singleton()));
    g.set(
        "TypedArray",
        object()
            .with_static("from", object())
            .with_static("of", object())
            .with_static("BYTES_PER_ELEMENT", object()),
    );
    g.set("TypeError", object());
    g.set("Uint8Array", object());
    g.set("Uint8ClampedArray", object());
    g.set("Uint16Array", object());
    g.set("Uint32Array", object());
    g.set("URIError", object());
    g.set("URLPattern", object());
    g.set("WeakMap", object());
    g.set("WeakRef", object());
    g.set("WeakSet", object());
    g.set("decodeURI", object());
    g.set("decodeURIComponent", object());
    g.set("encodeURI", object());
    g.set("encodeURIComponent", object());
    g.set("isFinite", object());
    g.set("isNaN", object());
    g.set("parseFloat", object());
    g.set("parseInt", object());
    g.set("undefined", object());
}

fn add_globals_console(g: &mut Globals) {
    g.set(
        "console",
        object()
            .with_static("assert", object())
            .with_static("clear", object())
            .with_static("countReset", object())
            .with_static("count", object())
            .with_static("debug", object())
            .with_static("dir", object())
            .with_static("dirxml", object())
            .with_static("error", object())
            .with_static("groupCollapsed", object())
            .with_static("groupEnd", object())
            .with_static("group", object())
            .with_static("info", object())
            .with_static("log", object())
            .with_static("profileEnd", object())
            .with_static("profile", object())
            .with_static("table", object())
            .with_static("timeEnd", object())
            .with_static("timeLog", object())
            .with_static("timeStamp", object())
            .with_static("time", object())
            .with_static("trace", object())
            .with_static("warn", object()),
    );
}

fn add_globals_web(g: &mut Globals) {
    g.set("Navigator", object());
    g.set("Window", object());
    g.set("Document", object());
    g.set("URLSearchParams", object());
    g.set("Range", object());

    g.set("Element", object());
    g.set("HTMLDocument", object());
    g.set("HTMLCollection", object());
    g.set("HTMLFormControlsCollection", object());
    g.set("HTMLOptionsCollection", object());
    g.set("HTMLElement", object());
    g.set("HTMLAreaElement", object());
    g.set("HTMLAnchorElement", object());
    g.set("HTMLAudioElement", object());
    g.set("HTMLBaseElement", object());
    g.set("HTMLBodyElement", object());
    g.set("HTMLBRElement", object());
    g.set("HTMLButtonElement", object());
    g.set("HTMLCanvasElement", object());
    g.set("HTMLDataElement", object());
    g.set("HTMLDataListElement", object());
    g.set("HTMLDetailsElement", object());
    g.set("HTMLDialogElement", object());
    g.set("HTMLDivElement", object());
    g.set("HTMLDListElement", object());
    g.set("HTMLEmbedElement", object());
    g.set("HTMLFencedFrameElement", object());
    g.set("HTMLFieldSetElement", object());
    g.set("HTMLFormElement", object());
    g.set("HTMLHeadElement", object());
    g.set("HTMLHeadingElement", object());
    g.set("HTMLHRElement", object());
    g.set("HTMLHtmlElement", object());
    g.set("HTMLIFrameElement", object());
    g.set("HTMLImageElement", object());
    g.set("HTMLInputElement", object());
    g.set("HTMLLabelElement", object());
    g.set("HTMLLegendElement", object());
    g.set("HTMLLIElement", object());
    g.set("HTMLLinkElement", object());
    g.set("HTMLMapElement", object());
    g.set("HTMLMediaElement", object());
    g.set("HTMLMenuElement", object());
    g.set("HTMLMetaElement", object());
    g.set("HTMLMeterElement", object());
    g.set("HTMLModElement", object());
    g.set("HTMLObjectElement", object());
    g.set("HTMLOListElement", object());
    g.set("HTMLOptGroupElement", object());
    g.set("HTMLOptionElement", object());
    g.set("HTMLOutputElement", object());
    g.set("HTMLParagraphElement", object());
    g.set("HTMLPictureElement", object());
    g.set("HTMLPreElement", object());
    g.set("HTMLProgressElement", object());
    g.set("HTMLQuoteElement", object());
    g.set("HTMLScriptElement", object());
    g.set("HTMLSelectElement", object());
    g.set("HTMLSlotElement", object());
    g.set("HTMLSourceElement", object());
    g.set("HTMLSpanElement", object());
    g.set("HTMLStyleElement", object());
    g.set("HTMLTableCaptionElement", object());
    g.set("HTMLTableCellElement", object());
    g.set("HTMLTableColElement", object());
    g.set("HTMLTableElement", object());
    g.set("HTMLTableRowElement", object());
    g.set("HTMLTableSectionElement", object());
    g.set("HTMLTemplateElement", object());
    g.set("HTMLTextAreaElement", object());
    g.set("HTMLTimeElement", object());
    g.set("HTMLTitleElement", object());
    g.set("HTMLTrackElement", object());
    g.set("HTMLUListElement", object());
    g.set("HTMLUnknownElement", object());
    g.set("HTMLVideoElement", object());
    g.set("SVGElement", object());

    g.set("AnimationEvent", object());
    g.set("AnimationPlaybackEvent", object());
    g.set("BeforeUnloadEvent", object());
    g.set("CloseEvent", object());
    g.set("CommandEvent", object());
    g.set("CompositionEvent", object());
    g.set("CustomEvent", object());
    g.set("DragEvent", object());
    g.set("ErrorEvent", object());
    g.set("FetchEvent", object());
    g.set("FocusEvent", object());
    g.set("FontFaceSetLoadEvent", object());
    g.set("FormDataEvent", object());
    g.set("GamepadEvent", object());
    g.set("HashChangeEvent", object());
    g.set("InputEvent", object());
    g.set("InstallEvent", object());
    g.set("KeyboardEvent", object());
    g.set("MessageEvent", object());
    g.set("MouseEvent", object());
    g.set("PointerEvent", object());
    g.set("ProgressEvent", object());
    g.set("PromiseRejectionEvent", object());
    g.set("SubmitEvent", object());
    g.set("TimeEvent", object());
    g.set("ToggleEvent", object());
    g.set("TouchEvent", object());
    g.set("TrackEvent", object());
    g.set("UIEvent", object());
    g.set("WheelEvent", object());

    g.set("navigator", object());
    g.set("document", object());
    g.set("crypto", object());
    g.set("crossOriginIsolated", object());
    g.set("customElements", object());
    g.set("frameElement", object());
    g.set("history", object());
    g.set("isSecureContext", object());
    g.set("localStorage", object());
    g.set("sessionStorage", object());
    g.set("trustedTypes", object());
    g.set("setTimeout", object());
    g.set("clearTimeout", object());
    g.set("queueMicrotask", object());
    g.set("performance", object());

    g.set("MessageChannel", object());
    g.set("MessagePort", object());
    g.set("BroadcastChannel", object());

    g.set("requestAnimationFrame", object());
    g.set("cancelAnimationFrame", object());

    g.set("Blob", object());
    g.set("FormData", object());
    g.set("XMLHttpRequest", object());
    g.set("Request", object());
    g.set("fetch", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/Streams_API
    g.set("ReadableStream", object());
    g.set("ReadableStreamDefaultReader", object());
    g.set("ReadableStreamDefaultController", object());
    g.set("WritableStream", object());
    g.set("WritableStreamDefaultWriter", object());
    g.set("WritableStreamDefaultController", object());
    g.set("TransformStream", object());
    g.set("TransformStreamDefaultController", object());
    g.set("ByteLengthQueuingStrategy", object());
    g.set("CountQueuingStrategy", object());
    g.set("ReadableStreamBYOBReader", object());
    g.set("ReadableByteStreamController", object());
    g.set("ReadableStreamBYOBRequest", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/Clipboard_API
    g.set("Clipboard", object());
    g.set("ClipboardEvent", object());
    g.set("ClipboardItem", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/CSS_Object_Model
    g.set("AnimationEvent", object());
    g.set("CaretPosition", object());
    g.set("CSS", object().with_static("highlights", object()));
    g.set("CSSConditionRule", object());
    g.set("CSSCounterStyleRule", object());
    g.set("CSSFontFaceRule", object());
    g.set("CSSFontFeatureValuesMap", object());
    g.set("CSSFontFeatureValuesRule", object());
    g.set("CSSGroupingRule", object());
    g.set("CSSImportRule", object());
    g.set("CSSKeyframeRule", object());
    g.set("CSSKeyframesRule", object());
    g.set("CSSMarginRule", object());
    g.set("CSSMediaRule", object());
    g.set("CSSNamespaceRule", object());
    g.set("CSSPageRule", object());
    g.set("CSSPositionTryRule", object());
    g.set("CSSPositionTryDescriptors", object());
    g.set("CSSRule", object());
    g.set("CSSRuleList", object());
    g.set("CSSStartingStyleRule", object());
    g.set("CSSStyleDeclaration", object());
    g.set("CSSStyleSheet", object());
    g.set("CSSStyleRule", object());
    g.set("CSSSupportRule", object());
    g.set("CSSNestedDeclarations", object());
    g.set("FontFace", object());
    g.set("FontFaceSet", object());
    g.set("FontFaceSetLoadEvent", object());
    g.set("MediaList", object());
    g.set("MediaQueryList", object());
    g.set("MediaQueryListEvent", object());
    g.set("Screen", object());
    g.set("StyleSheet", object());
    g.set("StyleSheetList", object());
    g.set("TransitionEvent", object());
    g.set("VisualViewport", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/MutationObserver
    g.set("MutationObserver", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
    g.set("CanvasRenderingContext2D", object());
    g.set("CanvasGradient", object());
    g.set("CanvasPattern", object());
    g.set("ImageBitmap", object());
    g.set("ImageData", object());
    g.set("TextMetrics", object());
    g.set("OffscreenCanvas", object());
    g.set("Path2D", object()); // Experimental
    g.set("ImageBitmapRenderingContext", object()); // Experimental

    // https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API
    g.set("IDBFactory", object());
    g.set("IDBOpenDBRequest", object());
    g.set("IDBDatabase", object());
    g.set("IDBTransaction", object());
    g.set("IDBRequest", object());
    g.set("IDBObjectStore", object());
    g.set("IDBIndex", object());
    g.set("IDBCursor", object());
    g.set("IDBCursorWithValue", object());
    g.set("IDBKeyRange", object());
    g.set("IDBVersionChangeEvent", object());
    g.set("indexedDB", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/Credential_Management_API
    g.set("Credential", object());
    g.set("CredentialsContainer", object());
    g.set("FederatedCredential", object());
    g.set("PasswordCredential", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API
    g.set("WorkerNavigator", object());
    g.set("WorkerGlobalScope", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API
    g.set("Cache", object());
    g.set("CacheStorage", object());
    g.set("Client", object());
    g.set("Clients", object());
    g.set("ExtendableEvent", object());
    g.set("ExtendableMessageEvent", object());
    g.set("FetchEvent", object());
    g.set("InstallEvent", object());
    g.set("NavigationPreloadManager", object());
    g.set("ServiceWorker", object());
    g.set("ServiceWorkerContainer", object());
    g.set("ServiceWorkerGlobalScope", object());
    g.set("ServiceWorkerRegistration", object());
    g.set("WindowClient", object());
    g.set("caches", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/Cookie_Store_API
    g.set("cookieStore", object()); // Experimental
    g.set("CookieStore", object()); // Experimental
    g.set("cookieStoreManager", object()); // Experimental
    g.set("CookieChangeEvent", object()); // Experimental
    g.set("ExtendableCookieChangeEvent", object()); // Experimental

    // https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices
    g.set("MediaDevices", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API
    g.set("IntersectionObserver", object());
    g.set("IntersectionObserverEntry", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/Idle_Detection_API
    g.set("IdleDeadline", object()); // Experimental
    g.set("requestIdleCallback", object()); // Experimental
    g.set("cancelIdleCallback", object()); // Experimental

    // https://developer.mozilla.org/en-US/docs/Web/API/Scheduler
    g.set("Scheduler", object()); // Experimental
    g.set("scheduler", object()); // Experimental

    // https://developer.mozilla.org/en-US/docs/Web/API/CSS_Custom_Highlight_API
    g.set("Highlight", object());
    g.set("HighlightRegistry", object());

    // https://developer.mozilla.org/en-US/docs/Web/API/EditContext_API
    g.set("EditContext", object());
    g.set("TextFormat", object());
    g.set("TextUpdateEvent", object());
    g.set("TextFormatUpdateEvent", object());
    g.set("CharacterBoundsUpdateEvent", object());
}

fn add_globals_web_typed_css(g: &mut Globals) {
    g.set(
        "CSS",
        object().with_static("registerProperty", object()).with_static("highlights", object()),
    );
    g.set("CSSPropertyRule", object());
    g.set(
        "CSSStyleValue",
        object().with_static("parseAll", object()).with_static("parse", object()),
    );
    g.set("CSSImageValue", object());
    g.set("CSSKeywordValue", object());
    g.set("CSSMathValue", object());
    g.set("CSSMathInvert", object());
    g.set("CSSMathMax", object());
    g.set("CSSMathMin", object());
    g.set("CSSMathNegate", object());
    g.set("CSSMathProduct", object());
    g.set("CSSMathSum", object());
    g.set("CSSNumericValue", object());
    g.set("CSSNumericArray", object());
    g.set("CSSPerspective", object());
    g.set("CSSPositionValue", object());
    g.set("CSSRotate", object());
    g.set("CSSScale", object());
    g.set("CSSSkew", object());
    g.set("CSSSkewX", object());
    g.set("CSSSkewY", object());
    g.set("CSSTransformValue", object());
    g.set("CSSTransformComponent", object());
    g.set("CSSTranslate", object());
    g.set("CSSUnitValue", object());
    g.set("CSSUnparsedValue", object());
    g.set("CSSVariableReferenceValue", object());
    g.set("StylePropertyMap", object());
    g.set("StylePropertyMapReadOnly", object());
}

// https://developer.mozilla.org/en-US/docs/Web/API/Background_Fetch_API
fn add_globals_web_background_fetch(g: &mut Globals) {
    g.set("BackgroundFetchManager", object());
    g.set("BackgroundFetchRegistration", object());
    g.set("BackgroundFetchRecord", object());
    g.set("BackgroundFetchEvent", object());
    g.set("BackgroundFetchUIEvent", object());
}

// https://developer.mozilla.org/en-US/docs/Web/API/Background_Synchronization_API
fn add_globals_web_sync(g: &mut Globals) {
    g.set("SyncManager", object());
    g.set("SyncEvent", object());
}

// https://developer.mozilla.org/en-US/docs/Web/API/Battery_Status_API
fn add_globals_web_battery(g: &mut Globals) {
    g.set("BatteryManager", object());
}

// https://developer.mozilla.org/en-US/docs/Web/API/Barcode_Detection_API
fn add_globals_web_barcode(g: &mut Globals) {
    g.set("BarcodeDetector", object());
}

// https://developer.mozilla.org/en-US/docs/Web/API/Web_Bluetooth_API
fn add_globals_web_bluetooth(g: &mut Globals) {
    g.set("Bluetooth", object());
    g.set("BluetoothCharacteristicProperties", object());
    g.set("BluetoothDevice", object());
    g.set("BluetoothRemoteGATTCharacteristic", object());
    g.set("BluetoothRemoteGATTDescriptor", object());
    g.set("BluetoothRemoteGATTServer", object());
    g.set("BluetoothRemoteGATTService", object());
}

// https://developer.mozilla.org/en-US/docs/Web/API/CSS_Painting_API
fn add_globals_web_paint(g: &mut Globals) {
    g.set("PaintWorkletGlobalScope", object());
    g.set("PaintRenderingContext2D", object());
    g.set("PaintSize", object());
}
