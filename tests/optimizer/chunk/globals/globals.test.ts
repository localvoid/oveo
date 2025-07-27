import { expect, test } from "bun:test";
import { readdir } from "node:fs/promises";
import * as path from "node:path";
import { Optimizer } from "@oveo/optimizer";
import { normalizeNewlines } from "../../normalize.js";

const optimizer = new Optimizer({ globals: { include: ["js", "web"], hoist: true } });

const units = path.join(import.meta.dir, "data");
const entries = await readdir(units, { recursive: true });
for (const entry of entries) {
  try {
    const input = await Bun.file(path.join(units, entry, "input.js")).text();

    test(`chunk/globals/${entry}`, async () => {
      const output = Bun.file(path.join(units, entry, "output.js"));
      const moduleResult = await optimizer.optimizeModule(input);
      const chunkResult = await optimizer.optimizeChunk(moduleResult.code);
      expect(normalizeNewlines(chunkResult.code)).toBe(normalizeNewlines(await output.text()));
    });
  } catch (err) { }
}

