import { hoist, preventHoist } from "oveo";

const a = 1;
function test(b) {
	hoist(((c) => hoist(() => a)));
}
