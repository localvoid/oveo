const o1 = /*@__CONST__*/({ a: [1, 2, 3] });
function test() {
	const o2 = /*@__CONST__*/({ a: [1, 2, 3] });
	return [o1, o2];
}
