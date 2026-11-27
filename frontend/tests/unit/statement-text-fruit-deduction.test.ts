// The fruit line of a partner's statement with a کل کٹوتی (plan 97 item 1, PRD FR-16.11 v1.59, FR-17.10), in the Urdu and the
// English app. The backend sends `deduction_paisa` on every fruit line (0 when none) and the whole value: "2 @ 1,400 = 2,800،
// کٹوتی 200" and the amount of the line is the net, 2,600. A line with no deduction reads as before.

import { describe, expect, it } from 'vitest';
import type { StatementLine } from '../../src/bindings/StatementLine';
import { detailText, type StatementSay } from '../../src/features/partners/statement-text';
import { LANGUAGES } from '../../src/i18n';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';

function fruitLine(values: StatementLine['values']): StatementLine {
  return {
    entry_date: '2026-10-10',
    heading: 'fruit',
    detail: 'fruit_taken',
    entry_id: 9,
    number: 4,
    venture_code: null,
    secondary_number: null,
    subject: { id: 63, code: '0063', name_ur: 'سیب', name_en: 'Apple' },
    packing: { id: 71, code: '0071', name_ur: 'پیٹی', name_en: 'Box' },
    reason: '',
    amount_paisa: paisaForTest(-260_000),
    running_paisa: paisaForTest(-260_000),
    values,
  };
}

describe.each(LANGUAGES)('the statement words of fruit with a deduction (%s)', (language) => {
  const say: StatementSay = {
    language,
    grouping: 'international',
    text: (id, vars) => message(language, id, vars ?? {}),
    exists: () => true,
  };
  const text = (values: StatementLine['values']) => visible(detailText(fruitLine(values), say));
  const base = { for: 'home', qty: 2, rate_paisa: 140_000, value_paisa: 280_000 };
  const deductionWord = language === 'ur' ? 'کٹوتی' : 'deduction';

  it('says the whole value and the deduction on the line, for home fruit', () => {
    const sentence = text({ ...base, deduction_paisa: 20_000 });
    expect(sentence).toContain('2,800');
    expect(sentence).toContain('200');
    // The sentence of the deduction variant, not the plain one.
    expect(sentence).toContain(deductionWord);
  });

  it('says the deduction for the shop’s guests too', () => {
    const sentence = text({ ...base, for: 'guests', deduction_paisa: 20_000 });
    expect(sentence).toContain('200');
    expect(sentence).toContain(message(language, 'glossary-mehman'));
  });

  it('a line with a deduction of 0 reads as before: no deduction word', () => {
    const plain = text({ ...base, deduction_paisa: 0 });
    const old = text(base);
    expect(plain).toBe(old);
    expect(plain).toContain('2,800');
    expect(plain).not.toContain('200');
    expect(plain).not.toContain(deductionWord);
  });
});
