// Plan 93 T4 (Umar, 22-11-2026): the page and the panels of a screen scroll by wheel, trackpad, keys and drag but show
// no scroll line, on Mac (WebKit) and on Windows (WebView2, Chromium). It is one shared utility, `no-scrollbar`, in
// `styles/globals.css`, with both the standard property and the WebKit pseudo-element. A data grid (AG Grid) and a
// wide table that scrolls sideways keep their own bars: the utility is never used on them. A browser draws the
// bar, so the rule itself and where it is used are what a unit test can hold.

import { readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

const SRC = path.resolve(__dirname, '../../src');
const css = readFileSync(path.join(SRC, 'styles/globals.css'), 'utf8');

/** Every source file under a folder, as text, with its path. */
function sources(folder: string): { file: string; text: string }[] {
  return readdirSync(folder).flatMap((name) => {
    const file = path.join(folder, name);
    if (statSync(file).isDirectory()) return sources(file);
    return /\.(tsx?|css)$/.test(name) ? [{ file, text: readFileSync(file, 'utf8') }] : [];
  });
}

describe('the no-scrollbar utility', () => {
  it('is one rule that hides the bar in Firefox-style and WebKit-style engines alike', () => {
    const block = /@utility no-scrollbar \{([\s\S]*?)\n\}/.exec(css)?.[1] ?? '';
    expect(block).toContain('scrollbar-width: none');
    expect(block).toMatch(/&::-webkit-scrollbar\s*\{\s*display: none;/);
  });

  it('is on the main screen area and on the inner panels of the pages named in the plan', () => {
    const used = sources(SRC)
      .filter(({ text }) => text.includes('no-scrollbar'))
      .map(({ file }) => path.relative(SRC, file));
    // The account form had a panel of its own; from plan 99 it scrolls in the screen of the shell like
    // every other form (AppShell holds the utility), so no card is cut off.
    for (const expected of [
      'app/layout/AppShell.tsx',
      'features/settings/pages/SettingsPage.tsx',
      'features/bikri/pages/BikriBookPage.tsx',
    ]) {
      expect(used).toContain(expected);
    }
  });

  it('is never used on a data grid or a table that scrolls sideways, which keep their own bars', () => {
    const grid = sources(path.join(SRC, 'shared/grid')).filter(({ text }) =>
      text.includes('no-scrollbar'),
    );
    expect(grid).toEqual([]);
    const tables = sources(path.join(SRC, 'shared/ui')).filter(
      ({ file, text }) =>
        /Table|PagedList|Grid/.test(path.basename(file)) && text.includes('no-scrollbar'),
    );
    expect(tables).toEqual([]);
    // Nothing that scrolls sideways has it: `overflow-x-auto` and the utility never share a class list.
    for (const { file, text } of sources(SRC)) {
      for (const match of text.matchAll(/className=[`"'{][^\n]*no-scrollbar[^\n]*/g)) {
        expect(match[0], path.relative(SRC, file)).not.toContain('overflow-x-auto');
      }
    }
  });
});
