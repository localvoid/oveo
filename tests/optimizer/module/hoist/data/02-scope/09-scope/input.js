import { hoist, scope } from "oveo";

const fn = scope((inner_0) => {
	hoist((inner_1) => {
		hoist(() => {
			inner_0;
		});
	});
});