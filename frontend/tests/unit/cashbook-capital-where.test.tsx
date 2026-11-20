// Plan 87, PRD FR-16.14 (Umar, 20-11-2026): the capital card of the روکڑ page reads like a statement. A headline with
// the partners' total capital and ✓ when the two sides are equal; where the money is in two blocks (to receive and in
// hand, to pay) with their subtotals and the net, and no minus figure and no "minus" word anywhere; whose it is, with
// each partner's share and a bar; an amber warning for the opening difference with the accounts that make it and the
// button that puts the partners' opening capital into روکڑ with a reason and the Owner's sign; every line opens its
// page; a day to read it as of; print and PDF; and the partner code. The card is shut at first and reads nothing
// until it is opened. Both languages, against a fake backend. The screen adds nothing up: each figure is read from
// what the backend sent (tests/helpers/capital-card.ts). The rows that start the book (plan 78 E) are at the end.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Cashbook } from '../../src/bindings/Cashbook';
import type { CashbookRow } from '../../src/bindings/CashbookRow';
import * as cashbookApi from '../../src/features/cashbook/api';
import type { CapitalWhere } from '../../src/features/cashbook/capital-types';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { prepareCashbook, named } from '../helpers/cashbook-screen';
import {
  EMPTY_CARD,
  LABOUR,
  OPENING_CAPITAL_ACCOUNT,
  PARTNER_A,
  PARTNER_B,
  PARTNER_C,
  UMAR_AFTER,
  UMAR_BEFORE,
  WORKED_DAY,
  line,
} from '../helpers/capital-card';
import { fail, leaveAppWindow, renderApp } from '../helpers/app';
import { paisaForTest } from '../helpers/paisa';
import { partnerMonthPrintData } from '../helpers/partner-month-print';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const rupees = (value: number) => paisaForTest(value * 100);

/** A minus sign before a figure: the card never shows one (the side of an amount is its block). */
/** A text matcher that ignores Fluent's isolation marks around the terms of a message. */
const said = (text: string) => (content: string) => visible(content) === text;

