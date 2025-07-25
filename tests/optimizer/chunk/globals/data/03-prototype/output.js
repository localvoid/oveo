const _GLOBAL_ = Object;
const _GLOBAL_2 = _GLOBAL_.prototype;
const _GLOBAL_3 = _GLOBAL_2.hasOwnProperty;
function test(x) {
	return _GLOBAL_3.call(x, "key");
}
