// The second print button of the sale book: ادھاری تفصیل (R5), next to the seller-bill print (plan 68,
// PRD FR-12.12, D8 Feature 8), in the Urdu and the English app, against a fake backend. Shift+F9 or its
// button open a box for the saved بکری on screen. It asks one thing: the whole day of the بکری (the
// default) or this بکری only. The box is the report's own slips preview, so Print and Save as PDF
// send R5 with `slips: true` and the same filters the preview was read with. Every figure on the
// slips is the backend's; this file checks only which lines were asked for and what was sent.
// The two buttons are drawn by `PrintAction`, which the sale book's toolbar holds; it is drawn here
// on its own, with the providers of the app, so the test is about the buttons and the box.

import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../src/app/providers';
import * as bikriApi from '../../src/features/bikri/api';
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

describe.each(LANGUAGES)('ادھاری تفصیل from the sale book (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** Opens the R5 box of the saved بکری 41 with its button. */
  async function openBox() {
    drawButtons(SAVED);
    const user = userEvent.setup();
    await user.click(toolbarButton(language, 'report-r5-title', 'Shift+F9'));
    const box = await screen.findByRole('dialog', undefined, SLOW_MS);
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsBuyerBillsPrintData)).toHaveBeenCalled();
    }, SLOW_MS);
    return { user, box };
  }

  const asked = () => vi.mocked(reportsApi.reportsBuyerBillsPrintData).mock.calls;

  it('has a button next to Print F9, named like the report, with its key on it', () => {
    drawButtons(SAVED);
    expect(toolbarButton(language, 'print-box-print', 'F9')).toBeInTheDocument();
    expect(toolbarButton(language, 'report-r5-title', 'Shift+F9')).toBeInTheDocument();
  });

  it('opens on the whole day of the بکری, asks R5 for that day with no بکری number, in the print language and on A4', async () => {
    const { box } = await openBox();
    expect(visible(within(box).getAllByRole('heading')[0]?.textContent)).toBe(
      message(language, 'print-adhari-title'),
    );
    const pressed = within(box)
      .getAllByRole('button', { pressed: true })
      .map((button) => visible(button.textContent));
    expect(pressed).toContain(message(language, 'print-adhari-whole-day', { date: '10-10-2026' }));
    expect(asked()).toHaveLength(1);
    expect(asked()[0]?.[0]).toMatchObject({
      id: 'R5',
      language,
      paper: 'a4',
      filters: { from: '2026-10-10', to: '2026-10-10', bikri_id: null },
    });
    // The slips are the backend's: the page shows what it sent, one sheet for each buyer.
    await waitFor(() => {
      expect(box.querySelectorAll('.print-sheet').length).toBeGreaterThan(0);
    }, SLOW_MS);
  });

  it('this بکری only asks again with the number of the بکری and no dates, and the day brings it back', async () => {
    const { box, user } = await openBox();
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'print-adhari-this-bikri', { number: 1 })),
      }),
    );
    await waitFor(() => {
      expect(asked()).toHaveLength(2);
    }, SLOW_MS);
    expect(asked()[1]?.[0]).toMatchObject({
      id: 'R5',
      filters: { from: '', to: '', bikri_id: 41 },
    });
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, 'print-adhari-whole-day', { date: '10-10-2026' })),
      }),
    );
    await waitFor(() => {
      expect(asked()).toHaveLength(3);
    }, SLOW_MS);
    expect(asked()[2]?.[0]).toMatchObject({
      filters: { from: '2026-10-10', to: '2026-10-10', bikri_id: null },
    });
  });

  it('Print and Save as PDF send the slips of R5 with the filters that were read, and say what happened', async () => {
    const { box, user } = await openBox();
    await waitFor(() => {
      expect(box.querySelectorAll('.print-sheet').length).toBeGreaterThan(0);
    }, SLOW_MS);
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'report-print-pdf')) }),
    );
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsPrintReport)).toHaveBeenCalledTimes(1);
    }, SLOW_MS);
    expect(vi.mocked(reportsApi.reportsPrintReport).mock.calls[0]?.[0]).toMatchObject({
      report: {
        id: 'R5',
        language,
        paper: 'a4',
        filters: { from: '2026-10-10', to: '2026-10-10', bikri_id: null },
      },
      slips: true,
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
      expect(box.querySelectorAll('.print-sheet').length).toBeGreaterThan(0);
    }, SLOW_MS);
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'report-print-print')) }),
    );
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsPrintReport)).toHaveBeenCalledTimes(2);
    }, SLOW_MS);
    expect(vi.mocked(reportsApi.reportsPrintReport).mock.calls[1]?.[0]).toMatchObject({
      report: { id: 'R5', filters: { from: '', to: '', bikri_id: 41 } },
      slips: true,
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
