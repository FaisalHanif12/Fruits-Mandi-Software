// The voucher screen shows what the REAL backend answers (plan 15, T4). The backend wrote each
// answer in its own test (tests/fixtures/*.json, copied, never edited). The screen is driven by
// those answers and by the refusals the backend really sent: the notes of a preview, the bills and
// payables (also when the backend cut the list to the oldest 200), the list with its total, the
// refusals at their fields, the cash box, the collection round, make regular and the receipt slips.
// In the Urdu and in the English app. The numbers are the PRD worked day (cash lent 3,000 to Akram,
// payment 4,316 to Rahim, receipt 10,000 from Akram).

import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as reportsApi from '../../src/features/reports/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { ReceiptSlipTemplate } from '../../src/features/vouchers/print/ReceiptSlipTemplate';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters, record } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import { AKRAM, ARSHAD, KARIM, PUKHTA_BUYER, RAHIM, fakeVouchers } from '../helpers/vouchers';
import * as real from '../helpers/vouchers-real';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));
const dropdown = (language: Language, id: string) =>
  screen.findByRole('combobox', { name: named(message(language, id)) });
const pageText = () => visible(document.body.textContent);

async function pick(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  fieldId: string,
  typed: string,
) {
  await user.click(await dropdown(language, fieldId));
  await user.keyboard(typed);
  await waitFor(() => {
    expect(screen.getAllByRole('option')).toHaveLength(1);
  });
  await user.keyboard('{Enter}');
}

async function panelText(language: Language, titleId: string): Promise<string> {
  const title = await screen.findByRole('heading', { name: named(message(language, titleId)) });
  return visible(title.closest('section')?.textContent);
}

