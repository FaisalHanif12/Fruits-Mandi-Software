// The print box (plan 14, PRD FR-12.12, FR-7.36, FR-7.57), in the Urdu and the English app, against
// a fake backend. F9 opens it on the saved بکری on screen (or the one saved last, on a new one).
// It asks for the print language, the documents, one buyer or all, the copies and printer or PDF;
// it sends exactly that to `reports_print`, remembers the ticked documents, shows the printer
// errors in plain words, and offers the in-app preview where printing is not available.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as draftsApi from '../../src/shared/drafts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import {
  named,
  typeHeaderAndLot,
  typeTwoLines,
  openBook,
  salesGrid,
  textOf,
} from '../helpers/bikri-screen';
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
import { fakePartners } from '../helpers/partners';
import { fakePrinting } from '../helpers/print';
import { message, visible } from '../helpers/render';
import { IDS } from '../helpers/bikri-data';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
  vi.mocked(draftsApi.draftsGet).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

const other = (language: Language): Language => (language === 'ur' ? 'en' : 'ur');
const pageText = () => visible(document.body.textContent);

/** Opens the saved بکری 1 and the print box with F9. */
async function openBox(language: Language) {
  const printing = fakePrinting();
  fakeBikri();
  await renderApp('/bikri/book?id=41');
  await screen.findByRole('heading', {
    level: 1,
    name: named(message(language, 'bikri-saved-title', { number: 1 })),
  });
  const user = userEvent.setup();
  press('F9');
  const box = await screen.findByRole('dialog');
  // The box reads the settings first: it shows the form when they have arrived.
  await within(box).findByRole('checkbox', {
    name: named(message(language, 'print-doc-seller-bill')),
  });
  return { printing, user, box };
}

/** A button of the box by its words (the box has no key chips). */
const button = (scope: HTMLElement, id: string) =>
  within(scope).getByRole('button', {
    name: named(message(scope.lang === 'en' ? 'en' : 'ur', id)),
  });

