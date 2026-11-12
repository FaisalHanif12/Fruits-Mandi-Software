// 1,000 sale lines in one lot (D8 plan 61, NFR-SCL-3, NFR-SPD-4, NFR-SPD-5, NFR-SPD-7). The real
// timings are taken in a real browser (the development page `#/dev/bikri?thousand`, written up in the
// plan); here the work that grew with the number of lines is held in check, with room for a slow
// machine, so a change that makes a key press, a move of the cursor or a summary slow again is caught:
//   - a move of the cursor builds no draft and asks no preview (it changes no row);
//   - the text of 1,000 lines and the draft of 1,000 lines are built in a fraction of a second;
//   - a typed quantity on a lot of 1,000 lines reaches the summary.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { parseSnapshot, serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import { buildDraft, draftKey } from '../../src/features/bikri/draft';
import { buildGridViews } from '../../src/features/bikri/grid-views';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fillThousandLines, THOUSAND, thousandAnswer } from '../helpers/bikri-thousand';
import { cellOf, openBook, salesGrid } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const SLOW_MS = { timeout: 15_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

/** The time a piece of work takes, in milliseconds. */
function timed(work: () => void): number {
  const start = performance.now();
  work();
  return performance.now() - start;
}

describe('the work that grows with the number of lines', () => {
  it('the draft of 1,000 lines is built once for each change of the rows, and moving the cursor changes none', () => {
    const stores = createBikriDraft('10-10-2026');
    fillThousandLines(stores);
    const lot = stores.lots.getState().rows[0];
    if (lot === undefined) throw new Error('no lot');
    const sales = stores.salesFor(lot.rowId);
    expect(sales.getState().rows).toHaveLength(THOUSAND);
    const before = stores.version.getState().value;
    // The cursor moves over 100 cells of the lines and of the lot: no row changes, so no new version.
    for (const row of sales.getState().rows.slice(0, 100)) {
      sales.getState().setFocus({ rowId: row.rowId, field: 'qty' });
    }
    stores.lots.getState().setFocus({ rowId: lot.rowId, field: 'seller' });
    expect(stores.version.getState().value).toBe(before);
    // A typed quantity is a new version.
    const first = sales.getState().rows[0];
    if (first === undefined) throw new Error('no line');
    sales.getState().updateTyped(first.rowId, 'qty', '11');
    expect(stores.version.getState().value).toBe(before + 1);
  });

  it('builds the draft of 1,000 lines and the text of their cells well inside a key press', () => {
    const stores = createBikriDraft('10-10-2026');
    fillThousandLines(stores);
    const built = { draft: buildDraft(stores) };
    expect(built.draft.lots[0]?.sales).toHaveLength(THOUSAND);
    const draftMs = timed(() => {
      built.draft = buildDraft(stores);
    });
    const lot = stores.lots.getState().rows[0];
    if (lot === undefined) throw new Error('no lot');
    const preview = thousandAnswer(built.draft);
    const keys = stores
      .salesFor(lot.rowId)
      .getState()
      .rows.map((row) => row.rowId);
    const viewMs = timed(() => {
      buildGridViews({
        preview,
        problems: [],
        lotKeys: [lot.rowId],
        selectedLot: lot.rowId,
        lineKeysByLot: [keys],
        showAll: false,
        lotLabel: () => '',
        grouping: 'international',
        say: (id) => id,
        words: (problem) => problem.item.code,
      });
    });
    // A browser does each in a few milliseconds; these limits are for a slow computer under load.
    expect(draftMs).toBeLessThan(150);
    expect(viewMs).toBeLessThan(300);
  });
});

describe('the draft as the backend sees it', () => {
  it('leaves out a blank row, so the app filling its quantity does not make a refusal old', () => {
    const stores = createBikriDraft('10-10-2026');
    fillThousandLines(stores, 3);
    const lot = stores.lots.getState().rows[0];
    if (lot === undefined) throw new Error('no lot');
    const sales = stores.salesFor(lot.rowId);
    const blank = sales.getState().addRow({});
    const before = draftKey(buildDraft(stores));
    // The app fills the blank row's quantity: not a change of the entry.
    sales.getState().updateTyped(blank, 'qty', '30');
    expect(draftKey(buildDraft(stores))).toBe(before);
    // A line with a rate is a line of the entry: changing it is a change.
    sales.getState().updateTyped(blank, 'rate', '600');
    expect(draftKey(buildDraft(stores))).not.toBe(before);
  });
});

describe.each(LANGUAGES)('a lot of 1,000 sale lines on the screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('opens with its lines, and a quantity typed on the first line reaches the summary, asking one preview', async () => {
    const stores = createBikriDraft('10-10-2026');
    fillThousandLines(stores);
    const snapshot = parseSnapshot(serialiseDraft(stores));
    if (snapshot === null) throw new Error('the draft is not a snapshot');
    usePendingRestore.getState().offer(snapshot);
    const { user, backend } = await openBook(language, thousandAnswer);
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'qty')).toBeInTheDocument();
    }, SLOW_MS);
    // The answer for the opened draft: the summary shows the sum of the quantities, 10,000 (a stand-in).
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain('10,000');
    }, SLOW_MS);
    await user.click(cellOf(salesGrid(language), 0, 'qty'));
    await new Promise((done) => setTimeout(done, 500));
    const asked = backend.previews.length;
    // Moving the cursor asks nothing.
    await user.click(cellOf(salesGrid(language), 0, 'qty'));
    await user.keyboard('{ArrowDown}{ArrowDown}{ArrowUp}');
    await new Promise((done) => setTimeout(done, 300));
    expect(backend.previews.length).toBe(asked);
    // A typed quantity asks one preview, and the summary follows (10,000 becomes 10,001).
    await user.click(cellOf(salesGrid(language), 0, 'qty'));
    await user.keyboard('11{Enter}');
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain('10,001');
    }, SLOW_MS);
    expect(backend.previews.length).toBe(asked + 1);
    expect(screen.queryAllByRole('alert')).toHaveLength(0);
  });
});
