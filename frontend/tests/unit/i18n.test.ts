// Fluent files: the i18n check (NFR-URD-22), the glossary (PRD Section 3) and error messages.

import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, describe, expect, it } from 'vitest';
import { checkI18n } from '../../../scripts/i18n-check-lib.mjs';
import errorCodeSource from '../../src/bindings/ErrorCode.ts?raw';
import { LANGUAGES, bundles } from '../../src/i18n';
import { errorMessageId } from '../../src/shared/api';
import type { ErrorCode } from '../../src/bindings/ErrorCode';

const I18N_DIR = path.resolve(import.meta.dirname, '../../src/i18n');

const ftlSources = {
  ur: import.meta.glob<string>('../../src/i18n/ur/*.ftl', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
  en: import.meta.glob<string>('../../src/i18n/en/*.ftl', {
    query: '?raw',
    import: 'default',
    eager: true,
  }),
};

describe('pnpm i18n:check', () => {
  const tempDirs: string[] = [];

  /** Writes a small ur/en pair of Fluent folders and returns the folder that holds them. */
  function fixture(files: Record<string, string>): string {
    const dir = mkdtempSync(path.join(tmpdir(), 'mandi-i18n-'));
    tempDirs.push(dir);
    for (const [name, content] of Object.entries(files)) {
      mkdirSync(path.dirname(path.join(dir, name)), { recursive: true });
      writeFileSync(path.join(dir, name), content);
    }
    return dir;
  }

  afterEach(() => {
    for (const dir of tempDirs.splice(0)) rmSync(dir, { recursive: true, force: true });
  });

  it('passes on the real files', () => {
    expect(checkI18n(I18N_DIR)).toEqual([]);
  });

  it('the pnpm entry always runs the check and prints its result (reviewer finding 5)', () => {
    const entry = path.resolve(import.meta.dirname, '../../../scripts/i18n-check.mjs');
    const run = spawnSync(process.execPath, [entry], { encoding: 'utf8' });
    expect(run.status).toBe(0);
    expect(run.stdout).toContain('i18n:check passed');
  });

  it('passes on a matching pair', () => {
    const dir = fixture({
      'ur/common.ftl': 'common-save = محفوظ کریں\n',
      'en/common.ftl': 'common-save = Save\n',
    });
    expect(checkI18n(dir)).toEqual([]);
  });

  it('fails on a key missing in one language', () => {
    const dir = fixture({
      'ur/common.ftl': 'common-save = محفوظ کریں\ncommon-cancel = منسوخ کریں\n',
      'en/common.ftl': 'common-save = Save\n',
    });
    expect(checkI18n(dir)).toEqual(['en/common.ftl: missing common-cancel (present in ur)']);
  });

  it('fails on an empty key', () => {
    const dir = fixture({
      'ur/common.ftl': 'common-save = محفوظ کریں\ncommon-cancel = { "" }\n',
      'en/common.ftl': 'common-save = Save\ncommon-cancel = Cancel\n',
    });
    expect(checkI18n(dir)).toEqual(['ur/common.ftl: common-cancel is empty']);
  });

  it('fails on a key with no value at all (unreadable Fluent)', () => {
    const dir = fixture({
      'ur/common.ftl': 'common-save = محفوظ کریں\ncommon-cancel =\n',
      'en/common.ftl': 'common-save = Save\ncommon-cancel = Cancel\n',
    });
    const errors = checkI18n(dir);
    expect(errors).toContain('ur/common.ftl: cannot be read near "common-cancel ="');
    expect(errors).toContain('ur/common.ftl: missing common-cancel (present in en)');
  });

  it('fails on a file missing in one language', () => {
    const dir = fixture({
      'ur/common.ftl': 'common-save = محفوظ کریں\n',
      'en/common.ftl': 'common-save = Save\n',
      'ur/bikri.ftl': 'bikri-title = بکری\n',
    });
    expect(checkI18n(dir)).toEqual(['en/bikri.ftl: file is missing (present in ur)']);
  });

  it('fails on text left untranslated, unless the ID is allowed', () => {
    const files = {
      'ur/common.ftl': 'common-save = Save\n',
      'en/common.ftl': 'common-save = Save\n',
    };
    expect(checkI18n(fixture(files))).toEqual([
      'common.ftl: common-save is the same in ur and en; translate it or add it to the allow list',
    ]);
    expect(checkI18n(fixture(files), { identicalAllowed: new Set(['common-save']) })).toEqual([]);
  });

  it('accepts a message made only of references, filled from each glossary', () => {
    const dir = fixture({
      'ur/glossary.ftl': '-bikri = بکری\nglossary-bikri = { -bikri }\n',
      'en/glossary.ftl': '-bikri = Sale\nglossary-bikri = { -bikri }\n',
    });
    expect(checkI18n(dir)).toEqual([]);
  });
});

// PRD Section 3, row by row: the Urdu column and the English column (FR-7.37).
const PRD_GLOSSARY: readonly (readonly [id: string, ur: string, en: string])[] = [
  ['arhti', 'آڑھتی', 'Commission Agent (Arhti)'],
  ['index', 'انڈکس', 'Index'],
  ['beopari', 'بیوپاری', 'Seller'],
  ['zamindar', 'زمیندار', 'Zamindar'],
  ['buzgar', 'بزگر', 'Grower'],
  ['gahak', 'گاہک', 'Buyer'],
  ['kharidar', 'خریدار', 'Buyer'],
  ['pukhta-khata', 'پختہ کھاتہ', 'Pukhta Khata'],
  ['nag', 'نگ', 'Nag'],
  ['bikri', 'بکری', 'Sale'],
  ['bikri-book', 'بکری بک', 'Sale Book'],
  ['lot', 'لاٹ', 'Lot'],
  ['qism-maal', 'قسم مال', 'Item'],
  ['packing', 'پیکنگ', 'Packing'],
  ['marka', 'مارکہ', 'Mark'],
  ['adda', 'اڈا', 'Adda'],
  ['kham-bikri', 'خام بکری', 'Gross Sale'],
  ['safi-bikri', 'صافی بکری', 'Net Sale'],
  ['commission', 'کمیشن', 'Commission'],
  ['laga', 'لاگا', 'Laga'],
  ['mazdoori', 'مزدوری', 'Labour'],
  ['munshiana', 'منشیانہ', 'Munshiana'],
  ['kiraya', 'کرایہ', 'Freight'],
  ['kiraya-store', 'کرایہ سٹور', 'Store Rent'],
  ['naqd', 'نقد', 'Cash Paid'],
  ['phone', 'فون', 'Phone Charge'],
  ['naqdi-mandi', 'نقدی منڈی', 'Naqdi Mandi'],
  ['rate-farq', 'ریٹ فرق', 'Rate Difference'],
  ['udhari', 'ادھاری', 'Credit'],
  ['naqdi', 'نقدی', 'Cash'],
  ['rokar', 'روکڑ', 'Cash Book'],
  ['wasooli', 'وصولی', 'Receipt'],
  ['agrahi', 'اگراہی', 'Collection'],
  ['adaigi', 'ادائیگی', 'Payment'],
  ['peshgi-khata', 'پیشگی کھاتہ', 'Advance'],
  ['general', 'جنرل', 'Journal'],
  ['kharcha', 'خرچہ', 'Expense'],
  ['dukan-kharcha', 'دکان خرچہ', 'Shop Expense'],
  ['zati-kharcha', 'ذاتی خرچہ', 'Personal Drawing'],
  ['stock', 'سٹاک', 'Unsold Stock'],
  ['baqi-maal', 'باقی مال', 'Unsold Stock'],
  ['shirakat-dar', 'شراکت دار', 'Partner'],
  ['hissa', 'حصہ', 'Share'],
  ['sarmaya', 'سرمایہ', 'Capital'],
  ['tajziat', 'تجزیات', 'Analytics'],
  ['roznamcha', 'روزنامچہ', 'Day Book'],
  ['khata', 'کھاتہ', 'Ledger'],
  ['fard-hisab', 'فرد حساب', 'Statement'],
  ['banaam', 'بنام', 'Debit'],
  ['jama', 'جمع', 'Credit'],
  ['meezan', 'میزان', 'Total'],
  ['trial-balance', 'ٹرائل بیلنس', 'Trial Balance'],
];

/** Formats a message with Fluent's bidi marks removed, failing the test on any Fluent error. */
function format(
  language: (typeof LANGUAGES)[number],
  id: string,
  args: Record<string, string> = {},
): string {
  const bundle = bundles[language];
  const message = bundle.getMessage(id);
  if (message?.value == null) throw new Error(`${language}: no message ${id}`);
  const errors: Error[] = [];
  const text = bundle.formatPattern(message.value, args, errors);
  expect(errors, `${language} ${id}`).toEqual([]);
  return text.replace(/[⁨⁩]/g, '');
}

describe('glossary.ftl', () => {
  it.each(PRD_GLOSSARY)('glossary-%s has the PRD words: %s / %s', (id, ur, en) => {
    expect(format('ur', `glossary-${id}`)).toBe(ur);
    expect(format('en', `glossary-${id}`)).toBe(en);
  });
});

describe('Fluent bundles', () => {
  it.each(LANGUAGES)('every %s message formats without errors', (language) => {
    for (const source of Object.values(ftlSources[language])) {
      const ids = [...source.matchAll(/^([a-z][\w-]*) =/gm)].map((match) => match[1] ?? '');
      const args = Object.fromEntries(
        [...source.matchAll(/\$([a-zA-Z][\w-]*)/g)].map((match) => [match[1] ?? '', '1']),
      );
      for (const id of ids) {
        expect(format(language, id, args).trim(), `${language} ${id}`).not.toBe('');
      }
    }
  });

  it('has a message in both languages for every backend error code', () => {
    const codes = [...errorCodeSource.matchAll(/["'](ERR_[A-Z0-9_]+)["']/g)].map(
      (match) => match[1] as ErrorCode,
    );
    expect(codes.length).toBeGreaterThan(0);
    for (const code of codes) {
      for (const language of LANGUAGES) {
        expect(bundles[language].hasMessage(errorMessageId(code)), `${language} ${code}`).toBe(
          true,
        );
      }
    }
  });
});
