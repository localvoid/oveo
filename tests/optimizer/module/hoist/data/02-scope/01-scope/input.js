import { hoist, scope } from "oveo";

const a = 1;
function test(b) {
	scope(() => {
		hoist((c) => a);
	});
}
