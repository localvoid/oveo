const a = 1;
const _HOISTED_ = (c) => a;
const _HOISTED_2 = (c) => a;
function test() {
	_HOISTED_;
	_HOISTED_2;
}
