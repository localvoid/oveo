function test(p) {
  return Promise.all(p);
}

function test2(p) {
  return Promise.resolve(p);
}
