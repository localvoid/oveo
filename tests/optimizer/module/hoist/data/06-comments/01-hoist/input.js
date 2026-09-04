const a = 1;
function test(b) {
	/*@__HOIST__*/(c) => a;
}
