import { hoist } from "oveo";

const a = 1;
function test(b) {
	hoist({ a });
	hoist({ a, b });
}
