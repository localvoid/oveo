import { scope, hoist } from "oveo";

scope(() => {
	const a = 1;
	function test() {
		hoist({ a });
		hoist({ a });
	}
});
