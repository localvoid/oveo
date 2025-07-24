const a = 1;
const _HOISTED_ = (c) => a + c;
function test(b) {
	_HOISTED_;
}
