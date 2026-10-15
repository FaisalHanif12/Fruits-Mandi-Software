// @vitest-environment node
// Comments in the code cannot switch the guards off (reviewer finding 1, plan 01).
// These run the real ESLint and Stylelint configs of the frontend.

import { ESLint } from 'eslint';
import path from 'node:path';
import stylelint from 'stylelint';
import { describe, expect, it } from 'vitest';

const FRONTEND_DIR = path.resolve(import.meta.dirname, '../..');
// A real file in the TypeScript project, so the type-aware rules can run; its text is replaced.
const APP_FILE = path.join(FRONTEND_DIR, 'src/shared/ui/Code.tsx');
const LINT_TIMEOUT_MS = 60_000;

describe('ESLint config', () => {
  it(
    'ignores eslint-disable comments, so every mandi rule still reports',
    async () => {
      const code = [
        "import type { Paisa } from '../format/money';",
        'declare const a: Paisa;',
        '/* eslint-disable */',
        '// eslint-disable-next-line mandi/no-paisa-arithmetic',
        'export const total = a + a;',
        '// eslint-disable-next-line mandi/no-jsx-literals',
        'export const Title = () => <h1>Mandi Software</h1>;',
        "export const cls = 'ml-2'; // eslint-disable-line mandi/no-physical-direction",
        "export const pad = 'p-[13px]'; // eslint-disable-line",
        '',
      ].join('\n');
      const eslint = new ESLint({ cwd: FRONTEND_DIR });
      const [result] = await eslint.lintText(code, { filePath: APP_FILE });
      const ruleIds = (result?.messages ?? []).map((message) => message.ruleId);
      expect(ruleIds).toEqual(
        expect.arrayContaining([
          'mandi/no-paisa-arithmetic',
          'mandi/no-jsx-literals',
          'mandi/no-physical-direction',
          'mandi/no-raw-design-values',
        ]),
      );
      // Each disable comment is also reported, so `--max-warnings 0` fails on it.
      const ignoredComments = (result?.messages ?? []).filter((message) =>
        message.message.includes('noInlineConfig'),
      );
      expect(ignoredComments.length).toBeGreaterThanOrEqual(4);
    },
    LINT_TIMEOUT_MS,
  );
});

describe('ESLint config: no floating point and no switched-off types in the app code', () => {
  /** The rules the real config reports for a piece of app code. */
  async function rulesFor(code: string): Promise<(string | null)[]> {
    const eslint = new ESLint({ cwd: FRONTEND_DIR });
    const [result] = await eslint.lintText(code, { filePath: APP_FILE });
    return (result?.messages ?? []).map((message) => message.ruleId);
  }

  it.each([
    ['parseFloat', "export const a = parseFloat('1.5');", 'no-restricted-globals'],
    ['Number.parseFloat', "export const a = Number.parseFloat('1.5');", 'no-restricted-properties'],
    ['Math.round', 'export const a = Math.round(1.5);', 'no-restricted-properties'],
    ['toFixed', 'export const a = (1.5).toFixed(2);', 'no-restricted-properties'],
    ['as unknown as', 'export const a = 5 as unknown as string;', 'no-restricted-syntax'],
  ])(
    'reports %s',
    async (_name, code, rule) => {
      expect(await rulesFor(code)).toContain(rule);
    },
    LINT_TIMEOUT_MS,
  );

  it(
    'allows parseInt for whole pixel sizes',
    async () => {
      const rules = await rulesFor("export const a = Number.parseInt('44px', 10);");
      expect(rules).not.toContain('no-restricted-properties');
      expect(rules).not.toContain('no-restricted-globals');
    },
    LINT_TIMEOUT_MS,
  );
});

describe('Stylelint config', () => {
  it.each([
    '/* stylelint-disable */\n.demo {\n  margin-left: var(--space-2);\n}\n',
    '.demo {\n  margin-left: var(--space-2); /* stylelint-disable-line */\n}\n',
    '.demo {\n  /* stylelint-disable-next-line csstools/use-logical */\n  margin-left: var(--space-2);\n}\n',
  ])('ignores the disable comment in %j, so margin-left still fails', async (code) => {
    const { results } = await stylelint.lint({
      code,
      codeFilename: path.join(FRONTEND_DIR, 'src/styles/demo.css'),
      configFile: path.join(FRONTEND_DIR, 'stylelint.config.js'),
    });
    const rules = (results[0]?.warnings ?? []).map((warning) => warning.rule);
    expect(rules).toContain('csstools/use-logical');
  });
});

