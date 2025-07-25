import { hoist } from "oveo";

const a = 1;
function test() {
	hoist({ a });
	hoist({ a });
}
