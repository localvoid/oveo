const _DEDUPE_ = [
	1,
	2,
	3
];
const _DEDUPE_2 = { a: _DEDUPE_ };
const o1 = _DEDUPE_2;
function test() {
	const o2 = _DEDUPE_2;
	return [o1, o2];
}
