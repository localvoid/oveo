const a = 1;
function test(b) {
	const x = /* @__HOIST__ */ (c) => a;
	const y = /* hoist @__HOIST__ please */ (c) => a;
	const z = // @__HOIST__
	(c) => a;
	return [x, y, z];
}