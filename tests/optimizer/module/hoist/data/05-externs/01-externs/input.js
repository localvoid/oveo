import { myhoist, myscope } from "@test/oveo";

const a = 1;
function test(b) {
	myscope(() => {
		const d = 2;
		() => {
			myhoist(1, (c) => d);
		}
	});
}
