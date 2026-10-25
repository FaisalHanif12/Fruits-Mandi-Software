// The fruit lines of a partner's statement (plan 17d, PRD FR-17.10, E2E-42), in the Urdu and the
// English app, with the REAL answer of the backend for partner الف (tests/fixtures/
// partners_statement_fruit_alif.json): his income share, what he took home ("2 Box Banana @ 580 =
// 1,160") and his equal part of the fruit served to the shop's guests ("3 Box Apple @ 1,000 =
// 3,000", his part 1,000). The amount and the running total are the backend's (+1,424, then −1,160
// and −1,000, ending at −736); the words come from Fluent; nothing is added up on the screen
// (CLAUDE.md rule 1).

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES } from '../../src/i18n';
import { gridOf, textOf } from '../helpers/bikri-screen';
import { named } from '../helpers/cashbook-screen';
import { leaveAppWindow, renderApp } from '../helpers/app';
import * as real from '../helpers/fruit-real';
import { preparePartnerMoney } from '../helpers/partner-money';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
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

describe.each(LANGUAGES)('the fruit lines of a partner statement (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
    vi.mocked(partnersApi.partnersStatement).mockResolvedValue(real.PARTNERS_STATEMENT_FRUIT_ALIF);
  });

  const grid = () => gridOf(language, 'partners-statement-label');

  async function open() {
    await renderApp('/partners/statement?partner=1');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-statement-title')),
    });
    await waitFor(() => {
      expect(grid().querySelectorAll('.ag-row[row-index]').length).toBe(3);
    });
  }

  it('the heading is پھل and the amount and the running total are the backend’s', async () => {
    await open();
    expect(textOf(grid(), 1, 'heading')).toBe(
      message(language, 'partners-statement-heading-fruit'),
    );
    expect([0, 1, 2].map((row) => textOf(grid(), row, 'amount'))).toEqual([
      '1,424',
      '-1,160',
      '-1,000',
    ]);
    expect([0, 1, 2].map((row) => textOf(grid(), row, 'running'))).toEqual([
      '1,424',
      '264',
      '-736',
    ]);
  });

  it('fruit taken home reads: the item, the quantity, the packing, the rate and the value', async () => {
    await open();
    expect(textOf(grid(), 1, 'detail')).toBe(
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
    const statement = real.PARTNERS_STATEMENT_FRUIT_ALIF;
    const fruit = statement.lines[1];
    if (fruit === undefined) throw new Error('no fruit line');
    vi.mocked(partnersApi.partnersStatement).mockResolvedValue({
      ...statement,
      lines: [
        statement.lines[0] ?? fruit,
        { ...fruit, subject: null, packing: null, values: {} },
        statement.lines[2] ?? fruit,
      ],
    });
    await open();
    const words = textOf(grid(), 1, 'detail');
    expect(words).toBe(
      message(language, 'partners-statement-detail-fruit-earlier', {
        fruit: message(language, 'fruit-number', { number: 1 }),
      }),
    );
    expect(words).not.toMatch(/[{}$]/);
    // The amount of the line is still the backend's.
    expect(textOf(grid(), 1, 'amount')).toBe('-1,160');
  });

  it('fruit for the guests says his equal part and shows the whole value', async () => {
    await open();
    expect(textOf(grid(), 2, 'detail')).toBe(
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
