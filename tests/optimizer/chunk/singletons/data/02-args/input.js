function test1() {
  return {
    e: new TextEncoder(),
    d: new TextDecoder(),
  };
}

function test2() {
  return {
    e: new TextEncoder("utf-8"),
    d: new TextDecoder("utf-16", { fatal: true }),
  };
}
