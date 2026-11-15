// The second print button of the sale book, next to the seller-bill print (plan 68, plan 76 point 3, PRD
// FR-12.12, FR-19.15), in the Urdu and the English app, against a fake backend. Shift+F9 or its button
// open a box for the saved بکری on screen. It asks what to print, the buyer parchi (the default) or the
// overall sale print, and which lines, the whole day of the بکری (the default) or this بکری only. The box
// is the slips' own preview, so Print and Save as PDF send the same slips the preview was read with.
// Every figure on the slips is the backend's; this file checks only which slips were asked for and what
// was sent.
// The two buttons are drawn by `PrintAction`, which the sale book's toolbar holds; it is drawn here
// on its own, with the providers of the app, so the test is about the buttons and the box.

import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../src/app/providers';
import * as bikriApi from '../../src/features/bikri/api';
import { forgetMixedSaves, rememberMixedSave } from '../../src/features/bikri/print/mixed-save-ids';
import { PrintAction } from '../../src/features/bikri/print/PrintAction';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as draftsApi from '../../src/shared/drafts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { fakePrinting } from '../helpers/print';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import { fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const SLOW_MS = { timeout: 8_000 };

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
  forgetMixedSaves();
});

/** The saved بکری 41, number 1, as the sale book gives it to the print buttons. */
const SAVED = { id: 41, number: 1 };

/** The print buttons with the providers of the app: the data cache, the language and the toasts. */
function drawButtons(target: typeof SAVED | null) {
  fakePrinting();
  fakeBikri();
  fakeReports();
  return render(
    <AppProviders>
      <PrintAction target={target} />
    </AppProviders>,
  );
}

/** A button of the toolbar by its words and, after them, its key (the key chip is part of its name). */
const toolbarButton = (language: Language, id: string, key: string) =>
  screen.getByRole('button', {
    name: (name: string) => {
      const seen = visible(name);
      return seen.startsWith(message(language, id)) && seen.endsWith(key);
    },
  });

