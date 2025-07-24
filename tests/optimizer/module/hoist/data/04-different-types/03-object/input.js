import { hoist } from "oveo";

const a = 1;
function test(b) {
	hoist({ x: a });
	hoist({ x: a, y: b });
}
