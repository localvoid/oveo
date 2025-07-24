import { expect, test } from "bun:test";
import { readdir } from "node:fs/promises";
import * as path from "node:path";
import { Optimizer } from "@oveo/optimizer";

const decoder = new TextDecoder();
const optimizer = new Optimizer({ dedupe: true });

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
