const a = 1;
function test(b) {
	() => {
		const d = 2;
		if (a) {
			(e) => {
				const _HOISTED_ = (c) => e;
				() => {
					_HOISTED_;
				};
			};
		}
	};
}
