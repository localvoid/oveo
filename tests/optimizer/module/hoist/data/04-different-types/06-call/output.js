const a = 1;
const _HOISTED_ = call(a);
function test(b) {
	_HOISTED_;
	call(a, b);
}