describe.each(LANGUAGES)('the print box (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('F9 opens it on the saved بکری: the title has its number, the documents are the remembered ones, the language is the app language', async () => {
    const { box } = await openBox(language);
    expect(visible(within(box).getByRole('heading').textContent)).toBe(
      message(language, 'print-box-title', { number: 1 }),
    );
    for (const id of ['print-doc-seller-bill', 'print-doc-buyer-slips', 'print-doc-pukhta-slip']) {
      expect(
        within(box).getByRole('checkbox', { name: named(message(language, id)) }),
      ).toBeChecked();
    }
    const pressed = within(box)
      .getAllByRole('button', { pressed: true })
      .map((item) => item.textContent);
    expect(pressed[0]).toBe(language === 'ur' ? 'اردو' : 'English');
    // The key that prints after a save is told in the help line.
    expect(visible(box.textContent)).toContain('Ctrl+Enter');
  });

  it('Print sends exactly what the box shows, and says what was sent', async () => {
    const { box, user, printing } = await openBox(language);
    await user.click(button(box, 'print-box-print'));
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
    expect(printing.prints[0]).toEqual({
      bikri_id: 41,
      docs: ['seller_bill', 'buyer_slips', 'pukhta_slip'],
      language,
      copies: null,
      to: 'printer',
      only_buyer_id: null,
    });
    await waitFor(() => {
      expect(visible(box.textContent)).toContain(
        message(language, 'print-box-done-printed', {
          docs: new Intl.ListFormat(language, { style: 'long', type: 'conjunction' }).format([
            message(language, 'print-doc-seller-bill'),
            message(language, 'print-doc-buyer-slips'),
            message(language, 'print-doc-pukhta-slip'),
          ]),
        }),
      );
    });
  });

  it('prints in the other language without changing the app language (PRD FR-7.36)', async () => {
    const { box, user, printing } = await openBox(language);
    const otherName = other(language) === 'ur' ? 'اردو' : 'English';
    await user.click(within(box).getByRole('button', { name: otherName }));
    await user.click(button(box, 'print-box-print'));
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
    expect(printing.prints[0]?.language).toBe(other(language));
    expect(useLanguageStore.getState().language).toBe(language);
    expect(document.documentElement.lang).toBe(language);
  });

  it('sends the copies as typed text, and shows a refusal under the copies field', async () => {
    const { box, user, printing } = await openBox(language);
    const copies = within(box).getByRole('textbox', {
      name: named(message(language, 'print-box-copies')),
    });
    expect(copies).toHaveAttribute('placeholder', '1');
    await user.type(copies, '120');
    vi.mocked((await import('../../src/features/reports/api')).reportsPrint).mockRejectedValueOnce(
      fail({ code: 'ERR_OUT_OF_RANGE', field: 'copies', values: { min: 1, max: 99 } }),
    );
    await user.click(button(box, 'print-box-print'));
    await waitFor(() => {
      expect(visible(copies.closest('div')?.parentElement?.textContent)).toContain('99');
    });
    expect(printing.prints).toHaveLength(0);
    // The words are in the Field, once; the box does not repeat them above.
    expect(within(box).queryAllByRole('alert')).toHaveLength(0);
    await user.clear(copies);
    await user.type(copies, '3');
    await user.click(button(box, 'print-box-print'));
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
    expect(printing.prints[0]?.copies).toBe('3');
  });

  it('remembers the ticked documents as the list of "print after save"', async () => {
    const { box, user, printing } = await openBox(language);
    await user.click(
      within(box).getByRole('checkbox', {
        name: named(message(language, 'print-doc-buyer-slips')),
      }),
    );
    await user.click(button(box, 'print-box-print'));
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
    expect(printing.prints[0]?.docs).toEqual(['seller_bill', 'pukhta_slip']);
    expect(printing.saves).toEqual([
      {
        paper_seller_bill: 'a4',
        paper_buyer_slips: 'a5',
        paper_pukhta_slip: 'a5',
        paper_receipt_slip: 'a5',
        printer: '',
        copies: '1',
        urdu_digits: false,
        print_after_save: ['seller_bill', 'pukhta_slip'],
      },
    ]);
    expect(printing.settings().print_after_save).toEqual(['seller_bill', 'pukhta_slip']);
  });

  it('does not save the settings when the documents are the same ones', async () => {
    const { box, user, printing } = await openBox(language);
    await user.click(button(box, 'print-box-print'));
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
    expect(printing.saves).toHaveLength(0);
  });

  it('prints the slip of one buyer when one is chosen', async () => {
    const { box, user, printing } = await openBox(language);
    const choose = within(box).getByRole('combobox', {
      name: named(message(language, 'print-box-only-buyer')),
    });
    await user.click(choose);
    await user.click(
      await screen.findByRole('option', {
        name: named(language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani'),
      }),
    );
    await user.click(button(box, 'print-box-print'));
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
    expect(printing.prints[0]?.only_buyer_id).toBe(IDS.arshad);
  });

  it('PDF sends `pdf` and lists the files that were made', async () => {
    const { box, user, printing } = await openBox(language);
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'print-box-to-pdf')) }),
    );
    await user.click(button(box, 'print-box-save-pdf'));
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
    expect(printing.prints[0]?.to).toBe('pdf');
    await waitFor(() => {
      expect(visible(box.textContent)).toContain(message(language, 'print-box-done-pdf'));
    });
    expect(visible(box.textContent)).toContain(`prints/seller_bill-1-${language}.pdf`);
  });

  it('explains a computer that is not Windows, and the preview shows the same documents in the print language', async () => {
    const reports = await import('../../src/features/reports/api');
    const { box, user, printing } = await openBox(language);
    vi.mocked(reports.reportsPrint).mockRejectedValueOnce(fail({ code: 'ERR_PRINT_WINDOWS_ONLY' }));
    await user.click(button(box, 'print-box-print'));
    const alert = await within(box).findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-print-windows-only'));
    // The preview is one click away and shows the three documents as they would print.
    await user.click(button(box, 'print-box-preview'));
    await waitFor(() => {
      expect(printing.dataRequests.map((request) => request.doc)).toEqual([
        'seller_bill',
        'buyer_slips',
        'pukhta_slip',
      ]);
    });
    expect(printing.dataRequests.every((request) => request.language === language)).toBe(true);
    await waitFor(() => {
      expect(visible(box.textContent)).toContain('19,316');
    });
    expect(box.querySelectorAll('.print-doc')).toHaveLength(3);
    // The app itself keeps its language.
    expect(document.documentElement.lang).toBe(language);
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'common-back')) }),
    );
    expect(
      await within(box).findByRole('checkbox', {
        name: named(message(language, 'print-doc-seller-bill')),
      }),
    ).toBeInTheDocument();
  });

  it('previews in the other print language when that is chosen', async () => {
    const { box, user, printing } = await openBox(language);
    await user.click(
      within(box).getByRole('button', { name: other(language) === 'ur' ? 'اردو' : 'English' }),
    );
    await user.click(button(box, 'print-box-preview'));
    await waitFor(() => {
      expect(printing.dataRequests).toHaveLength(3);
    });
    expect(printing.dataRequests.every((request) => request.language === other(language))).toBe(
      true,
    );
    const sheet = await waitFor(() => {
      const found = box.querySelector<HTMLElement>('.print-doc');
      if (found === null) throw new Error('not yet');
      return found;
    });
    expect(sheet.lang).toBe(other(language));
  });

  it('names the printer that was not found', async () => {
    const reports = await import('../../src/features/reports/api');
    const { box, user } = await openBox(language);
    vi.mocked(reports.reportsPrint).mockRejectedValueOnce(
      fail({ code: 'ERR_PRINTER_NOT_FOUND', values: { printer: 'HP LaserJet' } }),
    );
    await user.click(button(box, 'print-box-print'));
    const alert = await within(box).findByRole('alert');
    expect(visible(alert.textContent)).toContain('HP LaserJet');
  });

  it('says so, in plain words, when the print itself fails', async () => {
    const reports = await import('../../src/features/reports/api');
    const { box, user } = await openBox(language);
    vi.mocked(reports.reportsPrint).mockRejectedValueOnce(fail({ code: 'ERR_PRINT_FAILED' }));
    await user.click(button(box, 'print-box-print'));
    const alert = await within(box).findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-print-failed'));
    expect(visible(alert.textContent)).not.toContain('ERR_PRINT_FAILED');
  });

  it('with no document ticked, Print and Preview cannot be pressed and the box says why', async () => {
    const { box, user } = await openBox(language);
    for (const id of ['print-doc-seller-bill', 'print-doc-buyer-slips', 'print-doc-pukhta-slip']) {
      await user.click(within(box).getByRole('checkbox', { name: named(message(language, id)) }));
    }
    expect(visible(box.textContent)).toContain(message(language, 'print-box-none-ticked'));
    expect(button(box, 'print-box-print')).toBeDisabled();
    expect(button(box, 'print-box-preview')).toBeDisabled();
  });

  it('can be done with the keyboard only: F9, Tab to Print, Enter', async () => {
    const { box, user, printing } = await openBox(language);
    const target = button(box, 'print-box-print');
    for (let step = 0; step < 30 && document.activeElement !== target; step += 1) {
      await user.tab();
    }
    expect(document.activeElement).toBe(target);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
  });

  it('Esc closes the box and gives the focus back', async () => {
    const { user } = await openBox(language);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });

  it('the toolbar has a Print button with its key on it', async () => {
    fakePrinting();
    fakeBikri();
    await renderApp('/bikri/book?id=41');
    const toolbar = await screen.findByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'print-box-print')),
    });
    expect(visible(toolbar.textContent)).toContain('F9');
  });
});

