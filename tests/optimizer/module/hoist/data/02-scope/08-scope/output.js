const a = 1;
function test(b) {
	() => {
		const d = 2;
		if (a) {
			(e) => {
				() => {
					(c) => b;
				};
			};
		}
	};
}
