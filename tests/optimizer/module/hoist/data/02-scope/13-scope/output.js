const a = 1;
function test(b) {
	(c) => {
		if (a) {
			(d) => {
				if (b) {
					(e) => {
						const _HOISTED_ = () => e;
						() => {
							_HOISTED_;
						};
					};
				}
			};
		}
	};
}