describe.each(LANGUAGES)('the print box when the settings do not load (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the plain error with a retry, and works after it', async () => {
    const settingsApi = await import('../../src/features/settings/api');
    const printing = fakePrinting();
    fakeBikri();
    vi.mocked(settingsApi.settingsGetPrinting).mockRejectedValueOnce(
      fail({ code: 'ERR_INTERNAL' }),
    );
    await renderApp('/bikri/book?id=41');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-saved-title', { number: 1 })),
    });
    press('F9');
    const box = await screen.findByRole('dialog');
    expect(
      await within(box).findByText(message(language, 'print-settings-load-failed')),
    ).toBeInTheDocument();
    await userEvent
      .setup()
      .click(within(box).getByRole('button', { name: named(message(language, 'common-retry')) }));
    expect(
      await within(box).findByRole('checkbox', {
        name: named(message(language, 'print-doc-seller-bill')),
      }),
    ).toBeInTheDocument();
    expect(printing.prints).toHaveLength(0);
  });
});

describe.each(LANGUAGES)('F9 on a new بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('says to save first when nothing is saved yet', async () => {
    fakePrinting();
    await openBook(language);
    press('F9');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'print-save-first'));
    });
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('after a save, F9 prints the one just saved', async () => {
    const printing = fakePrinting();
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-saved-toast', { number: 1 }));
    });
    press('F9');
    const box = await screen.findByRole('dialog');
    await within(box).findByRole('checkbox', {
      name: named(message(language, 'print-doc-seller-bill')),
    });
    expect(visible(within(box).getByRole('heading').textContent)).toBe(
      message(language, 'print-box-title', { number: 1 }),
    );
    await user.click(button(box, 'print-box-print'));
    await waitFor(() => {
      expect(printing.prints[0]?.bikri_id).toBe(41);
    });
    expect(bikriApi.bikriSave).toHaveBeenCalledTimes(1);
  });
});
