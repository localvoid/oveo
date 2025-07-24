import { myhoist, myscope } from "@test/oveo";
const a = 1;
function test(b) {
	myscope(() => {
		const d = 2;
		const _HOISTED_ = (c) => d;
		() => {
			myhoist(1, _HOISTED_);
		};
	});
}
