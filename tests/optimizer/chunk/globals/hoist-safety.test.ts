// Hoist-safety tripwire for `crates/oveo/src/globals.rs`.
//
// The chunk optimizer rewrites `A.b(...)` into `const _G = _A.b; … _G(...)`,
// which changes the receiver from `A` to `undefined`. This file checks, at
// runtime and WITHOUT involving the optimizer, that every hoistable static
// behaves identically with an `undefined` receiver.
//
// The table below is a copy of the globals registered in `globals.rs`:
// `[global name, hoistable statics]`. Keep it in sync when `globals.rs`
// changes (an unsafe addition like `Promise.all` or `Uint8Array.from` fails
// the matrix test instead of shipping broken code).
//
// Coverage notes:
// - Bare globals (empty statics) are always safe: identifier hoisting
//   (`const _G = fetch`) preserves the call shape and `this` exactly, and
//   `new _G()` preserves construction. They need no runtime proof.
// - `window` / `globalThis` resolve to the same root table, so they are
//   covered by the entries below.
// - `Object.prototype.*` intentionally diverge when detached; the optimizer
//   only ever emits them in `.call(...)` shape, which is tested separately.
// - `console.*` are host-defined and print on every call; detached calls were
//   verified manually (node + bun), structural check only.
// - `alert` / `confirm` / `prompt` block on input in a TTY; structural only.
// - `Math.random` / `Date.now` are nondeterministic; detached smoke only.
// - `TextEncoder` / `TextDecoder` are constructor singletons (the zero-arg
//   rule is enforced in `chunk/mod.rs`); interchange smoke below.
// - Entries missing in this runtime (e.g. DOM-only APIs under bun) are
//   skipped. The risky statics are all JS-core APIs present here.
import { expect, test } from 'bun:test';

