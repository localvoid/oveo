const _GLOBAL_ = TextEncoder;
const _SINGLETON_ = new _GLOBAL_();
const _GLOBAL_2 = TextDecoder;
const _SINGLETON_2 = new _GLOBAL_2();
function test1() {
	return {
		e: _SINGLETON_,
		d: _SINGLETON_2
	};
}
function test2() {
	return {
		e: _SINGLETON_,
		d: _SINGLETON_2
	};
}
