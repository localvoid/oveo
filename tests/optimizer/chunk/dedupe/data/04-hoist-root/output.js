const a = 1;
const _DEDUPE_ = { a };
const _HOISTED_ = _DEDUPE_;
const _HOISTED_2 = _DEDUPE_;
function test() {
	_HOISTED_;
	_HOISTED_2;
}