const ENTRIES: Array<[string, string[]]> = [
  ['AggregateError', []],
  ['Array', ['from', 'fromAsync', 'isArray', 'of']],
  ['ArrayBuffer', ['isView']],
  ['AsyncDisposableStack', []],
  ['AsyncFunction', []],
  ['AsyncGenerator', []],
  ['AsyncGeneratorFunction', []],
  ['AsyncIterator', []],
  [
    'Atomics',
    [
      'add',
      'and',
      'compareExchange',
      'exchange',
      'isLockFree',
      'load',
      'notify',
      'or',
      'pause',
      'store',
      'sub',
      'wait',
      'waitAsync',
      'xor',
    ],
  ],
  ['BigInt', ['asIntN', 'asUintN']],
  ['Boolean', []],
  ['DataView', []],
  ['Date', ['now', 'parse', 'UTC']],
  ['DisposableStack', []],
  ['Error', ['captureStackTrace', 'isError']],
  ['EvalError', []],
  ['FinalizationRegistry', []],
  ['Function', []],
  ['Generator', []],
  ['GeneratorFunction', []],
  ['Infinity', []],
  [
    'Intl',
    [
      'getCanonicalLocales',
      'supportedValuesOf',
      'Collator',
      'DateTimeFormat',
      'DisplayNames',
      'DurationFormat',
      'ListFormat',
      'Locale',
      'NumberFormat',
      'PluralRules',
      'RelativeTimeFormat',
      'Segmenter',
    ],
  ],
  ['Iterator', ['from']],
  ['JSON', ['isRawJSON', 'parse', 'rawJSON', 'stringify']],
  ['Map', ['groupBy']],
  [
    'Math',
    [
      'abs',
      'acos',
      'acosh',
      'asin',
      'asinh',
      'atan',
      'atan2',
      'atanh',
      'cbrt',
      'ceil',
      'clz32',
      'cos',
      'cosh',
      'exp',
      'expm1',
      'f16round',
      'floor',
      'fround',
      'hypot',
      'imul',
      'log',
      'log1p',
      'log2',
      'log10',
      'max',
      'min',
      'pow',
      'random',
      'round',
      'sign',
      'sin',
      'sinh',
      'sqrt',
      'sumPrecise',
      'tan',
      'tanh',
      'trunc',
      'E',
      'LN2',
      'LN10',
      'LOG2E',
      'LOG10E',
      'PI',
      'SQRT1_2',
      'SQRT2',
    ],
  ],
  ['NaN', []],
  [
    'Number',
    [
      'isFinite',
      'isInteger',
      'isNaN',
      'isSafeInteger',
      'parseFloat',
      'parseInt',
      'EPSILON',
      'MAX_SAFE_INTEGER',
      'MAX_VALUE',
      'MIN_SAFE_INTEGER',
      'MIN_VALUE',
      'NaN',
      'NEGATIVE_INFINITY',
      'POSITIVE_INFINITY',
    ],
  ],
  [
    'Object',
    [
      'prototype',
      'assign',
      'create',
      'defineProperties',
      'defineProperty',
      'entries',
      'freeze',
      'fromEntries',
      'getOwnPropertyDescriptor',
      'getOwnPropertyDescriptors',
      'getOwnPropertyNames',
      'getOwnPropertySymbols',
      'getPrototypeOf',
      'groupBy',
      'hasOwn',
      'is',
      'isExtensible',
      'isFrozen',
      'isSealed',
      'keys',
      'preventExtensions',
      'seal',
      'setPrototypeOf',
      'values',
    ],
  ],
  ['Promise', []],
  ['Proxy', []],
  ['RangeError', []],
  ['ReferenceError', []],
  [
    'Reflect',
    [
      'apply',
      'construct',
      'defineProperty',
      'deleteProperty',
      'get',
      'getOwnPropertyDescriptor',
      'getPrototypeOf',
      'has',
      'isExtensible',
      'ownKeys',
      'preventExtensions',
      'set',
      'setPrototypeOf',
    ],
  ],
  ['RegExp', ['escape']],
  ['Set', []],
  ['SharedArrayBuffer', []],
  ['String', ['fromCharCode', 'fromCodePoint', 'raw']],
  ['SuppressedError', []],
  [
    'Symbol',
    [
      'asyncDispose',
      'dispose',
      'for',
      'keyFor',
      'asyncIterator',
      'hasInstance',
      'isConcatSpreadable',
      'iterator',
      'match',
      'matchAll',
      'replace',
      'search',
      'species',
      'split',
      'toPrimitive',
      'toStringTag',
      'unscopables',
      'metadata',
    ],
  ],
  ['SyntaxError', []],
  ['Temporal', []],
  ['TextDecoder', []],
  ['TextEncoder', []],
  ['TypeError', []],
  ['URIError', []],
  ['URLPattern', []],
  ['WeakMap', []],
  ['WeakRef', []],
  ['WeakSet', []],
  ['decodeURI', []],
  ['decodeURIComponent', []],
  ['encodeURI', []],
  ['encodeURIComponent', []],
  ['escape', []],
  ['unescape', []],
  ['isFinite', []],
  ['isNaN', []],
  ['parseFloat', []],
  ['parseInt', []],
  ['undefined', []],
  ['Float16Array', ['BYTES_PER_ELEMENT']],
  ['Float32Array', ['BYTES_PER_ELEMENT']],
  ['Float64Array', ['BYTES_PER_ELEMENT']],
  ['Uint8Array', ['BYTES_PER_ELEMENT', 'fromBase64', 'fromHex']],
  ['Uint8ClampedArray', ['BYTES_PER_ELEMENT']],
  ['Uint16Array', ['BYTES_PER_ELEMENT']],
  ['Uint32Array', ['BYTES_PER_ELEMENT']],
  ['Int8Array', ['BYTES_PER_ELEMENT']],
  ['Int16Array', ['BYTES_PER_ELEMENT']],
  ['Int32Array', ['BYTES_PER_ELEMENT']],
  ['BigInt64Array', ['BYTES_PER_ELEMENT']],
  ['BigUint64Array', ['BYTES_PER_ELEMENT']],
  [
    'console',
    [
      'assert',
      'clear',
      'countReset',
      'count',
      'debug',
      'dir',
      'dirxml',
      'error',
      'groupCollapsed',
      'groupEnd',
      'group',
      'info',
      'log',
      'profileEnd',
      'profile',
      'table',
      'timeEnd',
      'timeLog',
      'timeStamp',
      'time',
      'trace',
      'warn',
    ],
  ],
  ['Navigator', []],
  ['Window', []],
  ['Document', []],
  ['XMLDocument', []],
  ['DocumentFragment', []],
  ['DocumentType', []],
  ['CustomElementRegistry', []],
  ['ShadowRoot', []],
  ['URL', ['canParse', 'createObjectURL', 'parse', 'revokeObjectURL']],
  ['URLSearchParams', []],
  ['AbstractRange', []],
  ['Range', []],
  ['StaticRange', []],
  ['Attr', []],
  ['CDATASection', []],
  ['CharacterData', []],
  ['Comment', []],
  ['DOMImplementation', []],
  ['DOMParser', []],
  ['XMLSerializer', []],
  ['DOMTokenList', []],
  ['ProcessingInstruction', []],
  ['TimeRanges', []],
  ['TreeWalker', []],
  ['DOMException', []],
  ['Node', []],
  ['NodeIterator', []],
  ['NodeList', []],
  ['NamedNodeMap', []],
  ['Text', []],
  ['Element', []],
  ['HTMLDocument', []],
  ['HTMLCollection', []],
  ['HTMLFormControlsCollection', []],
  ['HTMLOptionsCollection', []],
  ['HTMLElement', []],
  ['HTMLAreaElement', []],
  ['HTMLAnchorElement', []],
  ['HTMLAudioElement', []],
  ['HTMLBaseElement', []],
  ['HTMLBodyElement', []],
  ['HTMLBRElement', []],
  ['HTMLButtonElement', []],
  ['HTMLCanvasElement', []],
  ['HTMLDataElement', []],
  ['HTMLDataListElement', []],
  ['HTMLDetailsElement', []],
  ['HTMLDialogElement', []],
  ['HTMLDivElement', []],
  ['HTMLDListElement', []],
  ['HTMLEmbedElement', []],
  ['HTMLFencedFrameElement', []],
  ['HTMLFieldSetElement', []],
  ['HTMLFormElement', []],
  ['HTMLHeadElement', []],
  ['HTMLHeadingElement', []],
  ['HTMLHRElement', []],
  ['HTMLHtmlElement', []],
  ['HTMLIFrameElement', []],
  ['HTMLImageElement', []],
  ['HTMLInputElement', []],
  ['HTMLLabelElement', []],
  ['HTMLLegendElement', []],
  ['HTMLLIElement', []],
  ['HTMLLinkElement', []],
  ['HTMLMapElement', []],
  ['HTMLMediaElement', []],
  ['HTMLMenuElement', []],
  ['HTMLMetaElement', []],
  ['HTMLMeterElement', []],
  ['HTMLModElement', []],
  ['HTMLObjectElement', []],
  ['HTMLOListElement', []],
  ['HTMLOptGroupElement', []],
  ['HTMLOptionElement', []],
  ['HTMLOutputElement', []],
  ['HTMLParagraphElement', []],
  ['HTMLPictureElement', []],
  ['HTMLPreElement', []],
  ['HTMLProgressElement', []],
  ['HTMLQuoteElement', []],
  ['HTMLScriptElement', []],
  ['HTMLSelectElement', []],
  ['HTMLSlotElement', []],
  ['HTMLSourceElement', []],
  ['HTMLSpanElement', []],
  ['HTMLStyleElement', []],
  ['HTMLTableCaptionElement', []],
  ['HTMLTableCellElement', []],
  ['HTMLTableColElement', []],
  ['HTMLTableElement', []],
  ['HTMLTableRowElement', []],
  ['HTMLTableSectionElement', []],
  ['HTMLTemplateElement', []],
  ['HTMLTextAreaElement', []],
  ['HTMLTimeElement', []],
  ['HTMLTitleElement', []],
  ['HTMLTrackElement', []],
  ['HTMLUListElement', []],
  ['HTMLUnknownElement', []],
  ['HTMLVideoElement', []],
  ['Audio', []],
  ['Image', []],
  ['Option', []],
  ['SVGElement', []],
  ['SVGAElement', []],
  ['SVGAnimationElement', []],
  ['SVGAnimateMotionElement', []],
  ['SVGAnimateTransformElement', []],
  ['SVGCircleElement', []],
  ['SVGClipPathElement', []],
  ['SVGComponentTransferFunctionElement', []],
  ['SVGDefsElement', []],
  ['SVGDescElement', []],
  ['SVGDiscardElement', []],
  ['SVGEllipseElement', []],
  ['SVGFEBlendElement', []],
  ['SVGFEColorMatrixElement', []],
  ['SVGFEComponentTransferElement', []],
  ['SVGFECompositeElement', []],
  ['SVGFEConvolveMatrixElement', []],
  ['SVGFEDiffuseLightingElement', []],
  ['SVGFEDisplacementMapElement', []],
  ['SVGFEDistantLightElement', []],
  ['SVGFEDropShadowElement', []],
  ['SVGFEFloodElement', []],
  ['SVGFEFuncAElement', []],
  ['SVGFEFuncBElement', []],
  ['SVGFEFuncGElement', []],
  ['SVGFEFuncRElement', []],
  ['SVGFEGaussianBlurElement', []],
  ['SVGFEImageElement', []],
  ['SVGFEMergeElement', []],
  ['SVGFEMergeNodeElement', []],
  ['SVGFEMorphologyElement', []],
  ['SVGFEOffsetElement', []],
  ['SVGFEPointLightElement', []],
  ['SVGFESpecularLightingElement', []],
  ['SVGFESpotLightElement', []],
  ['SVGFETileElement', []],
  ['SVGFETurbulenceElement', []],
  ['SVGFilterElement', []],
  ['SVGForeignObjectElement', []],
  ['SVGGElement', []],
  ['SVGGeometryElement', []],
  ['SVGGradientElement', []],
  ['SVGGraphicsElement', []],
  ['SVGImageElement', []],
  ['SVGLinearGradientElement', []],
  ['SVGLineElement', []],
  ['SVGMarkerElement', []],
  ['SVGMaskElement', []],
  ['SVGMetadataElement', []],
  ['SVGPathElement', []],
  ['SVGPatternElement', []],
  ['SVGPolylineElement', []],
  ['SVGPolygonElement', []],
  ['SVGRadialGradientElement', []],
  ['SVGRectElement', []],
  ['SVGScriptElement', []],
  ['SVGSetElement', []],
  ['SVGStopElement', []],
  ['SVGStyleElement', []],
  ['SVGSVGElement', []],
  ['SVGSwitchElement', []],
  ['SVGSymbolElement', []],
  ['SVGTextContentElement', []],
  ['SVGTextElement', []],
  ['SVGTextPathElement', []],
  ['SVGTextPositioningElement', []],
  ['SVGTitleElement', []],
  ['SVGTSpanElement', []],
  ['SVGUseElement', []],
  ['SVGViewElement', []],
  ['SVGAngle', []],
  ['SVGLength', []],
  ['SVGLengthList', []],
  ['SVGNumber', []],
  ['SVGNumberList', []],
  ['SVGPreserveAspectRatio', []],
  ['SVGStringList', []],
  ['SVGTransform', []],
  ['SVGTransformList', []],
  ['SVGAnimatedAngle', []],
  ['SVGAnimatedBoolean', []],
  ['SVGAnimatedEnumeration', []],
  ['SVGAnimatedInteger', []],
  ['SVGAnimatedLength', []],
  ['SVGAnimatedLengthList', []],
  ['SVGAnimatedNumber', []],
  ['SVGAnimatedNumberList', []],
  ['SVGAnimatedPreserveAspectRatio', []],
  ['SVGAnimatedRect', []],
  ['SVGAnimatedString', []],
  ['SVGAnimatedTransformList', []],
  ['TimeEvent', []],
  ['ShadowAnimation', []],
  ['SVGUnitTypes', []],
  ['SVGUseElementShadowRoot', []],
  ['DOMMatrix', ['fromFloat32Array', 'fromFloat64Array', 'fromMatrix']],
  ['DOMMatrixReadOnly', ['fromFloat32Array', 'fromFloat64Array', 'fromMatrix']],
  ['DOMPoint', ['fromPoint']],
  ['DOMPointReadOnly', ['fromPoint']],
  ['DOMQuad', []],
  ['DOMRect', ['fromRect']],
  ['DOMRectReadOnly', ['fromRect']],
  ['Selection', []],
  ['getSelection', []],
  ['Event', []],
  ['EventTarget', []],
  ['BeforeUnloadEvent', []],
  ['CloseEvent', []],
  ['CommandEvent', []],
  ['CompositionEvent', []],
  ['CustomEvent', []],
  ['ErrorEvent', []],
  ['FetchEvent', []],
  ['FocusEvent', []],
  ['FormDataEvent', []],
  ['GamepadEvent', []],
  ['HashChangeEvent', []],
  ['InputEvent', []],
  ['KeyboardEvent', []],
  ['MessageEvent', []],
  ['MouseEvent', []],
  ['PointerEvent', []],
  ['ProgressEvent', []],
  ['PromiseRejectionEvent', []],
  ['SubmitEvent', []],
  ['ToggleEvent', []],
  ['TouchEvent', []],
  ['TrackEvent', []],
  ['UIEvent', []],
  ['WheelEvent', []],
  ['navigator', []],
  ['document', []],
  ['self', []],
  ['location', []],
  ['top', []],
  ['parent', []],
  ['frames', []],
  ['screen', []],
  ['structuredClone', []],
  ['atob', []],
  ['btoa', []],
  ['alert', []],
  ['addEventListener', []],
  ['removeEventListener', []],
  ['devicePixelRatio', []],
  ['innerWidth', []],
  ['innerHeight', []],
  ['outerWidth', []],
  ['outerHeight', []],
  ['crossOriginIsolated', []],
  ['customElements', []],
  ['frameElement', []],
  ['isSecureContext', []],
  ['localStorage', []],
  ['sessionStorage', []],
  ['trustedTypes', []],
  ['setTimeout', []],
  ['clearTimeout', []],
  ['setInterval', []],
  ['clearInterval', []],
  ['queueMicrotask', []],
  ['performance', []],
  ['Performance', []],
  ['PerformanceEntry', []],
  ['PerformanceObserver', []],
  ['open', []],
  ['close', []],
  ['stop', []],
  ['confirm', []],
  ['focus', []],
  ['moveBy', []],
  ['moveTo', []],
  ['createImageBitmap', []],
  ['print', []],
  ['prompt', []],
  ['reportError', []],
  ['resizeBy', []],
  ['resizeTo', []],
  ['scroll', []],
  ['scrollBy', []],
  ['scrollTo', []],
  ['MessageChannel', []],
  ['MessagePort', []],
  ['BroadcastChannel', []],
  ['postMessage', []],
  ['requestAnimationFrame', []],
  ['cancelAnimationFrame', []],
  ['DataTransfer', []],
  ['DataTransferItem', []],
  ['DataTransferItemList', []],
  ['DragEvent', []],
  ['AbortController', []],
  ['AbortSignal', ['abort', 'any', 'timeout']],
  ['Blob', []],
  ['File', []],
  ['FileReader', []],
  ['VideoFrame', []],
  ['FormData', []],
  ['XMLHttpRequest', []],
  ['Headers', []],
  ['Request', []],
  ['Response', []],
  ['fetch', []],
  ['WebSocket', []],
  ['WebSocketStream', []],
  ['EventSource', []],
  [
    'WebAssembly',
    [
      'compile',
      'compileStreaming',
      'instantiate',
      'instantiateStreaming',
      'validate',
      'Module',
      'Instance',
      'Memory',
      'Table',
      'Global',
      'Tag',
    ],
  ],
  ['ReadableStream', []],
  ['ReadableStreamDefaultReader', []],
  ['ReadableStreamDefaultController', []],
  ['WritableStream', []],
  ['WritableStreamDefaultWriter', []],
  ['WritableStreamDefaultController', []],
  ['TransformStream', []],
  ['TransformStreamDefaultController', []],
  ['ByteLengthQueuingStrategy', []],
  ['CountQueuingStrategy', []],
  ['CompressionStream', []],
  ['DecompressionStream', []],
  ['ReadableStreamBYOBReader', []],
  ['ReadableByteStreamController', []],
  ['ReadableStreamBYOBRequest', []],
  ['Clipboard', []],
  ['ClipboardEvent', []],
  ['ClipboardItem', []],
  ['History', []],
  ['PopStateEvent', []],
  ['history', []],
  ['Location', []],
  ['Notification', ['permission', 'requestPermission']],
  ['getComputedStyle', []],
  ['matchMedia', []],
  ['CaretPosition', []],
  [
    'CSS',
    [
      'highlights',
      'supports',
      'escape',
      'registerProperty',
      'Hz',
      'Q',
      'cap',
      'ch',
      'cm',
      'cbq',
      'cqh',
      'cqi',
      'cqmax',
      'cqmin',
      'cqw',
      'deg',
      'dpqm',
      'dpi',
      'dppx',
      'dvb',
      'dvh',
      'dvi',
      'dvmax',
      'dvmin',
      'dvw',
      'em',
      'ex',
      'fr',
      'grad',
      'ic',
      'in',
      'kHz',
      'lh',
      'lvb',
      'lvh',
      'lvi',
      'lvmax',
      'lvmin',
      'lvw',
      'mm',
      'ms',
      'number',
      'pc',
      'percent',
      'pt',
      'px',
      'rad',
      'rcap',
      'rch',
      'rem',
      'rex',
      'ric',
      'rlh',
      's',
      'svb',
      'svh',
      'svi',
      'svmax',
      'svmin',
      'svw',
      'turn',
      'vb',
      'vh',
      'vi',
      'vmax',
      'vmin',
      'vw',
      'paintWorklet',
    ],
  ],
  ['CSSConditionRule', []],
  ['CSSCounterStyleRule', []],
  ['CSSFontFaceRule', []],
  ['CSSFontFeatureValuesMap', []],
  ['CSSFontFeatureValuesRule', []],
  ['CSSGroupingRule', []],
  ['CSSImportRule', []],
  ['CSSKeyframeRule', []],
  ['CSSKeyframesRule', []],
  ['CSSMarginRule', []],
  ['CSSMediaRule', []],
  ['CSSNamespaceRule', []],
  ['CSSPageRule', []],
  ['CSSPositionTryRule', []],
  ['CSSPositionTryDescriptors', []],
  ['CSSRule', []],
  ['CSSRuleList', []],
  ['CSSStartingStyleRule', []],
  ['CSSStyleDeclaration', []],
  ['CSSStyleSheet', []],
  ['CSSStyleRule', []],
  ['CSSSupportRule', []],
  ['CSSNestedDeclarations', []],
  ['FontFace', []],
  ['FontFaceSet', []],
  ['FontFaceSetLoadEvent', []],
  ['MediaList', []],
  ['MediaQueryList', []],
  ['MediaQueryListEvent', []],
  ['Screen', []],
  ['StyleSheet', []],
  ['StyleSheetList', []],
  ['TransitionEvent', []],
  ['VisualViewport', []],
  ['CSSPropertyRule', []],
  ['CSSStyleValue', ['parseAll', 'parse']],
  ['CSSImageValue', []],
  ['CSSKeywordValue', []],
  ['CSSMathValue', []],
  ['CSSMathInvert', []],
  ['CSSMathMax', []],
  ['CSSMathMin', []],
  ['CSSMathNegate', []],
  ['CSSMathProduct', []],
  ['CSSMathSum', []],
  ['CSSNumericValue', []],
  ['CSSNumericArray', []],
  ['CSSPerspective', []],
  ['CSSPositionValue', []],
  ['CSSRotate', []],
  ['CSSScale', []],
  ['CSSSkew', []],
  ['CSSSkewX', []],
  ['CSSSkewY', []],
  ['CSSTransformValue', []],
  ['CSSTransformComponent', []],
  ['CSSTranslate', []],
  ['CSSUnitValue', []],
  ['CSSUnparsedValue', []],
  ['CSSVariableReferenceValue', []],
  ['StylePropertyMap', []],
  ['StylePropertyMapReadOnly', []],
  ['MutationObserver', []],
  ['CanvasRenderingContext2D', []],
  ['CanvasGradient', []],
  ['CanvasPattern', []],
  ['ImageBitmap', []],
  ['ImageData', []],
  ['TextMetrics', []],
  ['OffscreenCanvas', []],
  ['Path2D', []],
  ['ImageBitmapRenderingContext', []],
  ['Animation', []],
  ['AnimationEffect', []],
  ['AnimationEvent', []],
  ['AnimationTimeline', []],
  ['AnimationPlaybackEvent', []],
  ['DocumentTimeline', []],
  ['KeyframeEffect', []],
  ['ScrollTimeline', []],
  ['ViewTimeline', []],
  ['Storage', []],
  ['StorageEvent', []],
  ['StorageManager', []],
  ['IDBFactory', []],
  ['IDBOpenDBRequest', []],
  ['IDBDatabase', []],
  ['IDBTransaction', []],
  ['IDBRequest', []],
  ['IDBObjectStore', []],
  ['IDBIndex', []],
  ['IDBCursor', []],
  ['IDBCursorWithValue', []],
  ['IDBKeyRange', []],
  ['IDBVersionChangeEvent', []],
  ['indexedDB', []],
  ['Credential', []],
  ['CredentialsContainer', []],
  ['FederatedCredential', []],
  ['PasswordCredential', []],
  ['Worker', []],
  ['SharedWorker', []],
  ['WorkerNavigator', []],
  ['WorkerGlobalScope', []],
  ['Cache', []],
  ['CacheStorage', []],
  ['Client', []],
  ['Clients', []],
  ['ExtendableEvent', []],
  ['ExtendableMessageEvent', []],
  ['InstallEvent', []],
  ['NavigationPreloadManager', []],
  ['ServiceWorker', []],
  ['ServiceWorkerContainer', []],
  ['ServiceWorkerGlobalScope', []],
  ['ServiceWorkerRegistration', []],
  ['WindowClient', []],
  ['caches', []],
  ['cookieStore', []],
  ['CookieStore', []],
  ['cookieStoreManager', []],
  ['CookieChangeEvent', []],
  ['ExtendableCookieChangeEvent', []],
  ['MediaDevices', []],
  ['DeviceMotionEvent', []],
  ['DeviceMotionEventAcceleration', []],
  ['DeviceMotionEventRotationRate', []],
  ['DeviceOrientationEvent', []],
  ['ResizeObserver', []],
  ['ResizeObserverEntry', []],
  ['IntersectionObserver', []],
  ['IntersectionObserverEntry', []],
  ['IdleDeadline', []],
  ['requestIdleCallback', []],
  ['cancelIdleCallback', []],
  ['Scheduler', []],
  ['scheduler', []],
  ['Highlight', []],
  ['HighlightRegistry', []],
  ['EditContext', []],
  ['TextFormat', []],
  ['TextUpdateEvent', []],
  ['TextFormatUpdateEvent', []],
  ['CharacterBoundsUpdateEvent', []],
  ['PaintWorkletGlobalScope', []],
  ['PaintRenderingContext2D', []],
  ['PaintSize', []],
  ['BackgroundFetchManager', []],
  ['BackgroundFetchRegistration', []],
  ['BackgroundFetchRecord', []],
  ['BackgroundFetchEvent', []],
  ['BackgroundFetchUIEvent', []],
  ['SyncManager', []],
  ['SyncEvent', []],
  ['BatteryManager', []],
  ['BarcodeDetector', []],
  ['Bluetooth', []],
  ['BluetoothCharacteristicProperties', []],
  ['BluetoothDevice', []],
  ['BluetoothRemoteGATTCharacteristic', []],
  ['BluetoothRemoteGATTDescriptor', []],
  ['BluetoothRemoteGATTServer', []],
  ['BluetoothRemoteGATTService', []],
  ['Crypto', []],
  ['SubtleCrypto', []],
  ['CryptoKey', []],
  ['AesCbcParams', []],
  ['AesCtrParams', []],
  ['AesGcmParams', []],
  ['AesKeyGenParams', []],
  ['CryptoKeyPair', []],
  ['EcKeyGenParams', []],
  ['EcKeyImportParams', []],
  ['EcdhKeyDeriveParams', []],
  ['EcdsaParams', []],
  ['HkdfParams', []],
  ['HmacImportParams', []],
  ['HmacKeyGenParams', []],
  ['Pbkdf2Params', []],
  ['RsaHashedImportParams', []],
  ['RsaHashedKeyGenParams', []],
  ['RsaOaepParams', []],
  ['RsaPssParams', []],
  ['crypto', []],
  ['Geolocation', []],
  ['GeolocationPosition', []],
  ['GeolocationCoordinates', []],
  ['GeolocationPositionError', []],
];

