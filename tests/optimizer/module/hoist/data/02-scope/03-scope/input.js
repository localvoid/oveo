import { hoist, scope } from "oveo";

const a = 1;
function test(b) {
	scope(() => {
		const d = 2;
		if (a) {
			hoist((c) => d);
		}
	});
}
