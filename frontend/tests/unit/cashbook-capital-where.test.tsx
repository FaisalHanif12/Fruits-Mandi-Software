// Plan 78 E, PRD FR-16.14 (Umar, 17-11-2026): the روکڑ page shows where the cash rows of the start of the book are
// (opening cash, a partner's opening capital, capital put in and taken back, each with the partner's name) and a
// card, "where the capital is", behind the partner code. The card is shut at first and reads nothing until it is
// opened; it has two tables, where the partners' money is and whose it is, each with the backend's total (the two
// are equal), and it asks for the code in its own place while the section is locked. Both languages, against a fake
// backend. The numbers are those of the worked day after Events 1 to 5 (PRD FR-16.14): 58,484 + 21,564 − 10,160 −
// 4,400 = 65,488 = −1,004 + 746 + 746 + 65,000. The screen adds nothing up.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { CapitalWhere } from '../../src/bindings/CapitalWhere';
import type { Cashbook } from '../../src/bindings/Cashbook';
import type { CashbookRow } from '../../src/bindings/CashbookRow';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { prepareCashbook } from '../helpers/cashbook-screen';
import { fail, leaveAppWindow, renderApp } from '../helpers/app';
import { paisaForTest } from '../helpers/paisa';
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

const rupees = (value: number) => paisaForTest(value * 100);

const PARTNER_A = { id: 1, code: '', name_ur: 'الف', name_en: 'Partner A' };
const PARTNER_B = { id: 2, code: '', name_ur: 'ب', name_en: 'Partner B' };
const PARTNER_C = { id: 3, code: '', name_ur: 'ج', name_en: 'Partner C' };

/** The card after Events 1 to 5 (PRD FR-16.14): both sides are 65,488. */
const CAPITAL: CapitalWhere = {
  where_lines: [
    { kind: 'cash', amount_paisa: rupees(58_484) },
    { kind: 'buyers_owe', amount_paisa: rupees(21_564) },
    { kind: 'sellers_payable', amount_paisa: rupees(-10_160) },
    { kind: 'owed_for_charges', amount_paisa: rupees(-4_400) },
  ],
  where_total_paisa: rupees(65_488),
  whose_lines: [
    { kind: 'partner_total', partner: PARTNER_A, amount_paisa: rupees(-1_004) },
    { kind: 'partner_total', partner: PARTNER_B, amount_paisa: rupees(746) },
    { kind: 'partner_total', partner: PARTNER_C, amount_paisa: rupees(746) },
    { kind: 'opening_capital', partner: null, amount_paisa: rupees(65_000) },
  ],
  whose_total_paisa: rupees(65_488),
};

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('where the capital is, on the روکڑ page (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const nameOf = (partner: { name_ur: string; name_en: string }) =>
    language === 'ur' ? partner.name_ur : partner.name_en;

  beforeEach(() => {
    prepareCashbook(language, [], { cash: 58_484, cashRows: [] });
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(CAPITAL);
  });

  async function open() {
    const app = await renderApp('/rokar');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-title') });
    return { ...app, user: userEvent.setup() };
  }

  const foldButton = () =>
    screen.getByRole('button', {
      name: (name) => visible(name).includes(m('cashbook-capital-title')),
    });

  it('R1: the card is shut at first, and nothing is read until it is opened', async () => {
    const { user } = await open();
    expect(foldButton().getAttribute('aria-expanded')).toBe('false');
    expect(cashbookApi.cashbookCapitalWhere).not.toHaveBeenCalled();
    await user.click(foldButton());
    await waitFor(() => {
      expect(cashbookApi.cashbookCapitalWhere).toHaveBeenCalledTimes(1);
    });
    expect(foldButton().getAttribute('aria-expanded')).toBe('true');
  });

  it('R2: it shows where the money is and whose it is, each line and each total as the backend sent them', async () => {
    const { user } = await open();
    await user.click(foldButton());
    const where = await screen.findByRole('table', { name: m('cashbook-capital-where-title') });
    const whose = await screen.findByRole('table', { name: m('cashbook-capital-whose-title') });
    const whereText = visible(where.textContent);
    expect(whereText).toContain(m('cashbook-capital-where-cash'));
    for (const figure of ['58,484', '21,564', '10,160', '4,400', '65,488']) {
      expect(whereText).toContain(figure);
    }
    const whoseText = visible(whose.textContent);
    expect(whoseText).toContain(
      m('cashbook-capital-whose-partner-total', { partner: nameOf(PARTNER_A) }),
    );
    expect(whoseText).toContain(m('cashbook-capital-whose-opening-capital'));
    for (const figure of ['1,004', '746', '65,000', '65,488']) expect(whoseText).toContain(figure);
    // The two totals are in the last row of each table, equal: the screen only shows them.
    const lastRow = (table: HTMLElement) => {
      const rows = within(table).getAllByRole('row');
      return visible(rows[rows.length - 1]?.textContent);
    };
    expect(lastRow(where)).toContain('65,488');
    expect(lastRow(whose)).toContain('65,488');
  });

  it('R3: a partner section that is locked asks for the code in the place of the figures', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockRejectedValue(
      fail({ code: 'ERR_PARTNER_LOCKED', field: null, values: {} }),
    );
    const { user } = await open();
    await user.click(foldButton());
    expect(
      await screen.findByText((text) => visible(text) === m('partners-code-box-text')),
    ).toBeInTheDocument();
    // The cash rows of the page still work: nothing but the card is closed.
    expect(screen.queryByRole('table', { name: m('cashbook-capital-where-title') })).toBeNull();
    expect(
      screen.getByRole('heading', { level: 1, name: m('cashbook-title') }),
    ).toBeInTheDocument();
  });

  it('R4: another failure says so in words and can be tried again', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockRejectedValueOnce(
      fail({ code: 'ERR_INTERNAL', field: null, values: {} }),
    );
    const { user } = await open();
    await user.click(foldButton());
    expect(
      await screen.findByText((text) => visible(text) === m('cashbook-capital-failed')),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: m('common-retry') }));
    await screen.findByRole('table', { name: m('cashbook-capital-where-title') });
    expect(cashbookApi.cashbookCapitalWhere).toHaveBeenCalledTimes(2);
  });

  it('R5b: the four main lines always show, also at 0; any other line shows only when it is not 0', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue({
      ...CAPITAL,
      where_lines: [
        { kind: 'cash', amount_paisa: rupees(58_484) },
        { kind: 'buyers_owe', amount_paisa: rupees(21_564) },
        { kind: 'seller_advances', amount_paisa: rupees(0) },
        { kind: 'sellers_owe_beyond', amount_paisa: rupees(0) },
        { kind: 'sellers_payable', amount_paisa: rupees(-10_160) },
        { kind: 'money_lent', amount_paisa: rupees(0) },
        { kind: 'separate_khatas', amount_paisa: rupees(2_000) },
      ],
    });
    const { user } = await open();
    await user.click(foldButton());
    const where = await screen.findByRole('table', { name: m('cashbook-capital-where-title') });
    const text = visible(where.textContent);
    expect(text).toContain(m('cashbook-capital-where-seller-advances'));
    expect(text).toContain(m('cashbook-capital-where-separate-khatas'));
    expect(text).not.toContain(m('cashbook-capital-where-money-lent'));
    expect(text).not.toContain(m('cashbook-capital-where-sellers-owe-beyond'));
  });

  it('R5: a line the backend does not send is not shown, and a line that is only for a bug or a gap says so in words', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue({
      ...CAPITAL,
      whose_lines: [
        ...CAPITAL.whose_lines,
        { kind: 'unrecorded_where', partner: null, amount_paisa: rupees(5_000) },
      ],
    });
    const { user } = await open();
    await user.click(foldButton());
    const whose = await screen.findByRole('table', { name: m('cashbook-capital-whose-title') });
    const text = visible(whose.textContent);
    expect(text).toContain(m('cashbook-capital-whose-unrecorded-where'));
    // No advance line: the backend sent none, so none is drawn.
    expect(text).not.toContain(m('cashbook-capital-whose-advances-open'));
  });

  it('R6: a language switch keeps the card open and writes the partners in the other language', async () => {
    const { user } = await open();
    await user.click(foldButton());
    await screen.findByRole('table', { name: m('cashbook-capital-whose-title') });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const whose = await screen.findByRole('table', {
      name: message(other, 'cashbook-capital-whose-title'),
    });
    expect(visible(whose.textContent)).toContain(
      message(other, 'cashbook-capital-whose-partner-total', {
        partner: other === 'ur' ? PARTNER_A.name_ur : PARTNER_A.name_en,
      }),
    );
    // Still one read: the switch redraws, it does not ask again.
    expect(cashbookApi.cashbookCapitalWhere).toHaveBeenCalledTimes(1);
  });
});