// Paths below the top level (only `Object.prototype.*` in `globals.rs`).
// The optimizer only emits these in `.call(...)` shape (see below); if
// `globals.rs` nests more entries, extend this list and the test.
const NESTED: string[][] = [
  ['Object', 'prototype', 'hasOwnProperty'],
  ['Object', 'prototype', 'isPrototypeOf'],
  ['Object', 'prototype', 'propertyIsEnumerable'],
];

const BY_NAME = new Map(ENTRIES);

test('globals table shape', () => {
  expect(ENTRIES.length).toBeGreaterThan(200);
  expect(BY_NAME.has('TextEncoder')).toBe(true);
  expect(BY_NAME.has('TextDecoder')).toBe(true);

  // `this`-dependent statics must stay out of the table: detached calls to
  // these throw (see the matrix test below), so hoisting them would break.
  expect(BY_NAME.get('Promise')).toEqual([]);
  const TYPED_ARRAYS = [
    'Float16Array',
    'Float32Array',
    'Float64Array',
    'Uint8Array',
    'Uint8ClampedArray',
    'Uint16Array',
    'Uint32Array',
    'Int8Array',
    'Int16Array',
    'Int32Array',
    'BigInt64Array',
    'BigUint64Array',
  ];
  for (const name of TYPED_ARRAYS) {
    expect([...BY_NAME.get(name)!].sort()).toEqual(
      name === 'Uint8Array'
        ? ['BYTES_PER_ELEMENT', 'fromBase64', 'fromHex']
        : ['BYTES_PER_ELEMENT'],
    );
  }

  // `Array.from` / `of` / `fromAsync` read `this` but fall back to a plain
  // `Array` when detached — identical to calling them on the base `Array`
  // constructor, which is the only receiver this table can match.
  expect(BY_NAME.get('Array')).toEqual(
    expect.arrayContaining(['from', 'fromAsync', 'isArray', 'of']),
  );

  // Singleton constructors carry no statics.
  expect(BY_NAME.get('TextEncoder')).toEqual([]);
  expect(BY_NAME.get('TextDecoder')).toEqual([]);

  expect(NESTED.map((p) => p.join('.')).sort()).toEqual([
    'Object.prototype.hasOwnProperty',
    'Object.prototype.isPrototypeOf',
    'Object.prototype.propertyIsEnumerable',
  ]);
});

