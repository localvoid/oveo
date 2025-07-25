const a = 1;
const _DEDUPE_ = { a };
_DEDUPE_;
() => {
	if (a) {
		function test() {
			_DEDUPE_;
		}
	}
};
