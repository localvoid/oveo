const a = 1;
const _HOISTED_ = () => a;
function test(b) {
	((c) => _HOISTED_);
}
