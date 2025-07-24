const a = 1;
const _HOISTED_ = function() {
	a;
};
function test(b) {
	_HOISTED_;
	(function() {
		a + b;
	});
}
