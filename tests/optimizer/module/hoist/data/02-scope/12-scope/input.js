import { hoist, scope } from "oveo";

const a = 1;
function test(b) {
	scope((c) => {
		if (a) {
			scope((d) => {
				if (b) {
					scope((e) => {
						() => {
							hoist(() => d);
						}
					});
				}
			});
		}
	});
}
