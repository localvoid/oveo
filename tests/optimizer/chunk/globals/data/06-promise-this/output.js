const _GLOBAL_ = Promise;
function test(p) {
	return _GLOBAL_.all(p);
}
function test2(p) {
	return _GLOBAL_.resolve(p);
}
