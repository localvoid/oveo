function test() {
  return new URL("./test.css", import.meta.url).pathname;
}
