import { string, number, boolean, object } from "@test/oveo";
function test(x) {
	x("Test String");
	x(123);
	x(true);
	x({
		"a": [
			1,
			"two",
			true
		],
		"b": 3
	});
}
