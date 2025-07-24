const a = 1;
const _HOISTED_ = a`(${a})`;
function test(b) {
	_HOISTED_;
	a`(${b})`;
}
