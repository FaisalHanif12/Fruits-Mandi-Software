// The garden lines of a partner's statement (plan 17c, PRD FR-17.10, E2E-40), in the Urdu and the
// English app, with the REAL answer of the backend for partner الف (tests/fixtures/
// partners_statement_garden_alif.json): each part he paid from the shop's cash, his income share of
// the sale, and the closing, with the amount and the running کل رقم as the backend sent them
// (−18,000, −12,000, −3,000, +3,200, +39,840, ending at 10,040). The words of each line come from
// Fluent; nothing is added up on the screen (CLAUDE.md rule 1).

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES } from '../../src/i18n';
import { cellOf, gridOf, textOf } from '../helpers/bikri-screen';
import { named } from '../helpers/cashbook-screen';
import { leaveAppWindow, renderApp } from '../helpers/app';
import * as real from '../helpers/gardens-real';
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

describe.each(LANGUAGES)('the garden lines of a partner statement (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
    vi.mocked(partnersApi.partnersStatement).mockResolvedValue(real.PARTNERS_STATEMENT_GARDEN_ALIF);
  });

  const grid = () => gridOf(language, 'partners-statement-label');
  const garden = message(language, 'gardens-number', { number: 1 });

  async function open() {
    await renderApp('/partners/statement?partner=1');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-statement-title')),
    });
    await waitFor(() => {
      expect(grid().querySelectorAll('.ag-row[row-index]').length).toBe(5);
    });
  }

  it('shows each part paid from the shop, the income share of the sale and the closing, with the running total', async () => {
    await open();
    expect([0, 1, 2, 3, 4].map((row) => textOf(grid(), row, 'amount'))).toEqual([
      '-18,000',
      '-12,000',
      '-3,000',
      '3,200',
      '39,840',
    ]);
    expect([0, 1, 2, 3, 4].map((row) => textOf(grid(), row, 'running'))).toEqual([
      '-18,000',
      '-30,000',
      '-33,000',
      '-29,800',
      '10,040',
    ]);
    expect(textOf(grid(), 0, 'date')).toBe('10-10-2026');
    expect(textOf(grid(), 4, 'date')).toBe('13-10-2026');
  });

  it('a part paid from the shop reads: the garden, why, from the shop', async () => {
    await open();
    for (const [row, reason] of [
      [0, 'first installment'],
      [1, 'second installment'],
      [2, 'labour'],
    ] as const) {
      expect(textOf(grid(), row, 'detail')).toBe(
        message(language, 'partners-statement-detail-garden-paid', { garden, reason }),
      );
      expect(textOf(grid(), row, 'heading')).toBe(
        message(language, 'partners-statement-heading-garden'),
      );
    }
  });

  it('the closing reads: cost, sales, result, his percent, his part and his money back', async () => {
    await open();
    expect(textOf(grid(), 4, 'detail')).toBe(
      message(language, 'partners-statement-detail-garden-closed', {
        garden,
        cost: '55,000',
        sales: '66,400',
        result: '11,400',
        share: '60',
        part: '6,840',
        back: '39,840',
      }),
    );
    expect(cellOf(grid(), 4, 'detail')).not.toBeNull();
  });

  it('a closing that posted nothing for him is a note: no amount, the running total stays, the words are the same', async () => {
    // The backend sends such a closing with amount null and the same values (his part and his money
    // back are 0): the running total is the one before it.
    const statement = real.PARTNERS_STATEMENT_GARDEN_ALIF;
    const closing = statement.lines[4];
    const before = statement.lines[3];
    if (closing === undefined || before === undefined) throw new Error('no lines');
    vi.mocked(partnersApi.partnersStatement).mockResolvedValue({
      ...statement,
      lines: [
        ...statement.lines.slice(0, 4),
        {
          ...closing,
          amount_paisa: null,
          running_paisa: before.running_paisa,
          values: { ...closing.values, part_paisa: 0, back_paisa: 0 },
        },
      ],
    });
    await open();
    expect(textOf(grid(), 4, 'amount')).toBe('');
    expect(textOf(grid(), 4, 'running')).toBe(textOf(grid(), 3, 'running'));
    expect(textOf(grid(), 4, 'detail')).toBe(
      message(language, 'partners-statement-detail-garden-closed', {
        garden,
        cost: '55,000',
        sales: '66,400',
        result: '11,400',
        share: '60',
        part: '0',
        back: '0',
      }),
    );
  });

  it('the line of the sale in the sale book still reads as before', async () => {
    await open();
    expect(textOf(grid(), 3, 'detail')).toBe(
      message(language, 'partners-statement-detail-income-share', { number: 1 }),
    );
  });
});
