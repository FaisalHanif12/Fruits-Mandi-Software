// @vitest-environment node
// Text contrast of the design tokens (NFR-USE-21, WCAG AA: 4.5 to 1 for text, 3 to 1 for the edge
// of a control and the focus outline). It reads the colours from tokens.css, the only place they
// are written, and checks every pair the screens put text on (D1+D2 review, finding 37).

import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

const TOKENS = fs.readFileSync(
  path.resolve(import.meta.dirname, '../../src/styles/tokens.css'),
  'utf8',
);

/** `--color-ink: #1c1f23;` to { ink: '#1c1f23' }. Colours given as rgb() are left out. */
function colours(): Record<string, string> {
  const found: Record<string, string> = {};
  for (const match of TOKENS.matchAll(/--color-([\w-]+):\s*(#[0-9a-fA-F]{3,6})\s*;/g)) {
    const name = match[1];
    const hex = match[2];
    if (name !== undefined && hex !== undefined) found[name] = hex;
  }
  return found;
}

const CHANNELS = 3;
const FULL = 255;

/** The red, green and blue of a `#rgb` or `#rrggbb` colour, 0 to 255. */
function rgbOf(hex: string): [number, number, number] {
  const digits = hex.slice(1);
  const full = digits.length === CHANNELS ? digits.replace(/./g, (digit) => digit + digit) : digits;
  const parts = [0, 2, 4].map((at) => Number.parseInt(full.slice(at, at + 2), 16));
  return [parts[0] ?? 0, parts[1] ?? 0, parts[2] ?? 0];
}

/** Relative luminance (WCAG 2.x). */
function luminance(hex: string): number {
  const [r, g, b] = rgbOf(hex).map((channel) => {
    const value = channel / FULL;
    return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  }) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function ratio(a: string, b: string): number {
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (light + 0.05) / (dark + 0.05);
}

const COLOURS = colours();
const colour = (name: string): string => {
  const hex = COLOURS[name];
  if (hex === undefined) throw new Error(`no colour token ${name}`);
  return hex;
};

const TEXT_MIN = 4.5;
const UI_MIN = 3;

/** Text colour, then the grounds it is drawn on. */
const TEXT_ON: [string, string[]][] = [
  ['ink', ['bg', 'surface', 'surface-sunken', 'surface-readonly', 'row-current', 'primary-soft']],
  ['ink-muted', ['bg', 'surface', 'surface-sunken', 'surface-readonly', 'row-current']],
  ['ink-placeholder', ['surface', 'surface-readonly', 'bg']],
  ['on-primary', ['primary', 'primary-chip']],
  ['nav-ink', ['nav', 'nav-active']],
  ['primary', ['bg', 'surface', 'primary-soft', 'surface-sunken']],
  ['money-in', ['bg', 'surface', 'money-in-soft']],
  ['due', ['bg', 'surface', 'due-soft']],
  ['warn', ['bg', 'surface', 'warn-soft']],
  ['money-in-on-nav', ['nav']],
  ['due-on-nav', ['nav']],
];

describe('text contrast of the colour tokens (NFR-USE-21)', () => {
  for (const [text, grounds] of TEXT_ON) {
    it.each(grounds)(`${text} on %s is at least 4.5 to 1`, (ground) => {
      expect(ratio(colour(text), colour(ground))).toBeGreaterThanOrEqual(TEXT_MIN);
    });
  }
});

describe('edges and focus (WCAG 1.4.11)', () => {
  it.each(['bg', 'surface'])('the edge of a control is at least 3 to 1 on %s', (ground) => {
    expect(ratio(colour('border-control'), colour(ground))).toBeGreaterThanOrEqual(UI_MIN);
  });

  it.each(['bg', 'surface', 'surface-sunken'])(
    'the focus outline is at least 3 to 1 on %s',
    (ground) => {
      expect(ratio(colour('focus'), colour(ground))).toBeGreaterThanOrEqual(UI_MIN);
    },
  );
});

describe('the contrast helper', () => {
  it('gives 21 to 1 for black on white and 1 to 1 for a colour on itself', () => {
    expect(ratio('#000', '#fff')).toBeCloseTo(21, 5);
    expect(ratio('#1d3d6e', '#1d3d6e')).toBeCloseTo(1, 5);
  });
});
