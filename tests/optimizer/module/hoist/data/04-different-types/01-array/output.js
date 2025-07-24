const a = 1;
const _HOISTED_ = [
	1,
	a,
	3
];
function test(b) {
	_HOISTED_;
	[
		1,
		a,
		b
	];
}
