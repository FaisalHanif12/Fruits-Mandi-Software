// Plan 39 R5 (Code Standards F18): a column of figures in a plain table has the width of its kind.
// Every `kind: 'number'` column of a `Table` names its `size` (money, qty, date, code ...), so a money
// column is as wide in the saved sale as in the dues list or the advance history. This reads the
// source of every screen, because a new table with an unsized figure column would be wide in one
// place and narrow in another, and no test of one screen would see it.

import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

const SRC = path.resolve(import.meta.dirname, '../../src');

function sourceFiles(dir: string): string[] {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) return sourceFiles(full);
    return entry.name.endsWith('.tsx') ? [full] : [];
  });
}

describe('the figure columns of the shared table', () => {
  const files = sourceFiles(SRC).filter((file) => {
    const text = fs.readFileSync(file, 'utf8');
    return text.includes('TableColumn<') && text.includes("kind: 'number'");
  });

  it('is used by some screens (the scan finds the tables)', () => {
    expect(files.length).toBeGreaterThan(10);
  });

  it.each(files.map((file) => [path.relative(SRC, file), file]))(
    '%s names the size of every figure column',
    (_name, file) => {
      const text = fs.readFileSync(file, 'utf8');
      const missing: string[] = [];
      for (const found of text.matchAll(/kind: 'number',\s*(size: '(\w+)',)?/g)) {
        if (found[1] === undefined) {
          const line = text.slice(0, found.index).split('\n').length;
          missing.push(`line ${String(line)}`);
        }
      }
      expect(missing).toEqual([]);
    },
  );
});
