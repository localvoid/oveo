import { hoist } from "oveo";

{
	const a = 1;
	function test(b) {
		hoist((c) => a);
	}
}