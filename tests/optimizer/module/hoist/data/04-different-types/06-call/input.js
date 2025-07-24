import { hoist } from "oveo";

const a = 1;
function test(b) {
	hoist(call(a));
	hoist(call(a, b));
}
