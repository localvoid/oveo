const _GLOBAL_ = Array;
const _GLOBAL_2 = _GLOBAL_.isArray;
const _GLOBAL_3 = Math;
const _GLOBAL_4 = _GLOBAL_3.random;
function test1(x) {
	if (_GLOBAL_2(x)) {
		return _GLOBAL_4();
	}
}
function test2() {
	return _GLOBAL_4();
}
