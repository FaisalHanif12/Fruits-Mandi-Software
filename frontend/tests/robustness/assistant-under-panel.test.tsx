// Robustness: the screen under the question panel (D6 Section 6, attack 13; plan 50 R3). Owner:
// `robustness`.
//
// The Owner has a بکری open with 30 unsaved sale lines. He presses F1, asks a question, switches
// the language, follows a button of the answer (it opens a report, so the leave prompt shows),
// chooses "back", and is where he was. Nothing he typed is lost at any step: the lines, the truck,
// the answer in the panel and the half-typed next question.
//
// The backend is a fake: the order of the lines is what the draft holds, the answer is the real
// answer of the backend (a copy of its own test fixture), and no number is worked out here.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { parseSnapshot, serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import { LOT, ON, SALE } from '../../src/features/bikri/types';
import * as assistantApi from '../../src/features/assistant/api';
import { useAssistantStore } from '../../src/features/assistant/store';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as draftsApi from '../../src/shared/drafts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri, RAHIM } from '../helpers/bikri';
import { combo, salesGrid } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { NEW_CONVERSATION, REPLY_ANSWERED, STATUS_ON } from '../helpers/assistant-real';
import { boxOf, panelOf, setLanguage } from '../helpers/assistant-shell';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/assistant/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** The sale lines the screen shows now: the typed quantity and rate of every drawn row. */
function linesOnScreen(language: Language): string[] {
  const grid = salesGrid(language);
  return [...grid.querySelectorAll<HTMLElement>('.ag-row')]
    .sort((a, b) => Number(a.getAttribute('row-index')) - Number(b.getAttribute('row-index')))
    .map((row) => {
      const cell = (field: string) =>
        visible(row.querySelector(`.ag-cell[col-id="${field}"]`)?.textContent ?? '').trim();
      return `${row.getAttribute('row-index') ?? '?'}:${cell('qty')}@${cell('rate')}`;
    });
}

/** A draft of a truck, Rahim's lot of 60 نگ and 30 lines of 2 نگ each at their own rate. */
function thirtyLines(): void {
  const stores = createBikriDraft('10-10-2026');
  stores.header.getState().patch({ truckNo: 'LES-1234' });
  const lot = stores.lots.getState().rows[0];
  if (lot === undefined) throw new Error('no lot row');
  stores.lots.getState().patchRow(lot.rowId, {
    typed: { [LOT.qty]: '60', [LOT.sellerFilled]: ON },
    picked: { [LOT.seller]: { id: RAHIM.id, code: RAHIM.code, name_ur: 'رحیم', name_en: 'Rahim' } },
  });
  const sales = stores.salesFor(lot.rowId).getState();
  const first = sales.rows[0];
  if (first === undefined) throw new Error('no line row');
  sales.patchRow(first.rowId, { typed: { [SALE.qty]: '2', [SALE.rate]: '601' } });
  for (let line = 2; line <= 30; line += 1) {
    sales.addRow({ [SALE.qty]: '2', [SALE.rate]: String(600 + line) });
  }
  const snapshot = parseSnapshot(serialiseDraft(stores));
  if (snapshot === null) throw new Error('the seeded draft is not a snapshot');
  expect(Object.values(snapshot.sales).flat().length).toBeGreaterThanOrEqual(30);
  usePendingRestore.getState().offer(snapshot);
}

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(draftsApi.draftsGet).mockResolvedValue(null);
  vi.mocked(draftsApi.draftsSave).mockResolvedValue({ saved_at: '2026-10-10T07:30:00Z' });
  vi.mocked(assistantApi.assistantStatus).mockResolvedValue(STATUS_ON);
  vi.mocked(assistantApi.assistantAsk).mockResolvedValue(REPLY_ANSWERED);
  vi.mocked(assistantApi.listenSteps).mockResolvedValue(() => undefined);
  vi.mocked(assistantApi.assistantNewConversation).mockResolvedValue(NEW_CONVERSATION);
  usePendingRestore.setState({ snapshot: null });
  useAssistantStore.getState().reset();
});

afterEach(() => {
  useAssistantStore.getState().reset();
  leaveAppWindow();
});

describe.each(LANGUAGES)('the panel over a بکری with 30 unsaved lines (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    thirtyLines();
  });

  it('ask, switch the language, follow a button (the leave prompt shows), come back: nothing is lost', async () => {
    const { router } = await renderApp('/bikri/book');
    const user = userEvent.setup();
    expect(await combo(language, 'bikri-header-truck')).toHaveValue('LES-1234');
    await waitFor(() => {
      expect(linesOnScreen(language).length).toBeGreaterThan(0);
    });
    const before = linesOnScreen(language);
    expect(before[0]).toBe('0:2@601');

    // F1, a question, its answer.
    press('F1');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await user.type(boxOf(language), 'ikram ke total bakaya kitne hai{Enter}');
    await waitFor(() => {
      expect(useAssistantStore.getState().entries.at(-1)?.working).toBe(false);
    });
    expect(assistantApi.assistantAsk).toHaveBeenCalledTimes(1);
    await user.type(boxOf(language), 'half typed');

    // The language is switched in the middle: the lines, the truck, the answer and the half-typed
    // next question are all still there, now in the other language.
    setLanguage(other);
    await waitFor(() => {
      expect(linesOnScreen(other)).toEqual(before);
    });
    expect(await combo(other, 'bikri-header-truck')).toHaveValue('LES-1234');
    expect(boxOf(other)).toHaveValue('half typed');
    const answer = within(panelOf(other)).getByLabelText(message(other, 'assistant-answer-label'));
    expect(visible(answer.textContent)).toContain(
      other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    );

    // A button of the answer opens a report. The بکری is unsaved, so the leave prompt shows and
    // the screen does not move.
    const openReport = within(panelOf(other))
      .getAllByRole('button')
      .find((b) => visible(b.textContent).includes(message(other, 'assistant-open-report')));
    if (openReport === undefined) throw new Error('the answer has no button to open a report');
    await user.click(openReport);
    const prompt = await screen.findByRole('alertdialog');
    expect(visible(prompt.textContent)).toContain(message(other, 'layout-save-title'));
    expect(router.state.location.pathname).toBe('/bikri/book');

    // "Back": he stays. Everything is as it was.
    await user.click(
      within(prompt).getByRole('button', { name: new RegExp(message(other, 'common-back')) }),
    );
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
    expect(router.state.location.pathname).toBe('/bikri/book');
    expect(linesOnScreen(other)).toEqual(before);
    expect(await combo(other, 'bikri-header-truck')).toHaveValue('LES-1234');
    expect(boxOf(other)).toHaveValue('half typed');
    expect(
      within(panelOf(other)).getByLabelText(message(other, 'assistant-answer-label')),
    ).toBeInTheDocument();
    // Nothing was thrown away on the way, and no second question was sent.
    expect(draftsApi.draftsDiscard).not.toHaveBeenCalled();
    expect(assistantApi.assistantAsk).toHaveBeenCalledTimes(1);

    // F1 closes the panel and the بکری is still the unsaved one.
    press('F1');
    expect(panelOf(other)).toHaveAttribute('inert');
    expect(linesOnScreen(other)).toEqual(before);
  });
});
