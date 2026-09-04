const a = 1;
function test() {
	/*@__HOIST__*/(c) => a;
	/*@__HOIST__*/(c) => a;
}
