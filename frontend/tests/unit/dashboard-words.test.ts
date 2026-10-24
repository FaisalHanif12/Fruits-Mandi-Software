// The words of the attention list and the dashboard (round 9, PRD FR-7.44): every kind has its
// message in Urdu and in English, with and without an age where it has both, each formats with no
// error for the figures the backend sends, and the English words use the right singular and plural.
// A kind that has no words would show a raw message ID to the Owner (NFR-USE-14).

import { FluentBundle, FluentResource } from '@fluent/bundle';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import type { AttentionItem } from '../../src/bindings/AttentionItem';
import type { AttentionKind } from '../../src/bindings/AttentionKind';
import {
  BACKUP_NEVER_MESSAGE,
  messageOfKind,
  wordsOfItem,
} from '../../src/features/dashboard/attention';
import { destinationOfFirst, destinationOfKind } from '../../src/features/dashboard/destinations';
import { formatPaisa, type Paisa } from '../../src/shared/format';

const KINDS: readonly AttentionKind[] = [
  'drafts',
  'open_days',
  'stock_waiting',
  'over_credit_limit',
  'dues_old',
  'payables_old',
  'backup_old',
  'loans_overdue',
  'takes_overdue',
  'names_missing',
];

function bundleOf(language: 'ur' | 'en'): FluentBundle {
  const bundle = new FluentBundle(language, { useIsolating: false });
  const folder = path.resolve(__dirname, '../../src/i18n', language);
  for (const file of readdirSync(folder).filter((name) => name.endsWith('.ftl'))) {
    bundle.addResource(new FluentResource(readFileSync(path.join(folder, file), 'utf8')));
  }
  return bundle;
}

const BUNDLES = { ur: bundleOf('ur'), en: bundleOf('en') } as const;

function item(kind: AttentionKind, overrides: Partial<AttentionItem> = {}): AttentionItem {
  return {
    kind,
    count: 3,
    amount_paisa: null,
    qty: 12,
    days: 5,
    first: null,
    ...overrides,
  };
}

const money = (amount: Paisa) => formatPaisa(amount, { grouping: 'international' });

function say(language: 'ur' | 'en', id: string, vars: Record<string, string | number>): string {
  const bundle = BUNDLES[language];
  const pattern = bundle.getMessage(id)?.value;
  if (pattern == null) throw new Error(`${language}: no message ${id}`);
  const errors: Error[] = [];
  const text = bundle.formatPattern(pattern, vars, errors);
  expect(errors, `${language} ${id}`).toEqual([]);
  return text.replace(/<\/?(num|name|money|key|unit)>/g, '');
}

describe.each(['ur', 'en'] as const)('the attention words (%s)', (language) => {
  it.each(KINDS)('%s has words, with its age and without it, and no raw ID shows', (kind) => {
    for (const days of [null, 5]) {
      const words = wordsOfItem(item(kind, { days }), money);
      const text = say(language, words.id, words.vars);
      expect(text.length).toBeGreaterThan(0);
      expect(text).not.toMatch(/attention-|\{|\}/);
    }
  });

  it('a backup that was never made has its own words, not an age', () => {
    const words = wordsOfItem(item('backup_old', { days: null }), money);
    expect(words.id).toBe(BACKUP_NEVER_MESSAGE);
    expect(say(language, words.id, words.vars).length).toBeGreaterThan(0);
  });

  it('the bell label, the title and the empty state have words', () => {
    for (const id of [
      'attention-title',
      'attention-empty-title',
      'attention-empty-help',
      'attention-load-failed',
    ]) {
      expect(say(language, id, {}).length).toBeGreaterThan(0);
    }
    expect(say(language, 'attention-bell-label', { count: 10 })).toContain('10');
  });
});

describe('the English singular and plural', () => {
  it('says one draft and two drafts, one day and two days', () => {
    expect(say('en', 'attention-drafts', { count: 1 })).toBe('1 unsaved draft');
    expect(say('en', 'attention-drafts', { count: 2 })).toBe('2 unsaved drafts');
    expect(say('en', 'attention-open-days', { count: 1 })).toBe('1 earlier day is not closed');
    expect(say('en', 'attention-open-days', { count: 8 })).toBe('8 earlier days are not closed');
    expect(say('en', 'attention-backup-old', { days: 1 })).toContain('1 day old');
    expect(say('en', 'attention-backup-old', { days: 2 })).toContain('2 days old');
    const loan = (count: number, days: number) =>
      say('en', 'attention-loans-overdue-aged', { count, days, amount: '2,000' });
    expect(loan(1, 1)).toContain('1 day past');
    expect(loan(2, 3)).toContain('up to 3 days');
  });
});

describe('the words only name a message the wrapper makes', () => {
  it('the id of a kind is attention-<kind with dashes>', () => {
    expect(messageOfKind('over_credit_limit')).toBe('attention-over-credit-limit');
    expect(messageOfKind('names_missing')).toBe('attention-names-missing');
  });

  it('the aged message is chosen only when the backend names an age', () => {
    expect(wordsOfItem(item('dues_old', { days: 41 }), money).id).toBe('attention-dues-old-aged');
    expect(wordsOfItem(item('dues_old', { days: null }), money).id).toBe('attention-dues-old');
    // A kind with no aged message keeps its one message whatever the days.
    expect(wordsOfItem(item('drafts', { days: 3 }), money).id).toBe('attention-drafts');
  });
});

describe('where a line leads', () => {
  it('each kind opens the screen of PRD FR-7.44', () => {
    expect(destinationOfKind('open_days')).toEqual({ kind: 'day' });
    expect(destinationOfKind('stock_waiting')).toEqual({ kind: 'stock' });
    expect(destinationOfKind('dues_old')).toEqual({ kind: 'report', id: 'R20' });
    expect(destinationOfKind('payables_old')).toEqual({ kind: 'report', id: 'R24' });
    expect(destinationOfKind('loans_overdue')).toEqual({ kind: 'report', id: 'R32' });
    expect(destinationOfKind('backup_old')).toEqual({ kind: 'backup' });
    expect(destinationOfKind('names_missing')).toEqual({ kind: 'index' });
    expect(destinationOfKind('takes_overdue')).toEqual({ kind: 'personal-drawings' });
  });

  it('the most urgent record opens itself: a ledger, a one-time person, a statement, a بکری', () => {
    expect(
      destinationOfFirst({
        target: 'account',
        id: 19,
        person_key: null,
        name_ur: 'کریم',
        name_en: 'Karim',
      }),
    ).toEqual({ kind: 'report', id: 'R13', party: 19 });
    const person = destinationOfFirst({
      target: 'account',
      id: 40,
      person_key: 'shahid|0300',
      name_ur: 'شاہد',
      name_en: 'Shahid',
    });
    expect(person).toMatchObject({
      kind: 'report',
      id: 'R13',
      party: 40,
      person: { accountId: 40, personKey: 'shahid|0300', name_en: 'Shahid' },
    });
    expect(
      destinationOfFirst({
        target: 'partner',
        id: 2,
        person_key: null,
        name_ur: null,
        name_en: null,
      }),
    ).toEqual({ kind: 'report', id: 'R31', partner: 2 });
    expect(
      destinationOfFirst({
        target: 'bikri',
        id: 7,
        person_key: null,
        name_ur: null,
        name_en: null,
      }),
    ).toEqual({ kind: 'bikri-book', id: 7 });
  });
});
