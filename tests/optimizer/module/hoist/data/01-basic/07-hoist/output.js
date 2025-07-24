const a = 1;
const _HOISTED_ = (c) => a;
{
	function test(b) {
		_HOISTED_;
	}
}
