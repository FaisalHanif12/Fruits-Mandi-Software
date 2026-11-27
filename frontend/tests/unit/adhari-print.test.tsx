// ادھاری تفصیل, the second print button of the sale book (plan 68, plan 78 A1, A5, PRD R5, FR-12.12), in the Urdu
// and the English app, against a fake backend. Shift+F9 or its button open a box for the saved بکری on screen.
// It prints only ادھاری تفصیل and only for the whole day: there is no choice of slips or of lines in it. The box
// is the table print's own preview, so Print and Save as PDF send the same day the preview was read with. Every
// figure is the backend's; this file checks only what was asked for and what was sent. The buttons are drawn by
// `PrintAction`, which the sale book's toolbar holds; it is drawn here on its own, with the providers of the
// app, so the test is about the button and the box.

import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../src/app/providers';
import { forgetMixedSaves } from '../../src/features/bikri/print/mixed-save-ids';
import { PrintAction } from '../../src/features/bikri/print/PrintAction';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
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
import { refusal } from '../helpers/ui-error';

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

describe.each(LANGUAGES)('ادھاری تفصیل, the second print of the sale book (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const sheetAsked = () => vi.mocked(reportsApi.reportsAdhariSheetPrintData).mock.calls;

  /** Opens the box of the saved بکری 41 with its button. */
  async function openBox() {
    drawButtons(SAVED);
    const user = userEvent.setup();
    await user.click(toolbarButton(language, 'report-r5-title', 'Shift+F9'));
    const box = await screen.findByRole('dialog', undefined, SLOW_MS);
    await waitFor(() => {
      expect(sheetAsked()).toHaveLength(1);
    }, SLOW_MS);
    return { user, box };
  }

  it('asks for the whole day of the بکری, in the print language and on A4, and offers no other choice', async () => {
    const { box } = await openBox();
    expect(visible(within(box).getAllByRole('heading')[0]?.textContent)).toBe(
      message(language, 'print-adhari-title'),
    );
    expect(sheetAsked()[0]?.[0]).toEqual({ date: '2026-10-10', language, paper: 'a4' });
    // No choice of slips and no choice of lines: it is the day's sheet and nothing else.
    for (const id of ['print-menu-buyers', 'print-menu-sellers', 'print-menu-sale']) {
      expect(within(box).queryByRole('button', { name: named(message(language, id)) })).toBeNull();
    }
    expect(
      within(box).queryByRole('button', {
        name: named(message(language, 'print-menu-this-bikri', { number: 1 })),
      }),
    ).toBeNull();
    expect(visible(box.textContent)).toContain(message(language, 'print-adhari-box-help'));
  });

  it('draws one table, a row for each buyer, never a page for a buyer, with the total credit at the end', async () => {
    const { box } = await openBox();
    await waitFor(() => {
      expect(box.querySelector('.print-doc')).not.toBeNull();
    }, SLOW_MS);
    expect(box.querySelectorAll('table')).toHaveLength(1);
    // The backend's own sheet of the worked day: ارشد and اکرم, one row each; the چکتا buyer شاہد is not on it (PRD R5 v1.57).
    // Under them the backend's row of today's buyers (their total, PRD R5 v1.59); nobody else owes, so no other row.
    expect(box.querySelectorAll('tbody tr[data-row="normal"]')).toHaveLength(2);
    expect(box.querySelectorAll('tbody tr[data-row="group"]')).toHaveLength(1);
    expect(box.querySelectorAll('tbody tr')).toHaveLength(3);
    // One header for the whole print and no slip of a buyer.
    expect(box.querySelectorAll('.print-header')).toHaveLength(1);
    expect(box.querySelector('.print-parchi-slip')).toBeNull();
    const text = visible(box.textContent);
    expect(text).toContain(message(language, 'print-adhari-sheet-all'));
    // The totals row reads 31,564 / 31,564 / 0 / 31,564: the bills, the credit of the day, the balance before and the total; there is no cash column or cash total (PRD R5 v1.57).
    const totals = [...(box.querySelector('tfoot tr')?.querySelectorAll('td') ?? [])].map((cell) =>
      visible(cell.textContent),
    );
    expect(totals.filter((cell) => cell !== '')).toEqual(expect.arrayContaining(['31,564', '0']));
    expect(text).toContain('31,564');
    // The names are in the print language: Urdu names on an Urdu print, English names on an English one.
    expect(text).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(text).not.toContain(language === 'ur' ? 'Akram Toll Tax' : 'اکرم ٹول ٹیکس');
  });

  it('Print and Save as PDF send the day that was read, with the language and the paper chosen', async () => {
    const { box, user } = await openBox();
    await waitFor(() => {
      expect(box.querySelector('.print-doc')).not.toBeNull();
    }, SLOW_MS);
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'print-paper-a5')) }),
    );
    await waitFor(() => {
      expect(sheetAsked().at(-1)?.[0]).toMatchObject({ paper: 'a5' });
    }, SLOW_MS);
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'report-print-pdf')) }),
    );
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsAdhariSheetPrint)).toHaveBeenCalledTimes(1);
    }, SLOW_MS);
    expect(vi.mocked(reportsApi.reportsAdhariSheetPrint).mock.calls[0]?.[0]).toEqual({
      sheet: { date: '2026-10-10', language, paper: 'a5' },
      to: 'pdf',
      copies: null,
    });
    expect(await within(box).findByRole('status', undefined, SLOW_MS)).toBeInTheDocument();
  });

  it('a day that fails to read says so, with a retry that reads it again', async () => {
    drawButtons(SAVED);
    vi.mocked(reportsApi.reportsAdhariSheetPrintData).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
    );
    const user = userEvent.setup();
    await user.click(toolbarButton(language, 'report-r5-title', 'Shift+F9'));
    const box = await screen.findByRole('dialog', undefined, SLOW_MS);
    expect(
      await within(box).findByText(message(language, 'report-print-failed'), undefined, SLOW_MS),
    ).toBeInTheDocument();
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'common-retry')) }),
    );
    await waitFor(() => {
      expect(box.querySelector('.print-doc')).not.toBeNull();
    }, SLOW_MS);
  });

  it('with the keyboard only: Shift+F9 opens it, Esc closes it', async () => {
    drawButtons(SAVED);
    press('F9', { shiftKey: true });
    await screen.findByRole('dialog', undefined, SLOW_MS);
    await userEvent.setup().keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
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
