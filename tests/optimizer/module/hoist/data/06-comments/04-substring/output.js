const a = 1;
const _HOISTED_ = (c) => a;
const _HOISTED_2 = (c) => a;
const _HOISTED_3 = (c) => a;
function test(b) {
	const x = _HOISTED_;
	const y = _HOISTED_2;
	const z = _HOISTED_3;
	return [
		x,
		y,
		z
	];
}