type Outcome =
  | { status: 'return'; value: unknown }
  | { status: 'throw'; name: string; message: string };

async function probe(fn: Function, thisArg: unknown, args: unknown[]): Promise<Outcome> {
  try {
    return { status: 'return', value: await Reflect.apply(fn, thisArg, args) };
  } catch (e) {
    return {
      status: 'throw',
      name: (e as Error)?.constructor?.name ?? typeof e,
      message: String((e as Error)?.message ?? e),
    };
  }
}

function valuesEqual(a: unknown, b: unknown): boolean {
  if (Object.is(a, b)) return true;
  // Functions (e.g. input-passthrough like `Object.assign(fn)` returning its
  // argument): probe shapes are freshly built per run, so identity differs
  // by construction — compare source instead.
  if (typeof a === 'function' || typeof b === 'function') {
    return typeof a === 'function' && typeof b === 'function' && String(a) === String(b);
  }
  if (typeof a !== typeof b || typeof a !== 'object' || a === null || b === null) return false;
  if ((a as object).constructor !== (b as object).constructor) return false;
  try {
    const sa = JSON.stringify(a);
    const sb = JSON.stringify(b);
    if (sa !== undefined || sb !== undefined) return sa === sb;
  } catch {
    // Non-serializable exotic objects (e.g. timers): same constructor is
    // the best available signal; attached/detached produced equivalents.
  }
  return true;
}

