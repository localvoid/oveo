const _DEDUPE_ = { a: 1 };
const o1 = _DEDUPE_;
const o2 = _DEDUPE_;
const o3 = { a: 1 };
function test() {
	return [
		o1,
		o2,
		o3
	];
}
