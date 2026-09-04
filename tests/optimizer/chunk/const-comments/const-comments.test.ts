import { expect, test } from 'bun:test';
import { readdir } from 'node:fs/promises';
import * as path from 'node:path';
import { Optimizer } from '@oveo/optimizer';

import { normalizeNewlines } from '../../normalize.js';

const optimizer = new Optimizer({ dedupe: true });

// renderChunk only (no module transform): comments are consumed directly
// from the chunk source.
const units = path.join(import.meta.dir, 'data');
const entries = await readdir(units, { recursive: true });
for (const entry of entries) {
  try {
    const input = await Bun.file(path.join(units, entry, 'input.js')).text();

    test(`chunk/const-comments/${entry}`, async () => {
      const output = Bun.file(path.join(units, entry, 'output.js'));
      const result = await optimizer.renderChunk(input);
      expect(normalizeNewlines(result.code)).toBe(normalizeNewlines(await output.text()));
    });
  } catch {}
}