function outcomesEqual(a: Outcome, b: Outcome): boolean {
  if (a.status !== b.status) return false;
  if (a.status === 'throw' && b.status === 'throw')
    return a.name === b.name && a.message === b.message;
  if (a.status === 'return' && b.status === 'return') return valuesEqual(a.value, b.value);
  return false;
}

// Generic argument shapes, as factories returning FRESH values on every call.
// Freshness matters: some functions mutate their arguments (`Atomics.add`,
// `Object.freeze`, …), and sharing one object between the attached and
// detached runs would compare the second run against mutated state.
// Exact validity per function doesn't matter: what matters is that attached
// and detached runs take identical code paths for identical arguments.
// Shapes include valid calls for the common cases (`Array.from`, `groupBy`,
// `String.raw`, …) to also cover post-validation `this` use.
const PROBE_SHAPES: Array<() => unknown[]> = [
  () => [],
  () => [undefined],
  () => [null],
  () => [0],
  () => [1],
  () => [''],
  () => ['x'],
  () => ['en'],
  () => [{}],
  () => [{ raw: ['a', 'b'] }],
  () => [[]],
  () => [[1, 2]],
  () => [[1, 2], (x: unknown) => x],
  () => [() => {}],
  () => [new Uint8Array([1, 2])],
];

