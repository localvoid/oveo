import { hoist } from "oveo";

const a = 1;
function test(b) {
	hoist(new a());
	hoist(new b(a));
}
