// @vitest-environment node
// Every class the code uses must make CSS (D1+D2 review, finding 6). tokens.css drops Tailwind's
// defaults, so a class such as `pe-12` or `max-w-xl` builds to nothing and the screen silently
// loses a space or a width. This test builds every class with the real Tailwind compiler.

import { describe, expect, it } from 'vitest';
import { classesInSource, classesWithNoCss } from '../helpers/tailwind-classes';

const BUILD_TIMEOUT_MS = 120_000;

describe('classes used in the code', () => {
  it(
    'all make CSS with the design tokens',
    async () => {
      const uses = classesInSource();
      expect(uses.length).toBeGreaterThan(150);
      const missing = await classesWithNoCss(uses);
      expect(missing.map((use) => `${use.name} (${use.file})`)).toEqual([]);
    },
    BUILD_TIMEOUT_MS,
  );

  it(
    'finds a class the tokens do not define',
    async () => {
      const missing = await classesWithNoCss([
        { name: 'pe-12', file: 'x.tsx' },
        { name: 'p-3', file: 'x.tsx' },
        { name: 'type-body', file: 'x.tsx' },
        { name: 'normal:flex', file: 'x.tsx' },
      ]);
      expect(missing.map((use) => use.name)).toEqual(['pe-12']);
    },
    BUILD_TIMEOUT_MS,
  );
});
