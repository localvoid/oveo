const _GLOBAL_ = Uint8Array;
const _GLOBAL_2 = Float64Array;
const _GLOBAL_3 = _GLOBAL_2.BYTES_PER_ELEMENT;
function test(data) {
	return _GLOBAL_.from(data);
}
function test2() {
	return _GLOBAL_3;
}
function test3(data) {
	return new _GLOBAL_(data);
}
