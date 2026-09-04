function test(URL) {
	return new URL("./test.css", import.meta.url).href;
}