/** The backend's refusal as the command wrapper rejects with it. */
const refused = (error: typeof real.REFUSED_NEEDS_REASON) => new CommandError(error);

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  const rows = [AKRAM, ARSHAD, PUKHTA_BUYER, RAHIM, KARIM];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({
    staff: [record('staff', 81, '0001', 'اسلم', 'Aslam')],
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the voucher screen with the real answers (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeVouchers();
  });

  async function open(type: 'receipt' | 'payment' | 'advance') {
    const app = await renderApp(`/bikri/voucher?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, `vouchers-${type}-title`),
    });
    return { ...app, user: userEvent.setup() };
  }
  const amountField = () => labelled(language, 'vouchers-field-amount');

  it('a receipt from a buyer: the balances, the bill it settles and the open bills, as the backend sends them', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_RECEIPT_BUYER);
    vi.mocked(vouchersApi.assistOpenBills).mockResolvedValue(real.OPEN_BILLS_AKRAM);
    const { user } = await open('receipt');
    await pick(user, language, 'vouchers-field-buyer', '1');
    await user.click(await amountField());
    await user.keyboard('10000');
    const bills = await panelText(language, 'vouchers-bills-title');
    expect(bills).toContain('12,880');
    expect(bills).toContain(message(language, 'vouchers-source-bikri', { number: 1 }));
    await waitFor(async () => {
      const text = await panelText(language, 'vouchers-bills-title');
      expect(text).toContain(message(language, 'vouchers-effect-part'));
      expect(text).toContain('10,000');
    });
    // The balance before and after are the backend's, and a receipt shows no cash note.
    expect(pageText()).toContain('2,880');
    expect(pageText()).not.toContain(message(language, 'common-cash-after', { amount: '0' }));
  });

  it('a payment to a seller with nothing payable: the warning with its numbers and the note that offers a پیشگی', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_PAYMENT_NOTHING_PAYABLE);
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', '030110002');
    await user.click(await amountField());
    await user.keyboard('1000');
    expect(
      await screen.findByText(
        (content) => visible(content) === message(language, 'vouchers-advance-hint'),
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: named(message(language, 'vouchers-make-advance')) }),
    ).toBeInTheDocument();
    expect(pageText()).toContain(
      message(language, 'err-payment-above-payable', { payable: '0', over: '1,000' }),
    );
    expect(pageText()).not.toContain('ERR_');
  });

  it('cash below zero: the amber note with the backend’s number, and the box asks before the save', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_PAYMENT_CASH_BELOW_ZERO);
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', '030110002');
    await user.click(await amountField());
    await user.keyboard('9900000');
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'common-cash-after', { amount: '-9,867,316' }),
      );
    });
    press('F10');
    const no = await screen.findByRole('button', {
      name: new RegExp(message(language, 'common-cash-confirm-no')),
    });
    await waitFor(() => {
      expect(no).toHaveFocus();
    });
    expect(vouchersApi.vouchersSave).not.toHaveBeenCalled();
    await user.click(no);
  });

  it('a seller returns advance money: his limit and the advance it adjusts', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_RECEIPT_SELLER_RETURN);
    vi.mocked(vouchersApi.assistAdvances).mockResolvedValue(real.ADVANCES_KARIM);
    const { user } = await open('receipt');
    const group = screen.getByRole('group', { name: message(language, 'vouchers-field-from') });
    await user.click(
      within(group).getByRole('button', { name: named(message(language, 'vouchers-from-seller')) }),
    );
    await pick(user, language, 'vouchers-field-seller', '030110002');
    await user.click(await amountField());
    await user.keyboard('2000');
    const advances = await panelText(language, 'vouchers-advances-title');
    expect(advances).toContain('5,000');
    await waitFor(() => {
      expect(pageText()).toContain('2,000');
      expect(pageText()).toContain(message(language, 'vouchers-return-limit'));
    });
  });

  it('a list the backend cut to the oldest 200 says so, and "full balance" still fills the whole amount', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_RECEIPT_BUYER);
    vi.mocked(vouchersApi.assistOpenBills).mockResolvedValue(real.OPEN_BILLS_TRUNCATED);
    const { user } = await open('receipt');
    await pick(user, language, 'vouchers-field-buyer', '1');
    const shown = String(real.OPEN_BILLS_TRUNCATED.bills.length);
    const total = String(real.OPEN_BILLS_TRUNCATED.total_count);
    expect(real.OPEN_BILLS_TRUNCATED.truncated).toBe(true);
    expect(shown).toBe('200');
    expect(
      await screen.findByText(
        (content) =>
          visible(content) === message(language, 'vouchers-truncated-note', { shown, total }),
      ),
    ).toBeInTheDocument();
    const panel = await panelText(language, 'vouchers-bills-title');
    expect(panel).toContain(message(language, 'vouchers-bills-open'));
  });

  it('payables the backend cut to the oldest 200', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_PAYMENT_SELLER);
    vi.mocked(vouchersApi.assistOpenPayables).mockResolvedValue(real.OPEN_PAYABLES_TRUNCATED);
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', '030110002');
    const shown = String(real.OPEN_PAYABLES_TRUNCATED.postings.length);
    const total = String(real.OPEN_PAYABLES_TRUNCATED.total_count);
    expect(
      await screen.findByText(
        (content) =>
          visible(content) === message(language, 'vouchers-truncated-note', { shown, total }),
      ),
    ).toBeInTheDocument();
  });

  it('the payables of Rahim as the backend sends them: credit, advance adjusted, paid, still open', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_PAYMENT_SELLER);
    vi.mocked(vouchersApi.assistOpenPayables).mockResolvedValue(real.OPEN_PAYABLES_RAHIM);
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', '030110001');
    const text = await panelText(language, 'vouchers-payables-title');
    for (const figure of ['24,316', '15,000', '5,000', '4,316']) expect(text).toContain(figure);
  });

  it('a refusal for cash lent with no reason is told at the reason field, in words', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_PAYMENT_BUYER);
    vi.mocked(vouchersApi.vouchersSave).mockRejectedValue(refused(real.REFUSED_NEEDS_REASON));
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', '03040001');
    await user.click(await amountField());
    await user.keyboard('3000');
    await user.click(await labelled(language, 'vouchers-field-reason-lent'));
    await user.keyboard('x');
    press('F10');
    expect(
      (
        await screen.findAllByText(
          (content) => visible(content) === message(language, 'err-payment-to-buyer-needs-reason'),
        )
      ).length,
    ).toBeGreaterThan(0);
    expect(pageText()).not.toContain('ERR_');
  });

  it('a seller returning too much is refused at the amount with the most he can return', async () => {
    vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_RECEIPT_SELLER_RETURN);
    vi.mocked(vouchersApi.vouchersSave).mockRejectedValue(refused(real.REFUSED_RETURN_TOO_MUCH));
    const { user } = await open('receipt');
    const group = screen.getByRole('group', { name: message(language, 'vouchers-field-from') });
    await user.click(
      within(group).getByRole('button', { name: named(message(language, 'vouchers-from-seller')) }),
    );
    await pick(user, language, 'vouchers-field-seller', '030110002');
    await user.click(await amountField());
    await user.keyboard('9000');
    press('F10');
    expect(
      (
        await screen.findAllByText(
          (content) =>
            visible(content) === message(language, 'err-return-exceeds-open', { open: '0' }),
        )
      ).length,
    ).toBeGreaterThan(0);
  });

  it('the refusal about cash opens the box with the cash after the entry, from a fresh preview', async () => {
    let tried = false;
    vi.mocked(vouchersApi.vouchersPreview).mockImplementation(() =>
      Promise.resolve(tried ? real.PREVIEW_PAYMENT_CASH_BELOW_ZERO : real.PREVIEW_PAYMENT_SELLER),
    );
    vi.mocked(vouchersApi.vouchersSave).mockImplementation(() => {
      tried = true;
      return Promise.reject(refused(real.REFUSED_CASH_BELOW_ZERO));
    });
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', '030110002');
    await user.click(await amountField());
    await user.keyboard('9900000');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenCalled();
    });
    press('F10');
    expect(
      await screen.findByRole('button', {
        name: new RegExp(message(language, 'common-cash-confirm-no')),
      }),
    ).toBeInTheDocument();
    expect(pageText()).toContain('-9,867,316');
  });

  it('the list of payments: a cancelled voucher is marked and left out of the backend’s total', async () => {
    vi.mocked(vouchersApi.vouchersList).mockResolvedValue(real.LIST_PAYMENTS);
    await open('payment');
    await waitFor(() => {
      expect(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).toHaveLength(2);
    });
    const cancelled = document.querySelector('.ag-row[row-id="4"] [col-id="status"]');
    expect(visible(cancelled?.textContent)).toBe(message(language, 'vouchers-state-cancelled'));
    const pinned = document.querySelector('.ag-row-pinned');
    expect(visible(pinned?.querySelector('[col-id="amount"]')?.textContent)).toBe('4,316');
    expect(visible(pinned?.querySelector('[col-id="account"]')?.textContent)).toBe(
      message(language, 'glossary-meezan'),
    );
  });

  it('a list with no total (all types) shows only the word', async () => {
    vi.mocked(vouchersApi.vouchersList).mockResolvedValue(real.LIST_ALL);
    await open('payment');
    await waitFor(() => {
      expect(document.querySelector('.ag-row-pinned')).not.toBeNull();
    });
    const pinned = document.querySelector('.ag-row-pinned');
    expect(visible(pinned?.querySelector('[col-id="amount"]')?.textContent)).toBe('');
  });
});

describe.each(LANGUAGES)(
  'the collection round and make regular with the real answers (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      fakeVouchers();
    });

    it('the list of the day has buyers with no collector or city; a saved row goes, a refused one keeps its words', async () => {
      vi.mocked(vouchersApi.assistCollectionList).mockResolvedValue(real.COLLECTION_SATURDAY);
      const [savedRow, , refusedRow] = real.RECEIPTS_BATCH.rows;
      const refusal = refusedRow?.errors[0];
      if (savedRow === undefined || refusedRow === undefined || refusal === undefined) {
        throw new Error('the batch fixture has no saved and refused row');
      }
      // The backend's rows, the refused one at the second place of this two-buyer round.
      vi.mocked(vouchersApi.vouchersSaveReceiptsBatch).mockResolvedValue({
        rows: [savedRow, { ...refusedRow, errors: [{ ...refusal, field: 'rows[1].amount' }] }],
        saved_paisa: real.RECEIPTS_BATCH.saved_paisa,
      });
      await renderApp('/bikri/collection');
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'vouchers-collection-title')),
      });
      await waitFor(() => {
        expect(document.querySelectorAll('.ag-row[row-id]')).toHaveLength(2);
      });
      // The backend's dues of the two buyers.
      expect(pageText()).toContain('3,880');
      expect(pageText()).toContain('13,684');
      const user = userEvent.setup();
      for (const rowIndex of [0, 1]) {
        const cell = document.querySelector<HTMLElement>(
          `.ag-row[row-index="${String(rowIndex)}"] [col-id="received"]`,
        );
        if (cell === null) throw new Error('no received cell');
        await user.click(cell);
        await user.keyboard('1000{Enter}');
      }
      press('F10');
      await waitFor(() => {
        expect(vouchersApi.vouchersSaveReceiptsBatch).toHaveBeenCalledTimes(1);
      });
      expect(
        (
          await screen.findAllByText((content) =>
            visible(content).includes(message(language, 'err-return-exceeds-open', { open: '0' })),
          )
        ).length,
      ).toBeGreaterThan(0);
    });

    it('make regular: the backend’s account takes the person’s place, and no opening was sent', async () => {
      const user = userEvent.setup();
      vi.mocked(vouchersApi.vouchersPreview).mockResolvedValue(real.PREVIEW_RECEIPT_BUYER);
      vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([
        {
          name_ur: 'شاہد',
          name_en: 'Shahid',
          phone: '03000000000',
          last_date: '2026-10-10',
          person_key: real.REQUEST_MAKE_REGULAR.person_key,
        },
      ]);
      vi.mocked(pukhtaApi.pukhtaMakeRegular).mockResolvedValue(real.MADE_REGULAR);
      await renderApp('/bikri/voucher?type=receipt');
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'vouchers-receipt-title'),
      });
      await pick(user, language, 'vouchers-field-buyer', '03049999');
      const person = await labelled(language, 'vouchers-field-person');
      await user.click(person);
      await user.keyboard('shah');
      await screen.findByRole('option');
      await user.keyboard('{Enter}');
      await user.click(
        await screen.findByRole('button', {
          name: named(message(language, 'vouchers-make-regular')),
        }),
      );
      const dialog = await screen.findByRole('dialog');
      await user.click(
        within(dialog).getByRole('button', {
          name: new RegExp(message(language, 'layout-save-save')),
        }),
      );
      await waitFor(() => {
        expect(pukhtaApi.pukhtaMakeRegular).toHaveBeenCalledTimes(1);
      });
      const sent = vi.mocked(pukhtaApi.pukhtaMakeRegular).mock.calls[0]?.[0];
      expect(sent).toMatchObject({
        person_key: real.REQUEST_MAKE_REGULAR.person_key,
        side: 'buyer',
        account_input: {
          account_type: 'buyer',
          name_ur: real.REQUEST_MAKE_REGULAR.account_input.name_ur,
          name_en: real.REQUEST_MAKE_REGULAR.account_input.name_en,
          phone1: real.REQUEST_MAKE_REGULAR.account_input.phone1,
          opening_dr: '',
          opening_cr: '',
          opening_is_advance: false,
          id: null,
          version: null,
        },
      });
      expect(await dropdown(language, 'vouchers-field-buyer')).toHaveValue(
        language === 'ur' ? real.MADE_REGULAR.account.name_ur : real.MADE_REGULAR.account.name_en,
      );
    });
  },
);

describe.each(LANGUAGES)('the receipt slips of the backend (%s)', (language) => {
  const slips = [
    ['a receipt in Urdu', real.SLIP_RECEIPT_UR, '10,000', 'دس ہزار روپے'],
    [
      'a payment in English',
      real.SLIP_PAYMENT_EN,
      '4,316',
      'Four thousand three hundred sixteen rupees only',
    ],
    ['an advance in Urdu', real.SLIP_ADVANCE_UR, '15,000', 'پندرہ ہزار روپے'],
    ['a receipt in English', real.SLIP_RECEIPT_EN, '5,000', 'Five thousand rupees only'],
  ] as const;

  it.each(slips)(
    '%s: every label has a message in both languages, and the page shows the facts',
    (_name, data, amount, words) => {
      const ids = new Set<string>([data.header.title_id]);
      for (const fact of data.header.facts) ids.add(fact.label_id);
      for (const page of data.pages) {
        ids.add(page.party.role_id);
        for (const row of page.amounts) ids.add(row.label_id);
        if (page.payment_id !== null) ids.add(page.payment_id);
        if (page.words !== null) ids.add(page.words.label_id);
      }
      for (const id of ids) {
        for (const other of LANGUAGES) expect(() => message(other, id)).not.toThrow();
      }
      // All money is روکڑ: the slip has no line about how it was paid, for any voucher.
      expect(data.pages.every((page) => page.payment_id === null)).toBe(true);
      const { container } = render(<ReceiptSlipTemplate data={data} />);
      const root = container.querySelector<HTMLElement>('.print-doc');
      expect(root?.querySelector('.print-payment')).toBeNull();
      if (root === null) throw new Error('no print document');
      const text = visible(root.textContent);
      expect(text).toContain(message(data.language, data.header.title_id));
      expect(text).toContain(amount);
      expect(text).toContain(words);
      for (const fact of data.header.facts) expect(text).toContain(fact.value);
      expect(root.getAttribute('lang')).toBe(data.language);
      expect(language).toBeDefined();
    },
  );
});
