import { expect, test } from "bun:test";
import { readdir } from "node:fs/promises";
import * as path from "node:path";
import { Optimizer } from "@oveo/optimizer";
import { normalizeNewlines } from "../../normalize.js";

const decoder = new TextDecoder();
const units = path.join(import.meta.dir, "data");
const entries = await readdir(units, { recursive: true });
const renameProperties = { pattern: "_$" };
for (const entry of entries) {
  try {
    const input = await Bun.file(path.join(units, entry, "input.js")).text();

    test(`chunk/rename_properties/${entry}`, async () => {
      const optimizer = new Optimizer({ renameProperties });
      const propsImport = Bun.file(path.join(units, entry, "props-import.ini"));
      if (await propsImport.exists()) {
        optimizer.importPropertyMap(await propsImport.bytes());
      }

      const output = Bun.file(path.join(units, entry, "output.js"));
      const moduleResult = await optimizer.transform(input, "js");
      const chunkResult = await optimizer.renderChunk(moduleResult.code);
      expect(normalizeNewlines(chunkResult.code)).toBe(normalizeNewlines(await output.text()));

      const propsExport = Bun.file(path.join(units, entry, "props-export.ini"));
      const newPropsData = optimizer.updatePropertyMap();

      if (await propsExport.exists()) {
        expect(newPropsData).not.toBe(null);
        expect(decoder.decode(newPropsData!))
          .toBe(normalizeNewlines(await propsExport.text()));
      } else {
        expect(newPropsData).toBe(null);
      }
    });
  } catch (err) { }
}
;
