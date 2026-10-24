// Ctrl+Enter on the sale book (plan 14, PRD FR-7.57): save, print the documents ticked in "print
// after save" and open a new بکری. In the Urdu and the English app, against a fake backend. A print
// that fails never undoes the save, and F10 never prints.

import { waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import * as draftsApi from '../../src/shared/drafts/api';
import * as urduApi from '../../src/shared/urdu/api';
import {
  combo,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeTwoLines,
} from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { fakePrinting } from '../helpers/print';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

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
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(draftsApi.draftsGet).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

const pageText = () => visible(document.body.textContent);

describe.each(LANGUAGES)('Ctrl+Enter prints after the save (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** Types Event 1 and presses Ctrl+Enter. */
  async function saveAndPrint(options: Parameters<typeof fakePrinting>[0] = {}) {
    const printing = fakePrinting(options);
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    await user.keyboard('{Control>}{Enter}{/Control}');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    return { printing, backend, user };
  }

  it('prints the ticked documents of the saved بکری in the app language, on the printer, and opens a new بکری', async () => {
    const { printing } = await saveAndPrint();
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
      expect(pageText()).toContain(message(language, 'print-toast-printed'));
    });
    expect(await combo(language, 'bikri-header-truck')).toHaveValue('');
  });

  it('prints only the documents of the remembered list', async () => {
    const { printing } = await saveAndPrint({ settings: { print_after_save: ['seller_bill'] } });
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
    expect(printing.prints[0]?.docs).toEqual(['seller_bill']);
  });

  it('prints nothing when no document is ticked, and says nothing about it', async () => {
    const { printing } = await saveAndPrint({ settings: { print_after_save: [] } });
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-saved-toast', { number: 1 }));
    });
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(printing.prints).toHaveLength(0);
    expect(pageText()).not.toContain(message(language, 'print-toast-printed'));
  });

  it('says nothing when every document had nothing to print', async () => {
    const { printing } = await saveAndPrint({ print: () => ({ printed: [], pdf_paths: null }) });
    await waitFor(() => {
      expect(printing.prints).toHaveLength(1);
    });
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(pageText()).not.toContain(message(language, 'print-toast-printed'));
  });

  it('on a computer that is not Windows the save stays and the message says printing needs Windows', async () => {
    const printing = fakePrinting();
    vi.mocked(reportsApi.reportsPrint).mockRejectedValue(fail({ code: 'ERR_PRINT_WINDOWS_ONLY' }));
    const { backend, user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    await user.keyboard('{Control>}{Enter}{/Control}');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-print-windows-only'));
    });
    expect(backend.saves).toHaveLength(1);
    expect(pageText()).toContain(message(language, 'bikri-saved-toast', { number: 1 }));
    expect(printing.prints).toHaveLength(0);
    // A new, empty بکری is ready, as after any save.
    expect(await combo(language, 'bikri-header-truck')).toHaveValue('');
  });

  it('F10 saves and does not print', async () => {
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
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(printing.prints).toHaveLength(0);
  });
});
