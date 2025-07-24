import { string, number, boolean, object } from "@test/oveo";

function test(x) {
	x(string);
	x(number);
	x(boolean);
	x(object);
}