// Verified manually to work detached (node + bun); invoking them here would
// print on every call, so structural check only.
const PRINTING = new Set(['console']);
// Block on user input in a TTY; structural check only.
const INTERACTIVE = new Set(['alert', 'confirm', 'prompt']);
// Nondeterministic; detached smoke check only.
const NONDETERMINISTIC = new Set(['Math.random', 'Date.now']);

function fmtOutcome(o: Outcome): string {
  return o.status === 'return'
    ? `returned ${String(o.value).slice(0, 60)}`
    : `threw ${o.name}: ${o.message.slice(0, 80)}`;
}

test('hoisted statics are receiver-independent', async () => {
  const failures: string[] = [];
  let checked = 0;
  let skipped = 0;

  for (const [name, statics] of ENTRIES) {
    const receiver = (globalThis as Record<string, unknown>)[name];
    if (receiver === undefined) {
      skipped += statics.length;
      continue;
    }
    if (statics.length === 0) {
      // Bare global: identifier hoisting preserves call shape and `this`.
      continue;
    }
    for (const prop of statics) {
      const key = `${name}.${prop}`;
      const value = (receiver as Record<string, unknown>)[prop];
      if (value === undefined) {
        skipped++;
        continue;
      }
      if (typeof value !== 'function') {
        // Data property (constants, well-known symbols, namespaces…):
        // hoisting copies the value, always safe.
        checked++;
        continue;
      }
      if (PRINTING.has(name) || INTERACTIVE.has(prop)) {
        checked++;
        continue;
      }
      if (NONDETERMINISTIC.has(key)) {
        const out = await probe(value as Function, undefined, []);
        if (out.status !== 'return' || typeof out.value !== 'number') {
          failures.push(`${key} detached smoke failed: ${fmtOutcome(out)}`);
        }
        checked++;
        continue;
      }
      for (const makeArgs of PROBE_SHAPES) {
        // Fresh arguments for each run (see PROBE_SHAPES comment).
        const attached = await probe(value as Function, receiver, makeArgs());
        const detached = await probe(value as Function, undefined, makeArgs());
        if (!outcomesEqual(attached, detached)) {
          failures.push(
            `${key} diverges for args ${JSON.stringify(makeArgs()).slice(0, 60)}: ` +
              `attached ${fmtOutcome(attached)} vs detached ${fmtOutcome(detached)}`,
          );
          break;
        }
      }
      checked++;
    }
  }

  // Guard against the test silently covering nothing (e.g. a runtime
  // missing most globals).
  expect(checked).toBeGreaterThan(100);
  expect(failures).toEqual([]);
});

test('Object.prototype methods work via .call (their only hoisted shape)', () => {
  const o = { a: 1 };
  expect(Object.prototype.hasOwnProperty.call(o, 'a')).toBe(true);
  expect(Object.prototype.hasOwnProperty.call(o, 'b')).toBe(false);
  expect(Object.prototype.isPrototypeOf.call(Object.prototype, o)).toBe(true);
  expect(Object.prototype.propertyIsEnumerable.call(o, 'a')).toBe(true);
});

test('nondeterministic statics work detached', () => {
  expect(typeof Reflect.apply(Math.random, undefined, [])).toBe('number');
  expect(typeof Reflect.apply(Date.now, undefined, [])).toBe('number');
});

test('TextEncoder/TextDecoder instances are interchangeable (singleton assumption)', () => {
  const bytes = new TextEncoder().encode('hello wörld');
  expect([...new TextEncoder().encode('hello wörld')]).toEqual([...bytes]);
  expect(new TextDecoder().decode(bytes)).toBe('hello wörld');
});