// Reviewer recheck finding B: shorthands listed in physical order (top, right, bottom, left).
describe('Stylelint: physical shorthands', () => {
  /** The rules the real config reports for one declaration. */
  async function rulesFor(declaration: string): Promise<(string | undefined)[]> {
    const { results } = await stylelint.lint({
      code: `.demo {\n  ${declaration}\n}\n`,
      codeFilename: path.join(FRONTEND_DIR, 'src/styles/demo.css'),
      configFile: path.join(FRONTEND_DIR, 'stylelint.config.js'),
    });
    return (results[0]?.warnings ?? []).map((warning) => warning.rule);
  }

  it.each([
    'margin: 0 var(--space-1) 0 0;',
    'margin: 0 var(--space-1) 0;',
    'padding: 0 0 0 var(--space-2);',
    'scroll-margin: 0 var(--space-1) 0 0;',
    'scroll-padding: 0 0 0 var(--space-2);',
    'inset: 0 auto auto 0;',
    'inset: 0 auto;',
    'inset: 0 auto 0;',
    'border-width: 0 var(--border-w) 0 0;',
    'border-style: none solid none none;',
    'border-color: transparent var(--color-border) transparent transparent;',
    'border-radius: var(--radius-2) 0 0 0;',
    'border-radius: var(--radius-2) 0;',
  ])('fails on %s', async (declaration) => {
    expect(await rulesFor(declaration)).toContain('mandi/no-physical-shorthand');
  });

  it.each([
    'margin: 0;',
    'margin: var(--space-2) auto;',
    'padding: var(--space-1) var(--space-2);',
    'margin: calc(var(--space-1) + var(--space-2));',
    'inset: 0;',
    'border-width: var(--border-w);',
    'border-radius: var(--radius-2);',
    'margin-block: 0 var(--space-2);',
    'padding-inline: var(--space-2) var(--space-3);',
    'inset-inline: 0 auto;',
  ])('allows %s', async (declaration) => {
    expect(await rulesFor(declaration)).toEqual([]);
  });
});

// Reviewer recheck 2, finding E: the keywords left and right in position properties.
describe('Stylelint: physical position keywords', () => {
  async function rulesFor(declaration: string): Promise<(string | undefined)[]> {
    const { results } = await stylelint.lint({
      code: `.demo {\n  ${declaration}\n}\n`,
      codeFilename: path.join(FRONTEND_DIR, 'src/styles/demo.css'),
      configFile: path.join(FRONTEND_DIR, 'stylelint.config.js'),
    });
    return (results[0]?.warnings ?? []).map((warning) => warning.rule);
  }

  it.each([
    'background-position: left;',
    'background-position: right top;',
    'background-position-x: left;',
    'transform-origin: left top;',
    'transform-origin: right;',
    'object-position: left center;',
    'mask-position: right;',
    '-webkit-mask-position: left;',
    // 01-M2: the shorthands.
    'background: url("a.png") no-repeat right top;',
    'background: var(--color-bg) left center;',
    'background: linear-gradient(to right, var(--color-bg), var(--color-surface));',
    'mask: url("m.svg") left / contain;',
    '-webkit-mask: url("m.svg") no-repeat right;',
  ])('fails on %s', async (declaration) => {
    expect(await rulesFor(declaration)).toContain('declaration-property-value-disallowed-list');
  });

  it.each([
    'background-position: center;',
    'transform-origin: center top;',
    'object-position: center;',
    'background-position: top center;',
    'background: var(--color-bg);',
    'background: url("a.png") no-repeat center;',
    'background: linear-gradient(to bottom, var(--color-bg), var(--color-surface));',
    'mask: url("m.svg") center / contain;',
  ])('allows %s', async (declaration) => {
    expect(await rulesFor(declaration)).toEqual([]);
  });
});
