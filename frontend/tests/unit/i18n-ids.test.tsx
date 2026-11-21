// Every message ID the code asks for exists in BOTH Fluent languages (plan 81, NFR-URD-1,
// NFR-URD-22). A missing ID shows as a raw ID on a screen or a print; in Urdu mode that is English
// text. Umar saw `print-partner-month-col-advances` as a column heading of an Urdu print.
//
// Four guards, each with a test:
// 1. The script (pnpm i18n:check, part two) reads the code and lists the IDs and ID templates that
//    no file defines. Here it runs on the real code, and on a small made-up code base to show that
//    it does find a gap.
// 2. The localization of the app records a lookup of a missing ID, and tests/setup.ts fails the
//    test that made it.
// 3. Every ID in the backend's recorded answers (tests/fixtures) is defined.
// 4. A message defined in two files stops the app at start; the check says so first.

import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { Localized } from '@fluent/react';
import { render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { checkI18n } from '../../../scripts/i18n-check-lib.mjs';
import {
  checkBackendIds,
  checkBackendTemplates,
  checkFrontendIds,
  checkMessageIds,
  definedIds,
} from '../../../scripts/i18n-ids-lib.mjs';
import { LANGUAGES } from '../../src/i18n';
import { createLocalization, takeMissingMessages } from '../../src/i18n/localization';
import { rawIdsIn } from '../helpers/raw-ids';
import { renderIn } from '../helpers/render';

const ROOT = path.resolve(import.meta.dirname, '../../..');
const I18N_DIR = path.join(ROOT, 'frontend', 'src', 'i18n');
/** The compiler reads the whole frontend: slower on a busy computer. */
const SLOW_MS = 240_000;

describe('the script behind pnpm i18n:check, on the real code', () => {
  it(
    'every ID and every ID template of the frontend and the backend is defined in ur and en',
    () => {
      expect(checkMessageIds(ROOT)).toEqual([]);
    },
    SLOW_MS,
  );

  it('knows the Rust templates: the report titles, the views, the posting types, the print columns', () => {
    const defined = definedIds(I18N_DIR);
    expect(checkBackendTemplates(ROOT, defined)).toEqual([]);
    expect(checkBackendIds(ROOT, defined)).toEqual([]);
    // The IDs of the families it checks are really there: the one Umar found, and a report title.
    for (const language of LANGUAGES) {
      expect(defined[language]?.has('print-partner-month-col-advances')).toBe(true);
      expect(defined[language]?.has('report-col-total')).toBe(true);
      expect(defined[language]?.has('report-r31-title')).toBe(true);
    }
  });
});

describe('the script finds a gap', () => {
  const tempDirs: string[] = [];

  afterEach(() => {
    for (const dir of tempDirs.splice(0)) rmSync(dir, { recursive: true, force: true });
  });

  /** A small repository: Fluent files, one TypeScript file and one Rust file. */
  function repository(files: Record<string, string>): string {
    const dir = mkdtempSync(path.join(tmpdir(), 'mandi-ids-'));
    tempDirs.push(dir);
    const all: Record<string, string> = {
      'frontend/tsconfig.json': JSON.stringify({
        compilerOptions: {
          strict: true,
          target: 'ES2023',
          lib: ['ES2023'],
          types: [],
          noEmit: true,
        },
        include: ['src'],
      }),
      'frontend/src/i18n/ur/demo.ftl': 'demo-kind-a = الف\ndemo-kind-b = ب\ndemo-ok = ٹھیک\n',
      'frontend/src/i18n/en/demo.ftl': 'demo-kind-a = A\ndemo-kind-b = B\ndemo-ok = Fine\n',
      ...files,
    };
    for (const [name, content] of Object.entries(all)) {
      mkdirSync(path.dirname(path.join(dir, name)), { recursive: true });
      writeFileSync(path.join(dir, name), content);
    }
    return dir;
  }

  const typescriptFrom = path.join(ROOT, 'frontend', 'package.json');

  it('an empty bindings folder is one clear error, not a list of every template', () => {
    const dir = repository({
      'frontend/src/bindings/.gitkeep': '',
      'frontend/src/kinds.ts':
        'export function words(kind: string): string {\n  return `demo-kind-${kind}`;\n}\n',
    });
    const errors = checkFrontendIds(dir, definedIds(path.join(dir, 'frontend/src/i18n')), {
      typescriptFrom,
    });
    expect(errors).toHaveLength(1);
    expect(errors[0]).toContain('frontend/src/bindings is empty');
    expect(errors[0]).toContain('pnpm gen:types');
  });

  it('a template whose key is missing in one language: the key is named', () => {
    const dir = repository({
      'frontend/src/kinds.ts':
        "type Kind = 'a' | 'b' | 'c';\nexport function words(kind: Kind): string {\n  return `demo-kind-${kind}`;\n}\n",
    });
    const errors = checkFrontendIds(dir, definedIds(path.join(dir, 'frontend/src/i18n')), {
      typescriptFrom,
    });
    expect(errors).toHaveLength(1);
    expect(errors[0]).toContain('"demo-kind-c"');
    expect(errors[0]).toContain('missing in ur and en');
  });

  it('the same template with all its keys defined passes', () => {
    const dir = repository({
      'frontend/src/kinds.ts':
        "type Kind = 'a' | 'b';\nexport function words(kind: Kind): string {\n  return `demo-kind-${kind}`;\n}\n",
    });
    const errors = checkFrontendIds(dir, definedIds(path.join(dir, 'frontend/src/i18n')), {
      typescriptFrom,
    });
    expect(errors).toEqual([]);
  });

  it('a key through .replaceAll and a helper is followed: demo-kind-c from a kind c_x is not hidden', () => {
    const dir = repository({
      'frontend/src/kinds.ts':
        "type Kind = 'a' | 'b' | 'c';\nfunction part(kind: string): string {\n  return kind.replaceAll('_', '-');\n}\nexport function words(kind: Kind): string {\n  return `demo-kind-${part(kind)}`;\n}\n",
    });
    const errors = checkFrontendIds(dir, definedIds(path.join(dir, 'frontend/src/i18n')), {
      typescriptFrom,
    });
    expect(errors.some((line) => line.includes('"demo-kind-c"'))).toBe(true);
  });

  it('a template whose keys cannot be worked out is an error, not silence', () => {
    const dir = repository({
      'frontend/src/kinds.ts':
        'export function words(kind: string): string {\n  return `demo-kind-${kind}`;\n}\nexport const used = words(String(Date.now()));\n',
    });
    const errors = checkFrontendIds(dir, definedIds(path.join(dir, 'frontend/src/i18n')), {
      typescriptFrom,
    });
    expect(errors).toHaveLength(1);
    expect(errors[0]).toContain('cannot be worked out');
  });

  it('a string given to getString that no file defines', () => {
    const dir = repository({
      'frontend/src/words.ts':
        "export const text = (id: string): string => id;\nexport const gone = text('demo-column-gone');\nexport const here = text('demo-ok');\n",
    });
    const errors = checkFrontendIds(dir, definedIds(path.join(dir, 'frontend/src/i18n')), {
      typescriptFrom,
    });
    expect(errors).toHaveLength(1);
    expect(errors[0]).toContain('"demo-column-gone" is missing in ur and en');
  });

  it('an ID the Rust code sends: a title_id or a Cell::Label that no file defines', () => {
    const dir = repository({
      'backend/crates/demo/src/lib.rs':
        'fn spec() {\n    Column { key: "x".to_owned(), title_id: "demo-col-gone".to_owned() };\n    Cell::Label { id: "demo-ok".to_owned() };\n    PrintFact { label_id: "novel-label-gone".to_owned() };\n    let file = "demo-not-an-id.pdf";\n}\n',
    });
    const errors = checkBackendIds(dir, definedIds(path.join(dir, 'frontend/src/i18n')));
    // One in a family the files have, one in a family nobody has yet (found by its place); the
    // defined ID and the file name are not.
    expect(errors).toHaveLength(2);
    expect(errors[0]).toContain('"demo-col-gone" is missing in ur and en');
    expect(errors[1]).toContain('"novel-label-gone" is missing in ur and en');
  });
});

describe('a message defined in two files', () => {
  it('is named by the check (the app would stop at start with "Attempt to override")', () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'mandi-i18n-dup-'));
    try {
      for (const language of LANGUAGES) {
        mkdirSync(path.join(dir, language), { recursive: true });
        writeFileSync(path.join(dir, language, 'a.ftl'), 'demo-same = One\n');
        writeFileSync(path.join(dir, language, 'b.ftl'), 'demo-same = Two\n');
      }
      expect(checkI18n(dir)).toEqual(
        expect.arrayContaining([
          'ur/b.ftl: demo-same is also defined in ur/a.ftl',
          'en/b.ftl: demo-same is also defined in en/a.ftl',
        ]),
      );
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});

describe('a message that refers to something nobody defines', () => {
  it('names the term and the message (the text would show "-gone" in the middle of a sentence)', () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'mandi-i18n-ref-'));
    try {
      for (const language of LANGUAGES) {
        mkdirSync(path.join(dir, language), { recursive: true });
        writeFileSync(
          path.join(dir, language, 'a.ftl'),
          '-known = Word\ndemo-a = { -known } and { -gone } and { demo-nothing }\ndemo-b = { demo-a }\n',
        );
      }
      expect(checkI18n(dir)).toEqual(
        expect.arrayContaining([
          'ur/a.ftl: demo-a refers to -gone, which is not defined',
          'ur/a.ftl: demo-a refers to demo-nothing, which is not defined',
          'en/a.ftl: demo-a refers to -gone, which is not defined',
          'en/a.ftl: demo-a refers to demo-nothing, which is not defined',
        ]),
      );
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});

describe('the localization of the app records a message that is not there', () => {
  it('a lookup with getString and an element with Localized are both recorded, in the language asked', () => {
    takeMissingMessages();
    const ur = createLocalization('ur');
    expect(ur.getString('no-such-message-one')).toBe('no-such-message-one');
    createLocalization('en').getString('no-such-message-two');
    renderIn(
      'ur',
      <Localized id="no-such-message-three">
        <span />
      </Localized>,
    );
    expect(takeMissingMessages().sort()).toEqual([
      'en: no-such-message-two',
      'ur: no-such-message-one',
      'ur: no-such-message-three',
    ]);
    // Taken: the record is empty again, so this test does not fail the check of tests/setup.ts.
    expect(takeMissingMessages()).toEqual([]);
  });

  it('a lookup that gives a fallback word is not a gap: the code expects some IDs to be missing', () => {
    takeMissingMessages();
    const text = createLocalization('ur').getString(
      'a-code-the-app-has-no-words-for',
      null,
      'other',
    );
    expect(text).toBe('other');
    expect(takeMissingMessages()).toEqual([]);
  });

  it('a message that exists is not recorded', () => {
    takeMissingMessages();
    render(<span>{createLocalization('ur').getString('common-save')}</span>);
    expect(takeMissingMessages()).toEqual([]);
  });
});

describe('a message ID drawn as text on a page', () => {
  it('is found by the check of tests/setup.ts, and a name, a date or a plain word is not', () => {
    const page = document.createElement('div');
    for (const text of [
      'print-partner-month-col-advances',
      'Akram Toll Tax',
      '10-10-2026',
      'per-unit',
      '0020',
    ]) {
      const element = document.createElement('span');
      element.textContent = text;
      page.append(element);
    }
    document.body.append(page);
    expect(rawIdsIn(page)).toEqual(['print-partner-month-col-advances']);
    // Taken down here, so the check after this test finds an empty page.
    page.remove();
    expect(rawIdsIn(document.body)).toEqual([]);
  });
});

describe('the answers the backend recorded (tests/fixtures)', () => {
  /** Every string in the JSON that is written as a message ID of a family the Fluent files have. */
  function idsIn(value: unknown, found: Set<string>, families: Set<string>): void {
    if (typeof value === 'string') {
      if (
        /^[a-z][a-z0-9]*(?:-[a-z0-9_]+)+$/.test(value) &&
        families.has(value.split('-')[0] ?? '')
      ) {
        found.add(value);
      }
    } else if (Array.isArray(value)) {
      for (const item of value) idsIn(item, found, families);
    } else if (value !== null && typeof value === 'object') {
      for (const item of Object.values(value)) idsIn(item, found, families);
    }
  }

  function jsonFiles(dir: string, out: string[] = []): string[] {
    for (const name of readdirSync(dir)) {
      const file = path.join(dir, name);
      if (statSync(file).isDirectory()) jsonFiles(file, out);
      else if (name.endsWith('.json')) out.push(file);
    }
    return out;
  }

  it('every message ID in them (a title, a column heading, a label, a kind) exists in ur and en', () => {
    const families = new Set<string>();
    const defined = definedIds(I18N_DIR);
    for (const id of defined['en'] ?? []) {
      const first = /^([a-z0-9]+)-/.exec(id)?.[1];
      if (first !== undefined) families.add(first);
    }
    const found = new Set<string>();
    for (const file of jsonFiles(path.join(ROOT, 'frontend', 'tests', 'fixtures'))) {
      idsIn(JSON.parse(readFileSync(file, 'utf8')) as unknown, found, families);
    }
    const missing = [...found]
      .filter((id) => !LANGUAGES.every((language) => defined[language]?.has(id) === true))
      .sort();
    // Words that are only the name of a kind in a recorded answer (not shown as words).
    const KINDS_NOT_MESSAGES = new Set<string>([]);
    expect(missing.filter((id) => !KINDS_NOT_MESSAGES.has(id))).toEqual([]);
  });
});
