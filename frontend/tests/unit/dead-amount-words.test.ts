// The words of the dead amount (plan 40, PRD FR-10.10, FR-7.23, NFR-URD-29): every message the
// backend sends for R38, R34 and R35 and every message of the screens exists in both languages and says
// something different in each; the Urdu uses the exact words of PRD 9.1; the audit log has words for the
// two new actions; and a party in the dead amount carries the word in a report cell.

import { describe, expect, it } from 'vitest';
import type { ReportParty } from '../../src/bindings/ReportParty';
import { cellText } from '../../src/features/reports/cells';
import { actionWord } from '../../src/features/audit/words';
import { bundles, type Language } from '../../src/i18n';
import { message } from '../helpers/render';
import { ARSHAD_DEAD } from '../helpers/dead';

/** The message IDs the backend sends in R38 (the screen's own `dead-*` ones), R34 and R35 (plan 40). */
const BACKEND_IDS = [
  'dead-report-title',
  'dead-report-help',
  'dead-col-party',
  'dead-col-type',
  'dead-col-reason',
  'dead-col-note',
  'dead-col-date',
  'dead-col-owed-then',
  'dead-col-recovered',
  'dead-col-corrections',
  'dead-col-still-owed',
  'dead-col-last-recovery',
  'dead-col-status',
  'dead-type-buyer',
  'dead-type-seller',
  'dead-type-pukhta',
  'dead-type-money_lent',
  'dead-reason-died',
  'dead-reason-left',
  'dead-reason-other',
  'dead-status-owing',
  'dead-status-recovered',
  'dead-status-undone',
  'dead-status-credit',
  'report-r34-sellers-dues',
  'report-r34-dead-amount',
  'report-r34-total-dues',
  'report-r35-total-with-dead',
  'audit-action-dead-mark',
  'audit-action-dead-undo',
];

describe('every message of the dead amount is in both languages', () => {
  it.each(BACKEND_IDS)('%s has Urdu and English words', (id) => {
    for (const language of ['ur', 'en'] as const) {
      expect(bundles[language].hasMessage(id), `${language} has ${id}`).toBe(true);
      expect(message(language, id).trim()).not.toBe('');
    }
    expect(message('ur', id)).not.toBe(message('en', id));
  });
});

describe('the Urdu says exactly what PRD 9.1 says', () => {
  it('has the fixed tile names', () => {
    expect(message('ur', 'dashboard-tile-dues')).toBe('گاہکوں کے ذمے');
    expect(message('ur', 'dashboard-tile-dues-sellers')).toBe('بیوپاریوں کے ذمے');
    expect(message('ur', 'dashboard-tile-dues-total')).toBe('کل اگراہی');
    expect(message('ur', 'dashboard-tile-dead')).toBe('ڈوبی رقم');
    expect(message('ur', 'dashboard-tile-cash')).toBe('موجود روکڑ');
    expect(message('ur', 'dashboard-tile-payable')).toBe('بیوپاریوں کو ادائیگی باقی');
    expect(message('en', 'dashboard-tile-dues')).toBe("Buyers' dues");
    expect(message('en', 'dashboard-tile-dues-sellers')).toBe("Sellers' dues");
    expect(message('en', 'dashboard-tile-dues-total')).toBe('Total Dues');
    expect(message('en', 'dashboard-tile-dead')).toBe('Dead Amount');
  });

  it('has the words of the buttons and the tag', () => {
    expect(message('ur', 'dead-move-button')).toBe('ڈوبی رقم میں ڈالیں');
    expect(message('ur', 'dead-undo-button')).toBe('دوبارہ فعال کریں');
    expect(message('ur', 'dead-tag')).toBe('ڈوبی');
    expect(message('ur', 'dead-reason-died')).toBe('فوت ہو گئے');
    expect(message('ur', 'dead-reason-left')).toBe('کاروبار چھوڑ دیا');
    expect(message('ur', 'dead-reason-other')).toBe('دیگر');
    expect(message('ur', 'dead-status-recovered')).toBe('پوری وصول ہو گئی');
  });
});

describe('the audit log', () => {
  it.each(['ur', 'en'] as const)('has words for the two actions (%s)', (language: Language) => {
    const text = (id: string) => message(language, id);
    expect(actionWord('dead_mark', text)).toBe(message(language, 'audit-action-dead-mark'));
    expect(actionWord('dead_undo', text)).toBe(message(language, 'audit-action-dead-undo'));
  });
});

describe('a party in the dead amount in a report cell', () => {
  const party: ReportParty = {
    account_id: 22,
    code: '03040002',
    short_code: '2',
    name_ur: 'ارشد ملتانی',
    name_en: 'Arshad Multani',
    tag: null,
    person_key: null,
  };
  const context = (language: Language) => ({
    language,
    text: (id: string) => message(language, id),
    money: () => '',
    signedMoney: () => '',
  });

  it.each(['ur', 'en'] as const)('carries the word after his name (%s)', (language: Language) => {
    const text = cellText(
      { type: 'party', party: { ...party, dead: ARSHAD_DEAD } },
      context(language),
    );
    expect(text).toBe(
      `03040002 ${language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani'} (${message(language, 'dead-tag')})`,
    );
  });

  it('has no word for a party who is not in it', () => {
    const text = cellText({ type: 'party', party }, context('en'));
    expect(text).toBe('03040002 Arshad Multani');
  });
});
