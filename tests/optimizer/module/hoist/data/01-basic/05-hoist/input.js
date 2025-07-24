import { hoist } from "oveo";

const a = 1;
function test(b) {
	const d = 2;
	hoist((c) => d);
}
