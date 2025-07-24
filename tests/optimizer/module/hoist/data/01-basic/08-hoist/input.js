import { hoist } from "oveo";

const a1 = 1;
{
	const a2 = 2;
	function test(b) {
		hoist((c) => a1 + a2);
	}
}