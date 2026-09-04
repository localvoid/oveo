function test(data) {
  return Uint8Array.from(data);
}

function test2() {
  return Float64Array.BYTES_PER_ELEMENT;
}

function test3(data) {
  return new Uint8Array(data);
}
