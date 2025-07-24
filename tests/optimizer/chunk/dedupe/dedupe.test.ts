import { expect, test } from "bun:test";
import { readdir } from "node:fs/promises";
import * as path from "node:path";
import { Optimizer } from "@oveo/optimizer";

const optimizer = new Optimizer({ dedupe: true });

const units = path.join(import.meta.dir, "data");
const entries = await readdir(units, { recursive: true });
for (const entry of entries) {
  try {
    const input = await Bun.file(path.join(units, entry, "input.js")).text();

    test(`chunk/dedupe/${entry}`, async () => {
      const output = Bun.file(path.join(units, entry, "output.js"));
      const moduleResult = await optimizer.optimizeModule(input);
      const chunkResult = await optimizer.optimizeChunk(moduleResult.code);
      expect(chunkResult.code).toBe(await output.text());
    });
  } catch (err) { }
}