describe.each(LANGUAGES)('the second print of the sale book (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** Opens the box of the saved بکری 41 with its button. */
  async function openBox() {
    drawButtons(SAVED);
    const user = userEvent.setup();
    await user.click(toolbarButton(language, 'report-r5-title', 'Shift+F9'));
    const box = await screen.findByRole('dialog', undefined, SLOW_MS);
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsBuyerParchiPrintData)).toHaveBeenCalled();
    }, SLOW_MS);
    return { user, box };
  }

  const parchiAsked = () => vi.mocked(reportsApi.reportsBuyerParchiPrintData).mock.calls;
  const saleAsked = () => vi.mocked(reportsApi.reportsSalePrintData).mock.calls;

  it('has a button next to Print F9, named like the report, with its key on it', () => {
    drawButtons(SAVED);
    expect(toolbarButton(language, 'print-box-print', 'F9')).toBeInTheDocument();
    expect(toolbarButton(language, 'report-r5-title', 'Shift+F9')).toBeInTheDocument();
  });

  it('opens on the buyer parchi of the whole day of the بکری, in the print language and on A4', async () => {
    const { box } = await openBox();
    expect(visible(within(box).getAllByRole('heading')[0]?.textContent)).toBe(
      message(language, 'print-adhari-title'),
    );
    const pressed = within(box)
      .getAllByRole('button', { pressed: true })
      .map((button) => visible(button.textContent));
    expect(pressed).toContain(message(language, 'print-adhari-whole-day', { date: '10-10-2026' }));
    expect(pressed).toContain(message(language, 'print-adhari-kind-parchi'));
    expect(parchiAsked()).toHaveLength(1);
    expect(parchiAsked()[0]?.[0]).toEqual({
      date: '2026-10-10',
      bikri_id: null,
      bikri_ids: null,
      buyer_ids: null,
      sections: 'buyers',
      language,
      paper: 'a4',
    });
    // The slips are the backend's: several to a page, each in its own box, with the line for the وصولی.
    await waitFor(() => {
      expect(box.querySelectorAll('.print-parchi-slip').length).toBe(1);
    }, SLOW_MS);
    expect(box.querySelector('.print-parchi-collect .print-write-line')).not.toBeNull();
  });

  it('this بکری only asks again with its number, and the day brings the whole day back', async () => {
    const { box, user } = await openBox();
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'print-adhari-this-bikri', { number: 1 })),
      }),
    );
    await waitFor(() => {
      expect(parchiAsked()).toHaveLength(2);
    }, SLOW_MS);
    expect(parchiAsked()[1]?.[0]).toMatchObject({ date: '2026-10-10', bikri_id: 41 });
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'print-adhari-whole-day', { date: '10-10-2026' })),
      }),
    );
    await waitFor(() => {
      expect(parchiAsked()).toHaveLength(3);
    }, SLOW_MS);
    expect(parchiAsked()[2]?.[0]).toMatchObject({ bikri_id: null });
  });

  it('after a save with carried lots, "this save only" sends the new بکری and the old بکری of each carried lot', async () => {
    // The save made بکری 41 (number 1) and sold a lot of بکری 12 (number 3) and one of بکری 13 (number 5); two lots
    // of بکری 12 are one بکری in the print.
    const carried = (lot: number, bikri: number, number: number) => ({
      lot_id: lot,
      old_bikri_id: bikri,
      old_number: number,
      entry_id: 900 + lot,
    });
    rememberMixedSave({
      date: '2026-10-10',
      new_bikri_id: 41,
      new_bikri_number: 1,
      carried: [carried(7, 12, 3), carried(8, 12, 3), carried(9, 13, 5)],
      entry_ids: [901, 902, 903, 904],
      // The backend's own list: the new بکری, then the old بکری of each carried lot, with no duplicates.
      print_bikri_ids: [41, 12, 13],
      preview: {} as never,
    });
    const { box, user } = await openBox();
    // The option names every بکری of the save, and the plain "this بکری" option is not offered.
    expect(
      within(box).queryByRole('button', {
        name: named(message(language, 'print-adhari-this-bikri', { number: 1 })),
      }),
    ).toBeNull();
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'print-adhari-this-save', { numbers: '1, 3, 5' })),
      }),
    );
    await waitFor(() => {
      expect(parchiAsked()).toHaveLength(2);
    }, SLOW_MS);
    expect(parchiAsked()[1]?.[0]).toMatchObject({
      date: '2026-10-10',
      bikri_id: 41,
      bikri_ids: [41, 12, 13],
    });
    // The whole day sends none.
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'print-adhari-whole-day', { date: '10-10-2026' })),
      }),
    );
    await waitFor(() => {
      expect(parchiAsked()).toHaveLength(3);
    }, SLOW_MS);
    expect(parchiAsked()[2]?.[0]).toMatchObject({ bikri_id: null, bikri_ids: null });
  });

  it('a save with no new بکری keeps nothing, and a plain بکری is asked by its own id with no list', async () => {
    rememberMixedSave({
      date: '2026-10-10',
      new_bikri_id: null,
      new_bikri_number: null,
      carried: [{ lot_id: 7, old_bikri_id: 12, old_number: 3, entry_id: 907 }],
      entry_ids: [907],
      print_bikri_ids: [12],
      preview: {} as never,
    });
    const { box, user } = await openBox();
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'print-adhari-this-bikri', { number: 1 })),
      }),
    );
    await waitFor(() => {
      expect(parchiAsked()).toHaveLength(2);
    }, SLOW_MS);
    expect(parchiAsked()[1]?.[0]).toMatchObject({ bikri_id: 41, bikri_ids: null });
  });

  it('the overall sale print asks for the sellers and the buyers, and draws the sellers first', async () => {
    const { box, user } = await openBox();
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'print-adhari-kind-sale')),
      }),
    );
    await waitFor(() => {
      expect(saleAsked()).toHaveLength(1);
    }, SLOW_MS);
    expect(saleAsked()[0]?.[0]).toMatchObject({
      date: '2026-10-10',
      bikri_id: null,
      sections: 'both',
      language,
      paper: 'a4',
    });
    await waitFor(() => {
      expect(box.querySelectorAll('.print-parchi-slip').length).toBe(2);
    }, SLOW_MS);
    const parts = [...box.querySelectorAll('[data-part]')].map((part) =>
      part.getAttribute('data-part'),
    );
    expect(parts).toEqual(['sellers', 'buyers']);
    // One header for the whole print, and no line to write on: a sale print is not a collector's slip.
    expect(box.querySelectorAll('.print-header')).toHaveLength(1);
    expect(box.querySelector('.print-parchi-collect')).toBeNull();
  });

  it('Print and Save as PDF send the slips that were read, and say what happened', async () => {
    const { box, user } = await openBox();
    await waitFor(() => {
      expect(box.querySelectorAll('.print-parchi-slip').length).toBeGreaterThan(0);
    }, SLOW_MS);
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'report-print-pdf')) }),
    );
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsBuyerParchiPrint)).toHaveBeenCalledTimes(1);
    }, SLOW_MS);
    expect(vi.mocked(reportsApi.reportsBuyerParchiPrint).mock.calls[0]?.[0]).toMatchObject({
      slips: { date: '2026-10-10', bikri_id: null, sections: 'buyers', language, paper: 'a4' },
      to: 'pdf',
      copies: null,
    });
    expect(await within(box).findByRole('status', undefined, SLOW_MS)).toBeInTheDocument();
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'print-adhari-this-bikri', { number: 1 })),
      }),
    );
    await waitFor(() => {
      expect(box.querySelectorAll('.print-parchi-slip').length).toBeGreaterThan(0);
    }, SLOW_MS);
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'report-print-print')) }),
    );
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsBuyerParchiPrint)).toHaveBeenCalledTimes(2);
    }, SLOW_MS);
    expect(vi.mocked(reportsApi.reportsBuyerParchiPrint).mock.calls[1]?.[0]).toMatchObject({
      slips: { bikri_id: 41 },
      to: 'printer',
    });
  });

  it('Shift+F9 opens it and F9 opens the seller bill box, never both at once; Esc closes the box', async () => {
    drawButtons(SAVED);
    press('F9', { shiftKey: true });
    const box = await screen.findByRole('dialog', undefined, SLOW_MS);
    expect(visible(within(box).getAllByRole('heading')[0]?.textContent)).toBe(
      message(language, 'print-adhari-title'),
    );
    // With the box open, F9 does not stack the seller bill box on it.
    press('F9');
    expect(screen.getAllByRole('dialog')).toHaveLength(1);
    const user = userEvent.setup();
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    // F9 alone is still the seller bill.
    press('F9');
    const bill = await screen.findByRole('dialog', undefined, SLOW_MS);
    expect(visible(within(bill).getByRole('heading').textContent)).toBe(
      message(language, 'print-box-title', { number: 1 }),
    );
    expect(vi.mocked(bikriApi.bikriGet)).toHaveBeenCalled();
  });

  it('says to save first when no بکری is saved yet, and opens no box', async () => {
    drawButtons(null);
    const user = userEvent.setup();
    await user.click(toolbarButton(language, 'report-r5-title', 'Shift+F9'));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(message(language, 'print-save-first'));
    });
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});
