// The words of the statement lines of a linked khata and of a loan that is linked (plan 76 points 5 and 10, PRD
// FR-16.6, FR-16.13, FR-17.10), in the Urdu and the English app. The backend sends the kind of the line, its
// number, the person or khata and the figures; the sentence is Fluent's. Each sentence names the khata or the
// person, the voucher in its own series (KO-, KI-, L-, LR-), the reason and the figures, and the note of a loan
// that is the shop's has no amount of its own. The same messages are read by the backend for the print.

import { describe, expect, it } from 'vitest';
import type { StatementLine } from '../../src/bindings/StatementLine';
import {
  detailText,
  headingRepeated,
  headingText,
  type StatementSay,
} from '../../src/features/partners/statement-text';
import { LANGUAGES } from '../../src/i18n';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';

function line(patch: Partial<StatementLine>): StatementLine {
  return {
    entry_date: '2026-10-10',
    heading: 'linked',
    detail: 'khata_paid',
    entry_id: 7,
    number: 3,
    venture_code: null,
    secondary_number: null,
    subject: { id: 71, code: '03050001', name_ur: 'خاندانی کھاتہ', name_en: 'Family Khata' },
    packing: null,
    reason: 'family',
    amount_paisa: paisaForTest(-400_000),
    running_paisa: paisaForTest(-400_000),
    values: {},
    ...patch,
  };
}

describe.each(LANGUAGES)('the statement words of linked money (%s)', (language) => {
  const say: StatementSay = {
    language,
    grouping: 'international',
    text: (id, vars) => message(language, id, vars ?? {}),
    exists: () => true,
  };
  const family = language === 'ur' ? 'خاندانی کھاتہ' : 'Family Khata';
  const text = (patch: Partial<StatementLine>) => visible(detailText(line(patch), say));
  /** A voucher number in its own series: KO-3 in English, the Urdu letter of the series in Urdu. */
  const series = (id: string) => visible(message(language, id, { number: 3 }));

  it('the group has its own heading', () => {
    expect(headingText(line({}), say)).toBe(message(language, 'partners-statement-heading-linked'));
  });

  it('the group chip is left out when the sentence already says it, and kept when it adds something', () => {
    const repeated = (patch: Partial<StatementLine>) =>
      headingRepeated(headingText(line(patch), say), detailText(line(patch), say));
    // "Sale 1: ..." under the group Sale, "Shop Expense E-1: ..." under Expense, "Fruit Taken for home: ..." under Fruit Taken.
    expect(repeated({ heading: 'bikri', detail: 'income_share', values: {} })).toBe(true);
    expect(repeated({ heading: 'expense', detail: 'expense_share', values: {} })).toBe(true);
    expect(
      repeated({
        heading: 'fruit',
        detail: 'fruit_taken',
        values: { for: 'home', qty: 2, rate_paisa: 140_000, value_paisa: 280_000 },
      }),
    ).toBe(true);
    // A khata or a loan line under "Separate Khata and Money Lent" says the khata and the voucher, not the group.
    expect(repeated({ detail: 'khata_paid', values: { kind: 'khata_out' } })).toBe(false);
    expect(repeated({ detail: 'lent_given', values: { kind: 'money_lent' } })).toBe(false);
    // A garden known by its number: the sentence does not say the word Garden, so the chip stays.
    expect(repeated({ heading: 'garden', detail: 'garden_paid', values: {} })).toBe(false);
  });

  it('a heading with no words is never repeated', () => {
    expect(headingRepeated('', 'anything')).toBe(false);
  });

  it('money paid out of a linked khata names the khata, its voucher KO-3 and the reason', () => {
    const sentence = text({ detail: 'khata_paid', values: { kind: 'khata_out' } });
    expect(sentence).toContain(family);
    expect(sentence).toContain(series('vouchers-number-khata-out'));
    expect(sentence).toContain('family');
  });

  it('money received for it uses the KI- series', () => {
    const sentence = text({ detail: 'khata_received', values: { kind: 'khata_in' } });
    expect(sentence).toContain(family);
    expect(sentence).toContain(series('vouchers-number-khata-in'));
  });

  it('money lent to a person and its return use the L- and LR- series', () => {
    const person = { id: 91, code: '01030091', name_ur: 'نواز', name_en: 'Nawaz' };
    const given = text({
      detail: 'lent_given',
      subject: person,
      reason: 'next season',
      values: { kind: 'money_lent' },
    });
    expect(given).toContain(language === 'ur' ? 'نواز' : 'Nawaz');
    expect(given).toContain(series('vouchers-number-money-lent'));
    expect(given).toContain('next season');
    const returned = text({
      detail: 'lent_returned',
      subject: person,
      values: { kind: 'money_lent_return' },
    });
    expect(returned).toContain(series('vouchers-number-money-lent-return'));
  });

  it("the other entries of a linked khata name it and the entry's number", () => {
    const sentence = text({ detail: 'linked_other', values: { kind: 'other' } });
    expect(sentence).toContain(family);
    expect(sentence).toContain('3');
  });

  it("a loan that is the shop's is a note with the amount and the partner's part, and no amount of its own", () => {
    const sentence = text({
      detail: 'lent_share_note',
      amount_paisa: null,
      subject: { id: 91, code: '01030091', name_ur: 'نواز', name_en: 'Nawaz' },
      values: { kind: 'given', amount_paisa: 500_000, part_paisa: 200_000 },
    });
    // 5,000 at 40 / 30 / 30: his part is 2,000.
    expect(sentence).toContain('5,000');
    expect(sentence).toContain('2,000');
    expect(sentence).toContain(series('vouchers-number-money-lent'));
    // Returned reads the other way.
    const back = text({
      detail: 'lent_share_note',
      amount_paisa: null,
      subject: { id: 91, code: '01030091', name_ur: 'نواز', name_en: 'Nawaz' },
      values: { kind: 'returned', amount_paisa: 200_000, part_paisa: 80_000 },
    });
    expect(back).toContain('2,000');
    expect(back).toContain('800');
    expect(back).not.toBe(sentence);
  });
});
