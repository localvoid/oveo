const a = 1;
function test(b) {
	const y = (c) => a;
	const z = ((c) => a);
	return [y, z];
}
