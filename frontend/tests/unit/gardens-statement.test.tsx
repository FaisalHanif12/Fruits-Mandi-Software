// The garden lines of a partner's statement (plan 17c, PRD FR-17.10, FR-17.12, E2E-40), in the Urdu and the
// English app, with the REAL lines of the backend for partner الف (tests/fixtures/
// partners_statement_garden_alif.json), shown in the days that open in place on the month statement: each
// part he paid from the shop's cash, his income share of the sale, and the closing, with the amount and the
// running کل رقم as the backend sent them (−18,000, −12,000, −3,000, +3,200, +39,840, ending at 10,040). The
// words of each line come from Fluent; nothing is added up on the screen (CLAUDE.md rule 1).

import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import { LANGUAGES } from '../../src/i18n';
import { named } from '../helpers/cashbook-screen';
import { DAY, leaveAppWindow, renderApp } from '../helpers/app';
import * as real from '../helpers/gardens-real';
import { fakePartnerMonth, openDays, type CardLine } from '../helpers/partner-month';
import { preparePartnerMoney } from '../helpers/partner-money';
import { message } from '../helpers/render';

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

const DAYS = ['2026-10-10', '2026-10-11', '2026-10-12', '2026-10-13'] as const;

describe.each(LANGUAGES)('the garden lines of a partner statement (%s)', (language) => {
  const statement = real.PARTNERS_STATEMENT_GARDEN_ALIF;
  const garden = '0080';

  beforeEach(() => {
    preparePartnerMoney(language);
    vi.mocked(appApi.dayStatus).mockResolvedValue({ ...DAY, today: '2026-10-31' });
    fakePartnerMonth(statement.partner, statement.lines, '2026-10-31');
  });

  /** The lines of the five movements, in the order of the statement: two parts on 11-10-26. */
  async function open(): Promise<CardLine[]> {
    await renderApp('/partners/statement?partner=1&month=2026-10');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-statement-title')),
    });
    const days = await openDays(userEvent.setup(), DAYS);
    return DAYS.flatMap((date) => days.get(date) ?? []);
  }

  it('shows each part paid from the shop, the income share of the sale and the closing, with the amounts of the backend', async () => {
    const lines = await open();
    expect(lines.map((line) => line.amount)).toEqual([
      '-18,000',
      '-12,000',
      '-3,000',
      '3,200',
      '39,840',
    ]);
  });

  it('the month shows the running کل رقم of each day: −18,000, −33,000, −29,800, then 10,040', async () => {
    await open();
    const running = DAYS.map((date) => {
      const row = screen.getByRole('row', {
        name: (name) => name.includes(date.split('-').reverse().join('-')),
      });
      const cells = row.querySelectorAll(':scope > div > [role="gridcell"]');
      return (cells[cells.length - 1]?.textContent ?? '').replace(/[⁨⁩]/g, '');
    });
    expect(running).toEqual(['-18,000', '-33,000', '-29,800', '10,040']);
  });

  it('a part paid from the shop reads: the garden, why, from the shop', async () => {
    const lines = await open();
    for (const [index, reason] of [
      [0, 'first installment'],
      [1, 'second installment'],
      [2, 'labour'],
    ] as const) {
      expect(lines[index]?.detail).toBe(
        message(language, 'partners-statement-detail-garden-paid', { garden, reason }),
      );
      expect(lines[index]?.heading).toBe(message(language, 'partners-statement-heading-garden'));
    }
  });

  it('the closing reads: cost, sales, result, his percent, his part and his money back', async () => {
    const lines = await open();
    expect(lines[4]?.detail).toBe(
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
  });

  it('a closing that posted nothing for him is a note: no amount, the running total stays, the words are the same', async () => {
    // The backend sends such a closing with amount null and the same values (his part and his money
    // back are 0): the running total is the one before it.
    const closing = statement.lines[4];
    const before = statement.lines[3];
    if (closing === undefined || before === undefined) throw new Error('no lines');
    fakePartnerMonth(
      statement.partner,
      [
        ...statement.lines.slice(0, 4),
        {
          ...closing,
          amount_paisa: null,
          running_paisa: before.running_paisa,
          values: { ...closing.values, part_paisa: 0, back_paisa: 0 },
        },
      ],
      '2026-10-31',
    );
    const lines = await open();
    expect(lines[4]?.amount).toBe('');
    expect(lines[4]?.detail).toBe(
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
    const lines = await open();
    expect(lines[3]?.detail).toBe(
      message(language, 'partners-statement-detail-income-share', { number: 1 }),
    );
  });
});
