const a = 1;
const _HOISTED_ = { x: () => a };
function test(b) {
	_HOISTED_;
	({
		x: () => a,
		y: () => b
	});
}
