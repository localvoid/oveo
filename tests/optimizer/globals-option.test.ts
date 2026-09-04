import { expect, test } from 'bun:test';
import { Optimizer } from '@oveo/optimizer';

import { normalizeNewlines } from './normalize.js';

const INPUT = `function t(x) {
  console.log(Array.isArray(x));
  return [new TextEncoder(), Math.random()];
}
`;

async function optimize(input: string, options: ConstructorParameters<typeof Optimizer>[0]) {
  const optimizer = new Optimizer(options);
  const moduleResult = await optimizer.transform(input, 'js');
  return (await optimizer.renderChunk(moduleResult.code)).code;
}

test('globals:true enables everything', async () => {
  const viaShorthand = await optimize(INPUT, { globals: true });
  const viaExplicit = await optimize(INPUT, {
    globals: {
      include: ['js', 'console', 'web', 'electron', 'tauri'],
      hoist: true,
      singletons: true,
    },
  });
  // Hoists console + js globals and dedups the TextEncoder singleton.
  expect(normalizeNewlines(viaShorthand)).toContain('_SINGLETON_');
  expect(normalizeNewlines(viaShorthand)).toContain('_GLOBAL_');
  expect(normalizeNewlines(viaShorthand)).toBe(normalizeNewlines(viaExplicit));
});

test('globals:false disables global optimization', async () => {
  const viaFalse = await optimize(INPUT, { globals: false });
  const viaOmitted = await optimize(INPUT, {});
  expect(normalizeNewlines(viaFalse)).toBe(normalizeNewlines(viaOmitted));
  expect(normalizeNewlines(viaFalse)).not.toContain('_GLOBAL_');
  expect(normalizeNewlines(viaFalse)).not.toContain('_SINGLETON_');
});
