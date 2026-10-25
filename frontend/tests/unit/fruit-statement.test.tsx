// The fruit lines of a partner's statement (plan 17d, PRD FR-17.10, E2E-42), in the Urdu and the
// English app: what a partner took home ("2 پیٹی مالٹا @ 1,400 = 2,800") and his equal part of the
// fruit served to the shop's guests (C27). The shape is the backend's own statement line (heading
// `fruit`, detail `fruit_taken`, the item as subject, the packing, and `qty`, `rate_paisa`,
// `value_paisa` and `for` in the values); the amount and the running total are the backend's. The
// words come from Fluent; nothing is added up on the screen (CLAUDE.md rule 1).

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { StatementLine } from '../../src/bindings/StatementLine';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES } from '../../src/i18n';
import { gridOf, textOf } from '../helpers/bikri-screen';
import { named } from '../helpers/cashbook-screen';
import { leaveAppWindow, renderApp } from '../helpers/app';
import * as real from '../helpers/gardens-real';
import { paisaForTest } from '../helpers/paisa';
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

const ORANGE = { id: 91, code: '', name_ur: 'مالٹا', name_en: 'Orange' };
const BOX = { id: 71, code: '', name_ur: 'پیٹی', name_en: 'Box' };

/** A fruit line the way the backend sends it: the amount is the partner's part, negative. */
function fruitLine(
  over: Pick<StatementLine, 'amount_paisa' | 'running_paisa' | 'values'>,
): StatementLine {
  return {
    entry_date: '2026-10-19',
    heading: 'fruit',
    detail: 'fruit_taken',
    entry_id: 61,
    number: 1,
    secondary_number: null,
    subject: ORANGE,
    packing: BOX,
    reason: '',
    ...over,
  };
}

describe.each(LANGUAGES)('the fruit lines of a partner statement (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
    const statement = real.PARTNERS_STATEMENT_GARDEN_ALIF;
    vi.mocked(partnersApi.partnersStatement).mockResolvedValue({
      ...statement,
      lines: [
        fruitLine({
          amount_paisa: paisaForTest(-280_000),
          running_paisa: paisaForTest(-280_000),
          values: { qty: 2, rate_paisa: 140_000, value_paisa: 280_000, for: 'home' },
        }),
        fruitLine({
          amount_paisa: paisaForTest(-100_000),
          running_paisa: paisaForTest(-380_000),
          values: { qty: 3, rate_paisa: 100_000, value_paisa: 300_000, for: 'guests' },
        }),
      ],
      total_count: 2,
    });
  });

  const grid = () => gridOf(language, 'partners-statement-label');

  async function open() {
    await renderApp('/partners/statement?partner=1');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-statement-title')),
    });
    await waitFor(() => {
      expect(grid().querySelectorAll('.ag-row[row-index]').length).toBe(2);
    });
  }

  it('the heading is پھل and the amount and the running total are the backend’s', async () => {
    await open();
    expect(textOf(grid(), 0, 'heading')).toBe(
      message(language, 'partners-statement-heading-fruit'),
    );
    expect([0, 1].map((row) => textOf(grid(), row, 'amount'))).toEqual(['-2,800', '-1,000']);
    expect([0, 1].map((row) => textOf(grid(), row, 'running'))).toEqual(['-2,800', '-3,800']);
  });

  it('fruit taken home reads: the item, the quantity, the packing, the rate and the value', async () => {
    await open();
    expect(textOf(grid(), 0, 'detail')).toBe(
      message(language, 'partners-statement-detail-fruit-taken', {
        for: 'home',
        subject: language === 'ur' ? 'مالٹا' : 'Orange',
        qty: '2',
        packing: language === 'ur' ? 'پیٹی' : 'Box',
        rate: '1,400',
        value: '2,800',
      }),
    );
  });

  it('fruit for the guests says his equal part and shows the whole value', async () => {
    await open();
    expect(textOf(grid(), 1, 'detail')).toBe(
      message(language, 'partners-statement-detail-fruit-taken', {
        for: 'guests',
        subject: language === 'ur' ? 'مالٹا' : 'Orange',
        qty: '3',
        packing: language === 'ur' ? 'پیٹی' : 'Box',
        rate: '1,000',
        value: '3,000',
      }),
    );
  });
});
