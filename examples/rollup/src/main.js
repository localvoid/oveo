import { dedupe, hoist } from "oveo";
import { val } from "./test-extern.js";

console.log(val, dedupe({ a: 1 }));

const obj = {
  prop1_: 123,
  prop2_: 456,
};

function test(v) {
  const prop1_ = 1;
  obj.prop2_ = 123;

  console.log(hoist(() => { }));
  console.log(dedupe({ a: 1 }));

  if (Array.isArray(v)) {
    const encoder = new TextEncoder();
    console.log(obj, prop1_, encoder.encode("text"));
  }

}
test();
