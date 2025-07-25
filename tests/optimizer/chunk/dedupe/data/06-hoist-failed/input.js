import { dedupe, scope, hoist } from "oveo";

const a = 1;
dedupe({ a });
scope(() => {
	if (a) {
		function test() {
			hoist({ a });
		}
	}
});
