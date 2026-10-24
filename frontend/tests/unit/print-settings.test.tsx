// The Printing tab of Settings (plan 14, PRD Section 21, A21), in the Urdu and the English app, by
// keyboard, against a fake backend: the paper of each document, the printer list, the copies (text,
// checked 1 to 99 by the backend, field `copies`), the Urdu digits, and the documents of "print
// after save". The four states: loading, error with a retry, an empty printer list, and the data.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as authApi from '../../src/features/auth/api';
import * as reportsApi from '../../src/features/reports/api';
import * as settingsApi from '../../src/features/settings/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePrinting } from '../helpers/print';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/urdu/api');

const named = (language: Language, id: string) => (name: string) =>
  visible(name) === message(language, id);

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(authApi.authLock).mockResolvedValue(null);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the Printing tab (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open() {
    const app = await renderApp('/settings?tab=printing');
    await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
    return { ...app, user: userEvent.setup() };
  }

  const saveButton = () =>
    screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) });
  const copiesField = () =>
    screen.getByRole('textbox', { name: named(language, 'print-settings-copies') });

  it('is a tab of the settings and shows what is saved (A4 bill, A5 slips, one copy, English digits)', async () => {
    fakePrinting({ printers: ['HP LaserJet', 'Epson TM-T20'] });
    await open();
    expect(
      await screen.findByRole('tab', { name: named(language, 'settings-tab-printing') }),
    ).toHaveAttribute('aria-selected', 'true');
    // One select for each document's paper, with the paper that is saved.
    const paper = async (docId: string) =>
      screen.findByRole('combobox', { name: named(language, docId) });
    expect(visible((await paper('print-doc-seller-bill')).textContent)).toContain(
      message(language, 'print-paper-a4'),
    );
    expect(visible((await paper('print-doc-buyer-slips')).textContent)).toContain(
      message(language, 'print-paper-a5'),
    );
    expect(visible((await paper('print-doc-pukhta-slip')).textContent)).toContain(
      message(language, 'print-paper-a5'),
    );
    expect(copiesField()).toHaveValue('1');
    expect(
      screen.getByRole('checkbox', { name: named(language, 'print-settings-urdu-digits') }),
    ).not.toBeChecked();
    expect(saveButton()).toBeDisabled();
  });

  it('shows the printers of the computer, with the default printer first', async () => {
    fakePrinting({ printers: ['HP LaserJet', 'Epson TM-T20'] });
    const { user } = await open();
    const printer = await screen.findByRole('combobox', {
      name: named(language, 'print-settings-printer'),
    });
    expect(visible(printer.textContent)).toContain(
      message(language, 'print-settings-printer-default'),
    );
    await user.click(printer);
    const names = (await screen.findAllByRole('option')).map((option) =>
      visible(option.textContent),
    );
    expect(names).toEqual([
      message(language, 'print-settings-printer-default'),
      'HP LaserJet',
      'Epson TM-T20',
    ]);
  });

  it('says so when the computer has no list of printers (empty state), and the default is used', async () => {
    fakePrinting({ printers: [] });
    await open();
    expect(
      await screen.findByText(message(language, 'print-settings-printer-none')),
    ).toBeInTheDocument();
  });

  it('keeps a saved printer in the list when it is not found now, and a failed list has a retry', async () => {
    fakePrinting({ settings: { printer: 'Old Printer' }, printers: [] });
    vi.mocked(reportsApi.reportsListPrinters).mockRejectedValueOnce(fail({ code: 'ERR_INTERNAL' }));
    const { user } = await open();
    expect(
      await screen.findByText(message(language, 'print-settings-printers-failed')),
    ).toBeInTheDocument();
    const printer = await screen.findByRole('combobox', {
      name: named(language, 'print-settings-printer'),
    });
    // The saved name stays, so opening the tab never changes the printer by itself.
    expect(visible(printer.textContent)).toContain('Old Printer');
    await user.click(screen.getByRole('button', { name: named(language, 'common-retry') }));
    await waitFor(() => {
      expect(reportsApi.reportsListPrinters).toHaveBeenCalledTimes(2);
    });
  });

  it('saves with F10: the copies go as typed text, the list and the paper as chosen', async () => {
    const printing = fakePrinting({ printers: ['HP LaserJet'] });
    const { user } = await open();
    await screen.findByRole('combobox', { name: named(language, 'print-doc-seller-bill') });
    // Paper of the buyer slips: the 80 mm roll.
    await user.click(
      screen.getByRole('combobox', { name: named(language, 'print-doc-buyer-slips') }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(language, 'print-paper-mm80') }),
    );
    // The printer.
    await user.click(
      screen.getByRole('combobox', { name: named(language, 'print-settings-printer') }),
    );
    await user.click(await screen.findByRole('option', { name: 'HP LaserJet' }));
    await user.clear(copiesField());
    await user.type(copiesField(), '2');
    await user.click(
      screen.getByRole('checkbox', { name: named(language, 'print-settings-urdu-digits') }),
    );
    // "Print after save": the پختہ slip is not wanted.
    const after = screen.getByRole('region', {
      name: named(language, 'print-settings-after-save'),
    });
    await user.click(
      within(after).getByRole('checkbox', { name: named(language, 'print-doc-pukhta-slip') }),
    );
    expect(saveButton()).toBeEnabled();
    press('F10');
    await waitFor(() => {
      expect(printing.saves).toHaveLength(1);
    });
    expect(printing.saves[0]).toEqual({
      paper_seller_bill: 'a4',
      paper_buyer_slips: 'mm80',
      paper_pukhta_slip: 'a5',
      paper_receipt_slip: 'a5',
      printer: 'HP LaserJet',
      copies: '2',
      urdu_digits: true,
      print_after_save: ['seller_bill', 'buyer_slips'],
    });
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(message(language, 'settings-saved'));
    });
    // The form starts again from what was saved: nothing left to save.
    await waitFor(() => {
      expect(saveButton()).toBeDisabled();
    });
  });

  it('the receipt slip of a voucher has its own paper and its own place in "print after save"', async () => {
    const printing = fakePrinting({ printers: [] });
    const { user } = await open();
    await user.click(
      await screen.findByRole('combobox', { name: named(language, 'print-doc-receipt-slip') }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(language, 'print-paper-mm80') }),
    );
    const after = screen.getByRole('region', {
      name: named(language, 'print-settings-after-save'),
    });
    await user.click(
      within(after).getByRole('checkbox', { name: named(language, 'print-doc-receipt-slip') }),
    );
    press('F10');
    await waitFor(() => {
      expect(printing.saves).toHaveLength(1);
    });
    expect(printing.saves[0]).toMatchObject({
      paper_receipt_slip: 'mm80',
      paper_seller_bill: 'a4',
      print_after_save: ['seller_bill', 'buyer_slips', 'pukhta_slip', 'receipt_slip'],
    });
  });

  it('the default printer is saved as an empty name', async () => {
    const printing = fakePrinting({
      settings: { printer: 'HP LaserJet' },
      printers: ['HP LaserJet'],
    });
    const { user } = await open();
    await user.click(
      await screen.findByRole('combobox', { name: named(language, 'print-settings-printer') }),
    );
    await user.click(
      await screen.findByRole('option', {
        name: named(language, 'print-settings-printer-default'),
      }),
    );
    press('F10');
    await waitFor(() => {
      expect(printing.saves).toHaveLength(1);
    });
    expect(printing.saves[0]?.printer).toBe('');
  });

  it('shows the backend refusal of the copies under the field, in plain words', async () => {
    fakePrinting();
    const { user } = await open();
    await screen.findByRole('combobox', { name: named(language, 'print-doc-seller-bill') });
    vi.mocked(settingsApi.settingsSetPrinting).mockRejectedValueOnce(
      fail({ code: 'ERR_OUT_OF_RANGE', field: 'copies', values: { min: 1, max: 99 } }),
    );
    await user.clear(copiesField());
    await user.type(copiesField(), '100');
    press('F10');
    const field = copiesField().closest('div')?.parentElement;
    await waitFor(() => {
      expect(visible(field?.textContent)).toContain('99');
    });
    expect(visible(field?.textContent)).not.toContain('ERR_OUT_OF_RANGE');
    expect(copiesField()).toHaveAttribute('aria-invalid', 'true');
  });

  it('can be done with the keyboard only', async () => {
    const printing = fakePrinting();
    const { user } = await open();
    await user.click(
      await screen.findByRole('checkbox', { name: named(language, 'print-settings-urdu-digits') }),
    );
    await user.keyboard('{Control>}s{/Control}');
    await waitFor(() => {
      expect(printing.saves).toHaveLength(1);
    });
    expect(printing.saves[0]?.urdu_digits).toBe(true);
  });

  it('shows a plain error with a retry when the settings do not load, and the data after it', async () => {
    fakePrinting();
    vi.mocked(settingsApi.settingsGetPrinting).mockRejectedValueOnce(
      fail({ code: 'ERR_INTERNAL' }),
    );
    const { user } = await open();
    expect(
      await screen.findByText(message(language, 'print-settings-load-failed')),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: named(language, 'common-retry') }));
    expect(copiesField()).toBeInTheDocument();
  });

  it('shows a loading state while the settings come', async () => {
    fakePrinting();
    let release: (() => void) | undefined;
    vi.mocked(settingsApi.settingsGetPrinting).mockImplementation(
      () =>
        new Promise((resolve) => {
          release = () => {
            resolve({
              paper_seller_bill: 'a4',
              paper_buyer_slips: 'a5',
              paper_pukhta_slip: 'a5',
              paper_receipt_slip: 'a5',
              printer: '',
              copies: 1,
              urdu_digits: false,
              print_after_save: [],
            });
          };
        }),
    );
    await open();
    expect(await screen.findByRole('status')).toHaveAttribute('aria-busy', 'true');
    release?.();
    expect(
      await screen.findByRole('combobox', { name: named(language, 'print-doc-seller-bill') }),
    ).toBeInTheDocument();
  });
});