/** A cash row that opens the book or moves a partner's capital (plan 78 E). */
function row(
  id: number,
  date: string,
  kind: CashbookRow['kind'],
  partner: CashbookRow['partner'],
  inRupees: number,
  running: number,
): CashbookRow {
  return {
    entry_id: id,
    entry_date: date,
    posting_type:
      kind === 'capital_out' ? 'capital_out' : kind === 'capital_in' ? 'capital_in' : 'opening',
    number: null,
    account: null,
    kind,
    partner,
    narration: '',
    in_paisa: rupees(inRupees),
    out_paisa: rupees(kind === 'capital_out' ? inRupees : 0),
    running_paisa: rupees(running),
    source: null,
  };
}

describe.each(LANGUAGES)('the rows that start the book, on the روکڑ page (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const nameOf = (partner: { name_ur: string; name_en: string }) =>
    language === 'ur' ? partner.name_ur : partner.name_en;

  it('R7: opening cash, a partner’s opening capital, capital in and capital out say what they are, with his name', async () => {
    prepareCashbook(language, [], { cash: 108_484, cashRows: [] });
    const rows: CashbookRow[] = [
      row(1, '2026-10-10', 'opening_cash', null, 65_000, 65_000),
      row(2, '2026-10-10', 'opening_capital', PARTNER_A, 50_000, 115_000),
      row(3, '2026-10-10', 'capital_in', PARTNER_B, 50_000, 165_000),
      row(4, '2026-10-10', 'capital_out', PARTNER_C, 6_516, 158_484),
    ];
    const page: Cashbook = {
      opening_paisa: rupees(0),
      closing_paisa: rupees(158_484),
      first_entry_date: '2026-10-10',
      rows,
      total_count: rows.length,
      totals: { in_paisa: rupees(165_000), out_paisa: rupees(6_516) },
      months: [],
    };
    vi.mocked(cashbookApi.cashbookGet).mockResolvedValue(page);
    await renderApp('/rokar');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-title') });
    const list = await screen.findByRole('group', { name: m('cashbook-list-label') });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain(m('cashbook-row-opening-cash'));
    });
    const text = visible(list.textContent);
    expect(text).toContain(m('cashbook-row-opening-capital', { partner: nameOf(PARTNER_A) }));
    expect(text).toContain(m('cashbook-row-capital-in', { partner: nameOf(PARTNER_B) }));
    expect(text).toContain(m('cashbook-row-capital-out', { partner: nameOf(PARTNER_C) }));
  });
});
