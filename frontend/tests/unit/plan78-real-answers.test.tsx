// Plans 78 and 87 with the backend's own files (tests/fixtures, written by its tests and copied byte for byte): the
// card "where the capital is" draws the real answers, never a hand-made one. They are the worked day after Events 1
// to 5 (65,488 on both sides), Umar-shaped data before the fix (cash short, 1,000,000 of partner capital and 65,000
// of party openings against the opening difference), the same books after the fix, a card as of a day, and the real
// refusals (the partner section locked, a wrong sign, nothing to place). The fix is asked with the backend's own
// request. The commands and the values of the new refusals are in plan78-api-contract.test.ts.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { prepareCashbook } from '../helpers/cashbook-screen';
import { DAY, leaveAppWindow, renderApp } from '../helpers/app';
import * as real from '../helpers/plan78-real';
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

afterEach(() => {
  leaveAppWindow();
});

/** A minus sign before a figure: the card never shows one. */
const MINUS = /[−−]|(^|[\s(])-\d/;

describe.each(LANGUAGES)('where the capital is, with the real answers (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const named = (text: string) => (name: string) => visible(name) === text;

  beforeEach(() => {
    vi.clearAllMocks();
    prepareCashbook(language, [], { cash: 58_484, cashRows: [] });
  });

  async function openCard() {
    const app = await renderApp('/rokar');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-title') });
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).includes(m('cashbook-capital-title')),
      }),
    );
    return { ...app, user };
  }
  const table = (titleId: string) => screen.findByRole('table', { name: m(titleId) });
  const headline = () =>
    screen.getByRole('heading', { level: 3, name: named(m('cashbook-capital-headline')) })
      .parentElement?.parentElement;
  const cardText = () =>
    visible(
      screen
        .getByRole('button', {
          name: (name) => visible(name).includes(m('cashbook-capital-title')),
        })
        .closest('section')?.textContent,
    );

  it('the worked day: every line the backend sends, the net of 65,488 and no minus anywhere', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(real.CAPITAL_WHERE);
    await openCard();
    const have = visible((await table('cashbook-capital-receive-title')).textContent);
    for (const figure of ['58,484', '21,564', '80,048']) expect(have).toContain(figure);
    const owe = visible((await table('cashbook-capital-owe-title')).textContent);
    // What is owed for charges is one line for each account, named by it.
    for (const figure of ['10,160', '300', '4,000', '100', '14,560']) expect(owe).toContain(figure);
    expect(owe).toContain(language === 'ur' ? 'مزدوری کھاتہ' : 'Labour Payable');
    const whose = visible((await table('cashbook-capital-whose-title')).textContent);
    for (const figure of ['1,004', '746', '65,000', '65,488']) expect(whose).toContain(figure);
    expect(visible(headline()?.textContent)).toContain('65,488');
    expect(visible(headline()?.textContent)).toContain(m('cashbook-capital-equal'));
    expect(cardText()).not.toMatch(MINUS);
  });

  it('before the fix: cash short on the paying side, the warning with its two sums and the button, and the real request goes out as it is', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(real.CAPITAL_WHERE_UNPLACED);
    vi.mocked(cashbookApi.partnersPlaceOpeningCapital).mockResolvedValue(real.PLACE_ANSWER);
    const { user } = await openCard();
    const owe = visible((await table('cashbook-capital-owe-title')).textContent);
    expect(owe).toContain(m('cashbook-capital-where-cash-short'));
    expect(owe).toContain('161,740');
    const text = cardText();
    expect(text).toContain(m('cashbook-capital-warning-place'));
    expect(text).toMatch(language === 'ur' ? /1,000,000\s*بنام/ : /1,000,000\s*Debit/);
    expect(text).toMatch(language === 'ur' ? /65,000\s*جمع/ : /65,000\s*Credit/);
    expect(text).not.toMatch(MINUS);
    // With a difference standing apart the headline claims no ✓.
    expect(text).not.toContain(m('cashbook-capital-equal'));
    await user.click(
      screen.getByRole('button', { name: named(m('cashbook-capital-warning-place-button')) }),
    );
    const box = await screen.findByRole('alertdialog');
    await user.type(
      within(box).getByRole('combobox', { name: named(m('cashbook-capital-place-label')) }),
      `${real.PLACE_REQUEST.reason}{Enter}`,
    );
    await user.keyboard(`${real.PLACE_REQUEST.sign}{Enter}`);
    await waitFor(() => {
      expect(cashbookApi.partnersPlaceOpeningCapital).toHaveBeenCalledTimes(1);
    });
    // The partners are the two whose capital stands against the difference, the reason and the sign as typed.
    expect(cashbookApi.partnersPlaceOpeningCapital).toHaveBeenCalledWith(real.PLACE_REQUEST);
  });

  it('after the fix: the opening capital is in cash and the old books’ difference of 65,000 is left, with no button', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(real.CAPITAL_WHERE_PLACED);
    await openCard();
    const have = visible((await table('cashbook-capital-receive-title')).textContent);
    expect(have).toContain('838,260');
    const text = cardText();
    expect(text).toContain(
      language === 'ur' ? 'پرانے کھاتوں کا فرق' : 'Difference in the old books',
    );
    expect(text).toMatch(language === 'ur' ? /65,000\s*جمع/ : /65,000\s*Credit/);
    expect(text).not.toMatch(MINUS);
    expect(
      screen.queryByRole('button', { name: named(m('cashbook-capital-warning-place-button')) }),
    ).toBeNull();
    // Adnan and Usman have the same share of the partners' money.
    const whose = visible((await table('cashbook-capital-whose-title')).textContent);
    expect(whose).toContain('50%');
  });

  it('as of a day: the day is sent as typed, DD-MM-YYYY, exactly as the backend’s request', async () => {
    // Today is after the day asked: a day that is today is read as "today" (no day sent).
    vi.mocked(appApi.dayStatus).mockResolvedValue({ ...DAY, today: '2026-11-20' });
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(real.CAPITAL_WHERE);
    const { user } = await openCard();
    await table('cashbook-capital-receive-title');
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(real.CAPITAL_WHERE_AS_OF);
    const asOf = real.CAPITAL_WHERE_AS_OF_REQUEST.as_of ?? '';
    await user.type(
      screen.getByRole('textbox', { name: named(m('cashbook-capital-go-label')) }),
      `${asOf}{Enter}`,
    );
    await waitFor(() => {
      expect(cashbookApi.cashbookCapitalWhere).toHaveBeenLastCalledWith(
        real.CAPITAL_WHERE_AS_OF_REQUEST,
      );
    });
    expect(
      await screen.findByText(
        (text) => visible(text) === m('cashbook-capital-as-of', { date: asOf }),
      ),
    ).toBeInTheDocument();
  });

  it('a wrong sign and nothing to place are told in the box, in words', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(real.CAPITAL_WHERE_UNPLACED);
    vi.mocked(cashbookApi.partnersPlaceOpeningCapital)
      .mockRejectedValueOnce(new CommandError(real.PLACE_SIGN_WRONG))
      .mockRejectedValueOnce(new CommandError(real.PLACE_NOTHING));
    const { user } = await openCard();
    await user.click(
      await screen.findByRole('button', {
        name: named(m('cashbook-capital-warning-place-button')),
      }),
    );
    const box = await screen.findByRole('alertdialog');
    await user.type(
      within(box).getByRole('combobox', { name: named(m('cashbook-capital-place-label')) }),
      `${real.PLACE_REQUEST.reason}{Enter}`,
    );
    await user.keyboard('munshi{Enter}');
    expect(
      await within(box).findByText((text) => visible(text) === m('err-sign-wrong')),
    ).toBeInTheDocument();
    // Nothing was read again after the refusal, and the box is still open.
    expect(cashbookApi.cashbookCapitalWhere).toHaveBeenCalledTimes(1);
    await user.click(
      within(box).getByRole('button', { name: named(m('cashbook-capital-place-confirm')) }),
    );
    expect(
      await within(box).findByText((text) => visible(text) === m('err-nothing-to-place')),
    ).toBeInTheDocument();
  });

  it('print: the card is laid out by the backend, every word and name from Fluent and no raw message id is left', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(real.CAPITAL_WHERE_UNPLACED);
    vi.mocked(cashbookApi.cashbookCapitalPrintData).mockResolvedValue(
      language === 'ur' ? real.CAPITAL_PRINT_UR : real.CAPITAL_PRINT_EN,
    );
    const { user } = await openCard();
    await table('cashbook-capital-receive-title');
    await user.click(
      screen.getByRole('button', { name: (name) => visible(name).startsWith(m('report-preview')) }),
    );
    await waitFor(() => {
      expect(cashbookApi.cashbookCapitalPrintData).toHaveBeenCalledWith({ language, paper: 'a4' });
    });
    const titles = await screen.findAllByText((text) => visible(text) === m('print-capital-title'));
    expect(titles.length).toBeGreaterThan(0);
    const text = visible(document.body.textContent);
    // The lines, the groups, the net and the difference, in the print language.
    for (const id of [
      'print-capital-have-title',
      'print-capital-owe-title',
      'print-capital-line-cash-short',
      'print-capital-line-buyers-owe',
      'print-capital-net',
      'print-capital-whose-title',
      'print-capital-diff-title',
      'print-capital-diff-partner-capital',
      'print-capital-total',
    ]) {
      expect(text).toContain(m(id));
    }
    for (const figure of ['105,000', '161,740', '1,000,000', '65,000', '935,000']) {
      expect(text).toContain(figure);
    }
    expect(text).not.toContain('print-capital-');
    expect(text).not.toMatch(MINUS);
  });

  it('asks for the code in its own place while the partner section is locked', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockRejectedValue(
      new CommandError(real.CAPITAL_WHERE_LOCKED),
    );
    await openCard();
    expect(
      await screen.findByText((text) => visible(text) === m('partners-code-box-text')),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.queryByRole('table', { name: m('cashbook-capital-receive-title') })).toBeNull();
    });
  });
});
