// The copies of the backend's real answers are the backend's (plan 17, 17b). The backend writes
// them in its own tests (backend/crates/services/tests/fixtures) and the frontend copies them here
// and never edits them. This compares each copy with the original as parsed JSON, so a stale copy
// fails here and not later on a screen. It compares the content and not the spaces: `prettier`
// formats the copies (the repository checks it), and the backend writes some of its files with
// expanded arrays, so a few copies differ from the original in white space only.

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

const HERE = import.meta.dirname;
const COPIES = path.join(HERE, '..', 'fixtures');
const ORIGINALS = path.join(
  HERE,
  '..',
  '..',
  '..',
  'backend',
  'crates',
  'services',
  'tests',
  'fixtures',
);

/** The fixture files that are in both folders. A file only in one of them is not compared. */
function sharedFiles(): string[] {
  if (!existsSync(ORIGINALS)) return [];
  const originals = new Set(readdirSync(ORIGINALS));
  return readdirSync(COPIES).filter((name) => name.endsWith('.json') && originals.has(name));
}

describe('the fixture copies hold what the backend wrote', () => {
  const files = sharedFiles();

  it('there are copies to compare', () => {
    expect(files.length).toBeGreaterThan(100);
  });

  it.each(files)('%s has the content of the backend’s file', (name) => {
    const copy: unknown = JSON.parse(readFileSync(path.join(COPIES, name), 'utf8'));
    const original: unknown = JSON.parse(readFileSync(path.join(ORIGINALS, name), 'utf8'));
    expect(copy).toEqual(original);
  });
});
