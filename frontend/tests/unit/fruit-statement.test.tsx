// The fruit lines of a partner's statement (plan 17d, PRD FR-17.10, FR-17.12, E2E-42), in the Urdu and the
// English app, with the REAL lines of the backend for partner الف (tests/fixtures/
// partners_statement_fruit_alif.json), shown in the day that opens in place on the month statement: his
// income share, what he took home ("2 Box Banana @ 580 = 1,160") and his equal part of the fruit served to
// the shop's guests ("3 Box Apple @ 1,000 = 3,000", his part 1,000). The amounts are the backend's (+1,424,
// then −1,160 and −1,000, ending at −736); the words come from Fluent; nothing is added up on the screen
// (CLAUDE.md rule 1).

import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import { LANGUAGES } from '../../src/i18n';
import { named } from '../helpers/cashbook-screen';
import { DAY, leaveAppWindow, renderApp } from '../helpers/app';
import * as real from '../helpers/fruit-real';
import { fakePartnerMonth, openDays } from '../helpers/partner-month';
import { preparePartnerMoney } from '../helpers/partner-money';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

const DATE = '2026-10-10';

describe.each(LANGUAGES)('the fruit lines of a partner statement (%s)', (language) => {
  const statement = real.PARTNERS_STATEMENT_FRUIT_ALIF;

  beforeEach(() => {
    preparePartnerMoney(language);
    vi.mocked(appApi.dayStatus).mockResolvedValue({ ...DAY, today: '2026-10-31' });
    fakePartnerMonth(statement.partner, statement.lines, '2026-10-31');
  });

  async function open() {
    await renderApp('/partners/statement?partner=1&month=2026-10');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-statement-title')),
    });
    const user = userEvent.setup();
    const days = await openDays(user, [DATE]);
    return days.get(DATE) ?? [];
  }

  it('the month row shows the fruit column and the کل رقم the backend gave: +1,424, −2,160, ending at −736', async () => {
    await open();
    const row = screen.getByRole('row', { name: (name) => name.includes('10-10-2026') });
    const text = visible(row.textContent);
    for (const figure of ['1,424', '-2,160', '-736']) expect(text).toContain(figure);
  });

  it('the group پھل is said once, by the sentence, and the amounts are the backend’s', async () => {
    const lines = await open();
    expect(lines.map((line) => line.amount)).toEqual(['1,424', '-1,160', '-1,000']);
    // "Fruit Taken for home: ..." already says the group, so there is no chip in front of it.
    expect(lines[1]?.heading).toBe('');
    expect(lines[1]?.detail).toContain(
      visible(message(language, 'partners-statement-heading-fruit')),
    );
  });

  it('fruit taken home reads: the item, the quantity, the packing, the rate and the value', async () => {
    const lines = await open();
    expect(lines[1]?.detail).toBe(
      message(language, 'partners-statement-detail-fruit-taken', {
        for: 'home',
        subject: language === 'ur' ? 'کیلا' : 'Banana',
        qty: '2',
        packing: language === 'ur' ? 'پیٹی' : 'Box',
        rate: '580',
        value: '1,160',
      }),
    );
  });

  it('an earlier posting of a changed entry has its number only and reads as that, not as a sentence with holes', async () => {
    const fruit = statement.lines[1];
    if (fruit === undefined) throw new Error('no fruit line');
    fakePartnerMonth(
      statement.partner,
      [
        statement.lines[0] ?? fruit,
        { ...fruit, subject: null, packing: null, values: {} },
        statement.lines[2] ?? fruit,
      ],
      '2026-10-31',
    );
    const lines = await open();
    const words = lines[1]?.detail ?? '';
    expect(words).toBe(
      message(language, 'partners-statement-detail-fruit-earlier', {
        fruit: message(language, 'fruit-number', { number: 1 }),
      }),
    );
    expect(words).not.toMatch(/[{}$]/);
    // The amount of the line is still the backend's.
    expect(lines[1]?.amount).toBe('-1,160');
  });

  it('fruit for the guests says his equal part and shows the whole value', async () => {
    const lines = await open();
    expect(lines[2]?.detail).toBe(
      message(language, 'partners-statement-detail-fruit-taken', {
        for: 'guests',
        subject: language === 'ur' ? 'سیب' : 'Apple',
        qty: '3',
        packing: language === 'ur' ? 'پیٹی' : 'Box',
        rate: '1,000',
        value: '3,000',
      }),
    );
  });
});
