import { hoist } from "oveo";

const a = 1;
function test(b) {
	hoist([1, a, 3]);
	hoist([1, a, b]);
}