const MINUS = /[−\u2212]|(^|[\s(])-\d/;

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
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(WORKED_DAY);
  });

  async function open(card: CapitalWhere = WORKED_DAY) {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(card);
    const app = await renderApp('/rokar');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-title') });
    const user = userEvent.setup();
    await user.click(foldButton());
    await screen.findByRole('heading', { level: 3, name: named(m('cashbook-capital-headline')) });
    return { ...app, user };
  }

  const foldButton = () =>
    screen.getByRole('button', {
      name: (name) => visible(name).includes(m('cashbook-capital-title')),
    });
  const table = (titleId: string) => screen.getByRole('table', { name: named(m(titleId)) });
  /** The accounts of the warning are folded so the card stays short: open them. */
  const showParts = async (user: ReturnType<typeof userEvent.setup>) => {
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).includes(m('cashbook-capital-warning-parts')),
      }),
    );
  };
  const cardText = () => visible(foldButton().closest('section')?.textContent);

  it('R1: the card is shut at first, and nothing is read until it is opened, as of today', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(WORKED_DAY);
    const { user } = await renderOpen();
    expect(foldButton().getAttribute('aria-expanded')).toBe('false');
    expect(cashbookApi.cashbookCapitalWhere).not.toHaveBeenCalled();
    await user.click(foldButton());
    await waitFor(() => {
      expect(cashbookApi.cashbookCapitalWhere).toHaveBeenCalledTimes(1);
    });
    expect(cashbookApi.cashbookCapitalWhere).toHaveBeenLastCalledWith({});
    expect(foldButton().getAttribute('aria-expanded')).toBe('true');
  });

  async function renderOpen() {
    const app = await renderApp('/rokar');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-title') });
    return { ...app, user: userEvent.setup() };
  }

  it('R2: the headline names the partners’ total capital with ✓ when both sides are equal', async () => {
    await open();
    const headline = screen.getByRole('heading', {
      level: 3,
      name: named(m('cashbook-capital-headline')),
    });
    const box = headline.closest('div')?.parentElement;
    expect(visible(box?.textContent)).toContain('65,488');
    expect(visible(box?.textContent)).toContain(m('cashbook-capital-equal'));
    expect(screen.queryByText(said(m('cashbook-capital-unequal')))).toBeNull();
  });

  it('R2b: a card whose sides differ says so in words instead of ✓', async () => {
    await open({ ...WORKED_DAY, sides_equal: false });
    expect(screen.getByText(said(m('cashbook-capital-unequal')))).toBeInTheDocument();
    expect(screen.queryByText(said(m('cashbook-capital-equal')))).toBeNull();
  });

  it('R2c: with an opening difference standing apart the headline claims nothing: no ✓ and no "differ" chip, the warning tells the story', async () => {
    await open(UMAR_BEFORE);
    expect(screen.queryByText(said(m('cashbook-capital-equal')))).toBeNull();
    expect(screen.queryByText(said(m('cashbook-capital-unequal')))).toBeNull();
    expect(cardText()).toContain(m('cashbook-capital-warning-place'));
  });

  it('R3: two blocks with their lines, the backend’s subtotals and the net, each amount as sent', async () => {
    await open();
    const receive = visible(table('cashbook-capital-receive-title').textContent);
    expect(receive).toContain(m('cashbook-capital-where-cash'));
    for (const figure of ['58,484', '21,564', '80,048']) expect(receive).toContain(figure);
    // What is owed for charges is one line for each account, named by the account.
    const owe = visible(table('cashbook-capital-owe-title').textContent);
    expect(owe).toContain(m('cashbook-capital-where-sellers-payable'));
    expect(owe).toContain(nameOf(LABOUR));
    for (const figure of ['10,160', '300', '4,000', '100', '14,560']) expect(owe).toContain(figure);
    // The subtotal is the last row of each block; the net closes them.
    const lastRow = (box: HTMLElement) => {
      const rows = within(box).getAllByRole('row');
      return visible(rows[rows.length - 1]?.textContent);
    };
    expect(lastRow(table('cashbook-capital-receive-title'))).toContain('80,048');
    expect(lastRow(table('cashbook-capital-owe-title'))).toContain('14,560');
    expect(cardText()).toContain(m('cashbook-capital-net-label'));
  });

  it('R4: no minus figure and no word for minus anywhere on the card, also for a partner below zero: his amount, the words "owes the firm" and no sign', async () => {
    await open();
    expect(cardText()).not.toMatch(MINUS);
    expect(cardText()).not.toContain(language === 'ur' ? 'منفی' : 'Minus');
    // Partner A took more than he has: his total is below zero, written without a sign, with the words.
    const whose = visible(table('cashbook-capital-whose-title').textContent);
    expect(whose).toContain(
      m('cashbook-capital-whose-partner-total', { partner: nameOf(PARTNER_A) }),
    );
    expect(whose).toContain('1,004');
    expect(whose).toContain(m('cashbook-capital-whose-owes'));
  });

  it('R5: whose it is: each partner’s total with his share as the backend sent it, and a bar for it', async () => {
    await open();
    const whose = table('cashbook-capital-whose-title');
    const text = visible(whose.textContent);
    expect(text).toContain(
      m('cashbook-capital-whose-partner-total', { partner: nameOf(PARTNER_B) }),
    );
    for (const figure of ['746', '65,000', '65,488', '1.14%']) expect(text).toContain(figure);
    // Two partners have a share: one bar each, as long as the share says. Partner A has none, so no bar.
    const bars = whose.querySelectorAll('[aria-hidden="true"] > span');
    expect(bars).toHaveLength(2);
    expect((bars[0] as HTMLElement).style.inlineSize).toBe('1.14%');
  });

  it('R6: cash below zero is on the paying side as "cash short" with its note, and a net that is short says so in words, never a minus', async () => {
    await open(UMAR_BEFORE);
    const owe = visible(table('cashbook-capital-owe-title').textContent);
    expect(owe).toContain(m('cashbook-capital-where-cash-short'));
    // The cash line is named "cash short", not "cash in hand" (the note says "cash in hand" in words).
    expect(
      within(table('cashbook-capital-owe-title')).queryByRole('button', {
        name: named(m('cashbook-capital-where-cash')),
      }),
    ).toBeNull();
    expect(owe).toContain('161,740');
    expect(owe).toContain(m('cashbook-capital-cash-short-note'));
    // The net (96,530) is what the shop owes beyond what it holds: the figure and the word "short", in the headline
    // and under the blocks.
    const headline = screen.getByRole('heading', {
      level: 3,
      name: named(m('cashbook-capital-headline')),
    }).parentElement?.parentElement;
    expect(visible(headline?.textContent)).toContain('96,530');
    expect(visible(headline?.textContent)).toContain(m('cashbook-capital-net-owe'));
    expect(cardText()).not.toMatch(MINUS);
  });

  it('R7: the warning of the opening difference names 1,000,000 of partner capital and 65,000 of party openings, and lists every account', async () => {
    const { user } = await open(UMAR_BEFORE);
    const text = cardText();
    expect(text).toContain(m('cashbook-capital-warning-place'));
    expect(text).toContain(m('cashbook-capital-warning-sum-partner'));
    expect(text).toMatch(language === 'ur' ? /1,000,000\s*بنام/ : /1,000,000\s*Debit/);
    expect(text).toContain(m('cashbook-capital-warning-sum-party'));
    expect(text).toMatch(language === 'ur' ? /65,000\s*جمع/ : /65,000\s*Credit/);
    await showParts(user);
    const parts = visible(table('cashbook-capital-warning-parts').textContent);
    for (const figure of ['500,000', '30,000', '15,000', '60,000', '10,000', '20,000'])
      expect(parts).toContain(figure);
    expect(parts).toContain(language === 'ur' ? 'عدنان' : 'Adnan');
    expect(parts).toContain('29-10-2026');
    expect(parts).toContain(m('cashbook-capital-warning-part-partner'));
    expect(parts).toContain(m('cashbook-capital-warning-part-party'));
    expect(
      screen.getByRole('button', { name: named(m('cashbook-capital-warning-place-button')) }),
    ).toBeInTheDocument();
  });

  it('R8: a wrong sign is told in the box and nothing is read again; the right sign puts the capital into cash and the card and the book are read again', async () => {
    vi.mocked(cashbookApi.partnersPlaceOpeningCapital)
      .mockRejectedValueOnce(fail({ code: 'ERR_SIGN_WRONG', field: 'sign', values: {} }))
      .mockResolvedValueOnce({
        placed: [],
        skipped: [],
        cash_after_paisa: rupees(838_260),
        reopened_from: '2026-10-29',
        closed_up_to: '2026-11-19',
      });
    const { user } = await open(UMAR_BEFORE);
    await user.click(
      screen.getByRole('button', { name: named(m('cashbook-capital-warning-place-button')) }),
    );
    const box = await screen.findByRole('alertdialog');
    const reason = within(box).getByRole('combobox', {
      name: named(m('cashbook-capital-place-label')),
    });
    await user.type(reason, 'old books{Enter}');
    await user.keyboard('munshi{Enter}');
    // The wrong sign: the box says so under the field, the command was asked once, and nothing was read again.
    expect(await within(box).findByText(said(m('err-sign-wrong')))).toBeInTheDocument();
    expect(cashbookApi.partnersPlaceOpeningCapital).toHaveBeenCalledTimes(1);
    expect(cashbookApi.cashbookCapitalWhere).toHaveBeenCalledTimes(1);
    const readsBefore = vi.mocked(cashbookApi.cashbookGet).mock.calls.length;
    // The right sign: the command has the partners, the reason and the sign as typed; the card is read again.
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(UMAR_AFTER);
    const sign = within(box).getByRole('textbox', {
      name: named(message(language, 'edit-fix-sign-label')),
    });
    await user.clear(sign);
    await user.type(sign, 'adnan{Enter}');
    await waitFor(() => {
      expect(cashbookApi.partnersPlaceOpeningCapital).toHaveBeenCalledTimes(2);
    });
    expect(vi.mocked(cashbookApi.partnersPlaceOpeningCapital).mock.calls[1]?.[0]).toEqual({
      partner_ids: [1, 2],
      reason: 'old books',
      sign: 'adnan',
    });
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    await waitFor(() => {
      expect(visible(table('cashbook-capital-receive-title').textContent)).toContain('838,260');
    });
    // The card shows the old books' difference now, with no button; the cash book was read again too.
    expect(cardText()).toContain(m('cashbook-capital-warning-old-text'));
    expect(
      screen.queryByRole('button', { name: named(m('cashbook-capital-warning-place-button')) }),
    ).toBeNull();
    expect(vi.mocked(cashbookApi.cashbookGet).mock.calls.length).toBeGreaterThan(readsBefore);
    expect(await screen.findByText(said(m('cashbook-capital-placed-toast')))).toBeInTheDocument();
  });

  it('R9: after the fix the difference of the old books is a warning without a button, with its accounts', async () => {
    const { user } = await open(UMAR_AFTER);
    const text = cardText();
    // The title is the old books' difference with its figure and side, drawn by the screen (no raw tag left).
    expect(text).toContain(
      language === 'ur' ? 'پرانے کھاتوں کا فرق' : 'Difference in the old books',
    );
    expect(text).not.toMatch(/<\/?balance>/);
    expect(text).toContain(m('cashbook-capital-warning-old-text'));
    expect(text).toMatch(language === 'ur' ? /65,000\s*جمع/ : /65,000\s*Credit/);
    expect(
      screen.queryByRole('button', { name: named(m('cashbook-capital-warning-place-button')) }),
    ).toBeNull();
    await showParts(user);
    const parts = visible(table('cashbook-capital-warning-parts').textContent);
    expect(parts).not.toContain(m('cashbook-capital-warning-part-partner'));
    expect(parts).toContain('30,000');
    expect(visible(table('cashbook-capital-receive-title').textContent)).toContain('838,260');
  });

  it('R10: the fix can be done with the keyboard alone: focus the button, Enter opens the box, Esc closes it, no command is sent', async () => {
    const { user } = await open(UMAR_BEFORE);
    screen.getByRole('button', { name: named(m('cashbook-capital-warning-place-button')) }).focus();
    await user.keyboard('{Enter}');
    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(cashbookApi.partnersPlaceOpeningCapital).not.toHaveBeenCalled();
  });

  it('R11: a box without a reason or a sign sends nothing', async () => {
    const { user } = await open(UMAR_BEFORE);
    await user.click(
      screen.getByRole('button', { name: named(m('cashbook-capital-warning-place-button')) }),
    );
    const box = await screen.findByRole('alertdialog');
    await user.click(
      within(box).getByRole('button', { name: named(m('cashbook-capital-place-confirm')) }),
    );
    expect(cashbookApi.partnersPlaceOpeningCapital).not.toHaveBeenCalled();
    expect(within(box).getAllByText(m('err-required')).length).toBeGreaterThan(0);
  });

  it.each([
    ['cash', () => m('cashbook-capital-where-cash'), '/rokar'],
    ['buyers', () => m('cashbook-capital-where-buyers-owe'), '/dues'],
    ['advances', () => m('cashbook-capital-where-seller-advances'), '/dues/advances'],
    ['sellers', () => m('cashbook-capital-where-sellers-payable'), '/dues/sellers'],
  ] as const)('R12: the %s line opens its page', async (_name, label, path) => {
    const { user, router } = await open();
    await user.click(screen.getByRole('button', { name: named(label()) }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(path);
    });
    if (path === '/dues/sellers')
      expect(router.state.location.search).toMatchObject({ side: 'shop_owes' });
  });

  it('R12b: a charge owed opens the ledger of its own account', async () => {
    const { user, router } = await open();
    await user.click(screen.getByRole('button', { name: named(nameOf(LABOUR)) }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R13');
    });
    expect(router.state.location.search).toMatchObject({ party: 77 });
  });

  it('R13: a partner opens his statement', async () => {
    const first = await open();
    await first.user.click(
      screen.getByRole('button', {
        name: named(m('cashbook-capital-whose-partner-total', { partner: nameOf(PARTNER_B) })),
      }),
    );
    await waitFor(() => {
      expect(first.router.state.location.pathname).toBe('/partners/statement');
    });
    expect(first.router.state.location.search).toMatchObject({ partner: 2 });
  });

  it('R13b: the opening capital line opens the ledger of the opening capital account', async () => {
    const { user, router } = await open();
    await user.click(
      screen.getByRole('button', { name: named(m('cashbook-capital-whose-opening-capital')) }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R13');
    });
    expect(router.state.location.search).toMatchObject({ party: OPENING_CAPITAL_ACCOUNT.id });
  });

  it('R14: a day to read it as of is sent as typed, and "up to today" goes back to today', async () => {
    const { user } = await open();
    const box = screen.getByRole('textbox', { name: named(m('cashbook-capital-go-label')) });
    await user.type(box, '05-10-2026{Enter}');
    await waitFor(() => {
      expect(cashbookApi.cashbookCapitalWhere).toHaveBeenLastCalledWith({ as_of: '05-10-2026' });
    });
    await user.click(screen.getByRole('button', { name: named(m('cashbook-capital-go-today')) }));
    await waitFor(() => {
      expect(cashbookApi.cashbookCapitalWhere).toHaveBeenLastCalledWith({});
    });
    // A day after today is refused at the box and nothing is read.
    const calls = vi.mocked(cashbookApi.cashbookCapitalWhere).mock.calls.length;
    await user.type(box, '31-12-2099{Enter}');
    expect(vi.mocked(cashbookApi.cashbookCapitalWhere).mock.calls.length).toBe(calls);
  });

  it('R15: print opens the preview of the card as of the day, in the print language, and the page comes back', async () => {
    vi.mocked(cashbookApi.cashbookCapitalPrintData).mockResolvedValue(
      partnerMonthPrintData(language),
    );
    const { user } = await open();
    await user.click(
      screen.getByRole('button', { name: (name) => visible(name).startsWith(m('report-preview')) }),
    );
    await waitFor(() => {
      expect(cashbookApi.cashbookCapitalPrintData).toHaveBeenCalledWith({ language, paper: 'a4' });
    });
    await user.keyboard('{Escape}');
    await screen.findByRole('button', {
      name: (name) => visible(name).includes(m('cashbook-capital-title')),
    });
  });

  it('R16: a partner section that is locked asks for the code in the place of the figures', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockRejectedValue(
      fail({ code: 'ERR_PARTNER_LOCKED', field: null, values: {} }),
    );
    const { user } = await renderOpen();
    await user.click(foldButton());
    expect(
      await screen.findByText((text) => visible(text) === m('partners-code-box-text')),
    ).toBeInTheDocument();
    expect(screen.queryByRole('table', { name: m('cashbook-capital-receive-title') })).toBeNull();
    expect(
      screen.getByRole('heading', { level: 1, name: m('cashbook-title') }),
    ).toBeInTheDocument();
  });

  it('R17: another failure says so in words and can be tried again', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockRejectedValueOnce(
      fail({ code: 'ERR_INTERNAL', field: null, values: {} }),
    );
    const { user } = await renderOpen();
    await user.click(foldButton());
    expect(
      await screen.findByText((text) => visible(text) === m('cashbook-capital-failed')),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: m('common-retry') }));
    await screen.findByRole('table', { name: m('cashbook-capital-receive-title') });
    expect(cashbookApi.cashbookCapitalWhere).toHaveBeenCalledTimes(2);
  });

  it('R18: nothing recorded yet is a message with a button to put capital in, not a table of zeros', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(EMPTY_CARD);
    const { user, router } = await renderOpen();
    await user.click(foldButton());
    expect(await screen.findByText(said(m('cashbook-capital-empty')))).toBeInTheDocument();
    expect(screen.queryByRole('table', { name: m('cashbook-capital-receive-title') })).toBeNull();
    await user.click(
      screen.getByRole('button', { name: named(m('cashbook-capital-empty-action')) }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/partners/capital');
    });
  });

  it('R19: the main lines always show, also at 0; any other line only when it is not 0', async () => {
    await open({
      ...WORKED_DAY,
      have_lines: [
        line('cash', 'have', 58_484),
        line('buyers_owe', 'have', 21_564),
        line('seller_advances', 'have', 0),
        line('money_lent', 'have', 0),
        line('separate_khatas', 'have', 2_000),
      ],
    });
    const text = visible(table('cashbook-capital-receive-title').textContent);
    expect(text).toContain(m('cashbook-capital-where-seller-advances'));
    expect(text).toContain(m('cashbook-capital-where-separate-khatas'));
    expect(text).not.toContain(m('cashbook-capital-where-money-lent'));
  });

  it.each([
    ['before the fix', UMAR_BEFORE],
    ['after the fix', UMAR_AFTER],
    ['the worked day', WORKED_DAY],
  ])('R20: no message id, brace or tag is left on the card (%s)', async (_name, card) => {
    await open(card);
    const text = cardText();
    expect(text).not.toContain('cashbook-capital-');
    expect(text).not.toMatch(/[{}]/);
    expect(text).not.toMatch(/<\/?[a-z]+>/);
    expect(text).not.toContain('undefined');
  });

  it('R21: a language switch keeps the card open and writes the partners in the other language, with one read', async () => {
    await open();
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
