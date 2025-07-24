import * as oveo from "oveo";
import { val } from "test";

console.log(val);
console.log(oveo.dedupe({ a: 1 }));

const obj = {
  prop1_: 123,
  prop2_: 456,
};

function test(v) {
  const prop1_ = 1;
  obj.prop2_ = 123;

  console.log(oveo.hoist(() => { }));
  console.log(oveo.dedupe({ a: 1 }));

  if (Array.isArray(v)) {
    const encoder = new TextEncoder();
    encoder.encode("text");
  }

  console.log(obj, prop1_, prop2_);
}
test();