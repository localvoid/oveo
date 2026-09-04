const o1 = /* @__CONST__ */ ({ a: 1 });
const o2 = /* keep @__CONST__ around */ ({ a: 1 });
// @__CONST__
const o3 = { a: 1 };
function test() {
	return [o1, o2, o3];
}
