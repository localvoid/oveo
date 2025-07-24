const a = 1;
const _HOISTED_ = new a();
function test(b) {
	_HOISTED_;
	new b(a);
}
