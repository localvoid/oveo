import { beforeAll, expect, test } from "bun:test";
import { readdir } from "node:fs/promises";
import * as path from "node:path";
import { Optimizer } from "@oveo/optimizer";

const EXTERNS = `{
  "@test/oveo": {
    "exports": {
      "string": {
        "type": "const",
        "value": "Test String"
      },
      "number": {
        "type": "const",
        "value": 123
      },
      "boolean": {
        "type": "const",
        "value": true
      },
      "object": {
        "type": "const",
        "value": { "a": [1, "two", true ], "b": 3 }
      }
    }
  }
}`;
const decoder = new TextDecoder();
const optimizer = new Optimizer({ inlineExternValues: true });
beforeAll(() => {
  optimizer.importExterns(new TextEncoder().encode(EXTERNS));
});

// read all the files in the current directory
const units = path.join(import.meta.dir, "data");
const entries = await readdir(units, { recursive: true });
for (const entry of entries) {
  try {
    const input = await Bun.file(path.join(units, entry, "input.js")).bytes();

    test(`module/hoist/${entry}`, async () => {
      const output = Bun.file(path.join(units, entry, "output.js"));
      const result = await optimizer.optimizeModule(decoder.decode(input));
      expect(result.code).toBe(decoder.decode(await output.bytes()));
    });
  } catch (err) { }
}

