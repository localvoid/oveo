import { beforeAll, expect, test } from "bun:test";
import { readdir } from "node:fs/promises";
import * as path from "node:path";
import { Optimizer } from "@oveo/optimizer";
import { normalizeNewlines } from "../../normalize.ts";

const EXTERNS = `{
  "@test/oveo": {
    "type": "module",
    "exports": {
      "myhoist": {
        "type": "function",
        "arguments": [{}, { "hoist": true }]
      },
      "myscope": {
        "type": "function",
        "arguments": [{ "scope": true }]
      }
    }
  }
}`;
const optimizer = new Optimizer({ hoist: true });
beforeAll(() => {
  optimizer.importExterns(new TextEncoder().encode(EXTERNS));
});

// read all the files in the current directory
const units = path.join(import.meta.dir, "data");
const entries = await readdir(units, { recursive: true });
for (const entry of entries) {
  try {
    const input = await Bun.file(path.join(units, entry, "input.js")).text();

    test(`module/hoist/${entry}`, async () => {
      const output = Bun.file(path.join(units, entry, "output.js"));
      const result = await optimizer.optimizeModule(input, "js");
      expect(normalizeNewlines(result.code)).toBe(normalizeNewlines(await output.text()));
    });
  } catch (err) { }
}

