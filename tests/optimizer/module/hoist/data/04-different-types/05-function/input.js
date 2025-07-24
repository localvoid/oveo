import { hoist } from "oveo";

const a = 1;
function test(b) {
	hoist(function () { a });
	hoist(function () { a + b });
}
