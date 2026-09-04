const a = 1;
function test(b) {
	const y = /*#__HOIST__*/(c) => a;
	const z = /*@__HOIST__*/((c) => a);
	return [y, z];
}