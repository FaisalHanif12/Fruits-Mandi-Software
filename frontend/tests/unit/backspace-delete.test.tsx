// D8 Bug 2 (plan 62, PRD FR-7.91, FR-7.9, NFR-USE-7): Backspace and Delete only delete. In a text field
// they delete text. On a grid cell that is not being typed in they clear an editable cell, like a
// spreadsheet, and a calculated cell does nothing. They never leave the screen and never open the
// "Save changes?" box.
//
// The cause (found in WebKit, the web view of the Mac app): with the focus on something that is not a
// text field (a grid cell, a button, the page), WebKit takes Backspace as "go back" in the history, and
// the leave guard of the screen then asks "Save changes?" (app/LeaveGuard.tsx). A test cannot make
// jsdom go back, so it checks what decides it: whether the key press was cancelled (`defaultPrevented`),
// because a browser goes back only for a key press that nobody cancelled.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  cellOf,
  focusedIn,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeTwoLines,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { renderGrid } from '../helpers/entry-grid';
import { fakePartners } from '../helpers/partners';
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

const SLOW_MS = { timeout: 8_000 };

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

/** Remembers every key press that reaches the document, to read afterwards whether it was cancelled. */
function recordKeys() {
  const seen: KeyboardEvent[] = [];
  const record = (event: KeyboardEvent) => {
    seen.push(event);
  };
  document.addEventListener('keydown', record, true);
  return {
    seen,
    named: (key: string) => seen.filter((event) => event.key === key),
    stop: () => {
      document.removeEventListener('keydown', record, true);
    },
  };
}

describe.each(LANGUAGES)('Backspace and Delete on the sale book (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** The sale book with unsaved work (a lot and two lines), the cursor on the rate of line 1. */
  async function openWithWork() {
    const view = await openBook(language);
    await typeHeaderAndLot(view.user, language);
    await typeTwoLines(view.user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'rate')).toBe('600');
    }, SLOW_MS);
    await view.user.click(cellOf(salesGrid(language), 0, 'rate'));
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:rate');
    }, SLOW_MS);
    return view;
  }

  it('the key press is cancelled on a grid cell, so the web view cannot go back, and no box opens', async () => {
    const { user } = await openWithWork();
    const keys = recordKeys();
    await user.keyboard('{Backspace}{Delete}');
    keys.stop();
    expect(keys.named('Backspace')).toHaveLength(1);
    expect(keys.named('Delete')).toHaveLength(1);
    // A browser goes back only for a key press that nobody cancelled.
    expect(keys.named('Backspace')[0]?.defaultPrevented).toBe(true);
    expect(keys.named('Delete')[0]?.defaultPrevented).toBe(true);
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('is cancelled on a calculated cell and on a button too: nothing changes, nothing is left', async () => {
    const { user } = await openWithWork();
    const keys = recordKeys();
    await user.click(cellOf(salesGrid(language), 0, 'buyerTotal'));
    await user.keyboard('{Backspace}');
    const button = salesGrid(language).querySelector<HTMLElement>('.row-remove-button');
    if (button === null) throw new Error('no remove button');
    button.focus();
    await user.keyboard('{Backspace}{Delete}');
    keys.stop();
    // Every Backspace was cancelled (Delete never goes back, so it need not be).
    expect(keys.named('Backspace')).toHaveLength(2);
    for (const event of keys.named('Backspace')) expect(event.defaultPrevented).toBe(true);
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('600');
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('clears an editable cell that is not being typed in, like a spreadsheet, and the totals follow', async () => {
    const { user, backend } = await openWithWork();
    await user.keyboard('{Backspace}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.seller_rate).toBe('');
    }, SLOW_MS);
    expect(textOf(salesGrid(language), 0, 'rate')).not.toBe('600');
    // Delete does the same on the quantity.
    await user.click(cellOf(salesGrid(language), 0, 'qty'));
    await user.keyboard('{Delete}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.qty).toBe('');
    }, SLOW_MS);
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('still deletes text in a text field: the key press is not cancelled there', async () => {
    const { user } = await openBook(language);
    await user.keyboard('LES-12');
    const truck = document.activeElement;
    expect(truck).toBeInstanceOf(HTMLInputElement);
    const keys = recordKeys();
    await user.keyboard('{Backspace}');
    keys.stop();
    expect((truck as HTMLInputElement).value).toBe('LES-1');
    expect(keys.named('Backspace')[0]?.defaultPrevented).toBe(false);
  });
});

describe.each(LANGUAGES)('Backspace and Delete on the shared entry grid (%s)', (language) => {
  it('clears a typed cell and a text cell; a calculated cell does nothing and the key is cancelled', async () => {
    const rig = await renderGrid(language, { rows: [{ qty: '50', rate: '600', note: 'x' }] });
    await rig.focusCell(0, 'rate');
    await rig.user.keyboard('{Backspace}');
    expect(rig.typed(0, 'rate')).toBe('');
    expect(rig.isEditing()).toBe(false);
    await rig.focusCell(0, 'qty');
    await rig.user.keyboard('{Delete}');
    expect(rig.typed(0, 'qty')).toBe('');
    await rig.focusCell(0, 'note');
    await rig.user.keyboard('{Backspace}');
    expect(rig.typed(0, 'note')).toBe('');
    // A calculated cell cannot be cleared, and the key is not left to the web view.
    const keys = recordKeys();
    await rig.focusCell(0, 'line');
    await rig.user.keyboard('{Backspace}{Delete}');
    keys.stop();
    for (const event of keys.seen) expect(event.defaultPrevented).toBe(true);
    expect(rig.isEditing()).toBe(false);
  });

  it('a Backspace while typing in a cell deletes text and keeps the cell open', async () => {
    const rig = await renderGrid(language, { rows: [{ qty: '50' }] });
    await rig.focusCell(0, 'qty');
    await rig.user.keyboard('123');
    expect(rig.isEditing()).toBe(true);
    await rig.user.keyboard('{Backspace}');
    expect(rig.isEditing()).toBe(true);
    await rig.user.keyboard('{Enter}');
    expect(rig.typed(0, 'qty')).toBe('12');
  });
});
