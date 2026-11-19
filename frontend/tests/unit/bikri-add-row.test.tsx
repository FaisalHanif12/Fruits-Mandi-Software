// The round "+" under each entry grid of the sale book (Umar, plan 71 follow-up, PRD FR-7.17, NFR-SCL-3,
// F24, F27): "New lot" under the lots and "New line" under the sale lines. Each goes through the grid's
// one new-row path (the same as ↓ on the last row), so the store, the cursor and the keyboard rules are
// the same whichever was used. What is checked: the shared part on its own (name and tip from Fluent,
// the click, the dimmed state with its words at a limit, the end side, the size); the screen (a click
// adds a lot or a line and the cursor lands on the new row's first cell; in the whole-sale view the
// line goes to the lot of the line that had the cursor, else to the first lot; ↓ on the last row gives
// the same result; the button is reached and pressed with the keyboard; a language switch loses
// nothing); and the limits of 50 lots and 1,000 lines (offered at one under, dimmed at the limit).

import { act, screen, waitFor } from '@testing-library/react';
import fs from 'node:fs';
import path from 'node:path';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { parseSnapshot, serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import { MAX_LOTS, MAX_SALE_LINES } from '../../src/features/bikri/limits';
import { AddRowButton, addRowAndFocus, createEntryStore } from '../../src/shared/grid';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { cellOf, focusedIn, lotsGrid, named, salesGrid } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, renderIn, visible } from '../helpers/render';

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

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  leaveAppWindow();
});

describe.each(LANGUAGES)('the shared "+" under a grid (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  it('is a round button named and tipped by its Fluent message, and a click adds', async () => {
    const onAdd = vi.fn();
    renderIn(language, <AddRowButton labelId="bikri-lots-add-row" onAdd={onAdd} />);
    const button = screen.getByRole('button', { name: named(m('bikri-lots-add-row')) });
    expect(button).toHaveAttribute('title', button.getAttribute('aria-label'));
    expect(visible(button.getAttribute('title'))).toBe(m('bikri-lots-add-row'));
    // A small circle, not a pill with words: it holds a 16 px icon and no text.
    expect(button).toHaveClass('rounded-full');
    expect(button).toHaveClass('w-target');
    expect(button).toHaveClass('h-target');
    expect(button).not.toHaveClass('w-control');
    expect(visible(button.textContent)).toBe('');
    expect(button.querySelector('svg')).toHaveClass('size-icon-sm');
    await userEvent.setup().click(button);
    expect(onAdd).toHaveBeenCalledTimes(1);
    expect(button).toBeEnabled();
  });

  it('stands at the end side of its own row, with no physical left or right written', () => {
    const { container } = renderIn(
      language,
      <AddRowButton labelId="bikri-lots-add-row" onAdd={vi.fn()} />,
    );
    const row = container.querySelector('[data-add-row]');
    expect(row).not.toBeNull();
    expect(row?.className).toContain('justify-end');
    expect(row?.outerHTML).not.toMatch(
      /(^|[\s"])(left|right|ml-|mr-|pl-|pr-|text-left|text-right)/,
    );
  });

  it('at a limit is dimmed and says why in words beside it, linked to the button', async () => {
    const onAdd = vi.fn();
    renderIn(
      language,
      <AddRowButton
        labelId="bikri-sales-add-row"
        onAdd={onAdd}
        limit={{ messageId: 'bikri-sales-limit', vars: { max: '1,000' } }}
      />,
    );
    const button = screen.getByRole('button', { name: named(m('bikri-sales-add-row')) });
    expect(button).toBeDisabled();
    const reason = m('bikri-sales-limit', { max: '1,000' });
    const words = screen.getByText(
      (_, element) => element?.tagName === 'P' && visible(element.textContent) === reason,
    );
    expect(words.id).not.toBe('');
    expect(button.getAttribute('aria-describedby')).toBe(words.id);
    await userEvent.setup().click(button);
    expect(onAdd).not.toHaveBeenCalled();
  });
});

describe('the size of the "+"', () => {
  const tokens = fs.readFileSync(
    path.resolve(import.meta.dirname, '../../src/styles/tokens.css'),
    'utf8',
  );

  it('is 32 px square, the smallest target (NFR-USE-23), with a 16 px icon, both from the tokens', () => {
    expect(tokens).toMatch(/--target-min:\s*32px;/);
    expect(tokens).toMatch(/--spacing-target:\s*var\(--target-min\);/);
    expect(tokens).toMatch(/--spacing-icon-sm:\s*16px;/);
  });
});

describe('the one new-row path', () => {
  it('adds a row at the end and puts the cursor on the field of that row', () => {
    const store = createEntryStore();
    const added = addRowAndFocus(store, 'buyer');
    expect(store.getState().rows).toHaveLength(2);
    expect(store.getState().rows[1]?.rowId).toBe(added);
    expect(store.getState().pendingFocus).toEqual({ rowIndex: 'last', field: 'buyer' });
  });
});

/**
 * A draft with `lotRows.length` lots; the lot `i` has `lotRows[i]` sale rows (all blank). The sale
 * book opens on it, the way it opens after "finish the last بکری".
 */
function seedRows(lotRows: readonly number[]): void {
  const stores = createBikriDraft('10-10-2026');
  for (let index = 1; index < lotRows.length; index += 1) stores.lots.getState().addRow();
  stores.lots.getState().rows.forEach((lot, index) => {
    const sales = stores.salesFor(lot.rowId);
    while (sales.getState().rows.length < (lotRows[index] ?? 1)) sales.getState().addRow();
  });
  const snapshot = parseSnapshot(serialiseDraft(stores));
  if (snapshot === null) throw new Error('the seeded draft is not a snapshot');
  usePendingRestore.getState().offer(snapshot);
}

describe.each(LANGUAGES)('the "+" buttons of the sale book (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  beforeEach(() => {
    vi.resetAllMocks();
    healthyBackend();
    fakePartners();
    vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
    vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const open = async () => {
    await renderApp('/bikri/book');
    await screen.findByRole('heading', { level: 1, name: m('glossary-bikri-book') });
    await waitFor(() => {
      expect(lotsGrid(language).querySelector('.ag-row')).not.toBeNull();
    });
    return userEvent.setup();
  };
  const rowsOf = (grid: HTMLElement) => grid.querySelectorAll('.ag-row[row-index]').length;
  /** All rows of a grid, also those a tall grid does not draw: AG Grid counts them in `aria-rowcount`. */
  const allRowsOf = (grid: HTMLElement) =>
    Number(grid.querySelector('[aria-rowcount]')?.getAttribute('aria-rowcount') ?? '0') - 1;
  const lotButton = () => screen.getByRole('button', { name: named(m('bikri-lots-add-row')) });
  const lineButton = () => screen.getByRole('button', { name: named(m('bikri-sales-add-row')) });
  const wholeSwitch = async () => {
    const [first] = await screen.findAllByRole('button', {
      name: (name) => visible(name) === m('bikri-summary-whole'),
    });
    if (first === undefined) throw new Error('no switch');
    return first;
  };

  it('a click on "New lot" adds a lot and the cursor lands on its seller', async () => {
    seedRows([1, 1]);
    const user = await open();
    expect(rowsOf(lotsGrid(language))).toBe(2);
    await user.click(lotButton());
    await waitFor(() => {
      expect(rowsOf(lotsGrid(language))).toBe(3);
    }, SLOW_MS);
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('2:seller');
    }, SLOW_MS);
  });

  it('a click on "New line" adds a line to the lot shown and the cursor lands on its buyer', async () => {
    seedRows([2, 1]);
    const user = await open();
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(2);
    });
    await user.click(lineButton());
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(3);
    }, SLOW_MS);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('2:buyer');
    }, SLOW_MS);
  });

  it('↓ on the last line gives the same: a new line and the cursor in the same column', async () => {
    seedRows([2, 1]);
    const user = await open();
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(2);
    });
    await user.click(cellOf(salesGrid(language), 1, 'qty'));
    await user.keyboard('{ArrowDown}');
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(3);
    }, SLOW_MS);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('2:qty');
    }, SLOW_MS);
  });

  it('is reached with the keyboard and pressed with Enter', async () => {
    seedRows([1, 1]);
    const user = await open();
    lotButton().focus();
    expect(lotButton()).toHaveFocus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rowsOf(lotsGrid(language))).toBe(3);
    }, SLOW_MS);
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('2:seller');
    }, SLOW_MS);
  });

  it('in the whole-sale view the line goes to the lot of the line that had the cursor', async () => {
    seedRows([2, 3]);
    const user = await open();
    await user.click(await wholeSwitch());
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(5);
    }, SLOW_MS);
    // The cursor is on the second line of the second lot (the fourth row of the list).
    await user.click(cellOf(salesGrid(language), 3, 'buyer'));
    await user.click(lineButton());
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(6);
    }, SLOW_MS);
    // That lot is the last, so its new line is the last row; the cursor is on its buyer.
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('5:buyer');
    }, SLOW_MS);
  });

  it('in the whole-sale view with no cursor on a line the line goes to the first lot', async () => {
    seedRows([2, 3]);
    const user = await open();
    await user.click(await wholeSwitch());
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(5);
    }, SLOW_MS);
    await user.click(lineButton());
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(6);
    }, SLOW_MS);
    // The first lot had two lines, so its new line is the third row.
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('2:buyer');
    }, SLOW_MS);
  });

  it('a language switch after adding keeps the rows and the buttons are named in the other language', async () => {
    seedRows([1, 1]);
    const user = await open();
    await user.click(lineButton());
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(2);
    }, SLOW_MS);
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    await waitFor(() => {
      expect(rowsOf(salesGrid(other))).toBe(2);
    }, SLOW_MS);
    expect(
      screen.getByRole('button', { name: named(message(other, 'bikri-sales-add-row')) }),
    ).toBeEnabled();
    expect(
      screen.getByRole('button', { name: named(message(other, 'bikri-lots-add-row')) }),
    ).toBeEnabled();
  });

  // In Urdu the text reads on to the left, so ← is "toward the end"; in English it is →.
  const toEnd = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
  type Keys = Awaited<ReturnType<typeof open>>;
  /** Walks the cursor toward the end of the row it is on, until it is on the trash cell. */
  const reachTrash = async (user: Keys, grid: () => HTMLElement) => {
    for (
      let presses = 0;
      presses < 14 && !(focusedIn(grid()) ?? '').endsWith(':__remove');
      presses += 1
    ) {
      await user.keyboard(toEnd);
    }
    expect(focusedIn(grid())).toMatch(/^\d+:__remove$/);
  };

  it('keyboard only, lots: the arrows reach the trash cell, Enter removes the lot, ↓ reaches the "+", ↑ comes back, Enter adds', async () => {
    seedRows([1, 1]);
    const user = await open();
    await user.click(cellOf(lotsGrid(language), 1, 'qty'));
    await reachTrash(user, () => lotsGrid(language));
    expect(focusedIn(lotsGrid(language))).toBe('1:__remove');
    // ↓ on the last lot's trash cell: the "+" under the lots; ↑ goes back; ↓ again.
    await user.keyboard('{ArrowDown}');
    expect(lotButton()).toHaveFocus();
    await user.keyboard('{ArrowUp}');
    expect(focusedIn(lotsGrid(language))).toBe('1:__remove');
    await user.keyboard('{ArrowDown}');
    expect(lotButton()).toHaveFocus();
    // Enter adds a lot and the cursor lands on its seller.
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rowsOf(lotsGrid(language))).toBe(3);
    }, SLOW_MS);
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('2:seller');
    }, SLOW_MS);
    // And Enter on a trash cell removes that lot, as the button does.
    await user.click(cellOf(lotsGrid(language), 2, 'qty'));
    await reachTrash(user, () => lotsGrid(language));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rowsOf(lotsGrid(language))).toBe(2);
    }, SLOW_MS);
  });

  it('keyboard only, sale lines: the arrows reach the trash cell, Enter removes the line, ↓ reaches the "+", Enter adds a line and the cursor lands on its buyer', async () => {
    seedRows([3, 1]);
    const user = await open();
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(3);
    });
    await user.click(cellOf(salesGrid(language), 1, 'qty'));
    await reachTrash(user, () => salesGrid(language));
    expect(focusedIn(salesGrid(language))).toBe('1:__remove');
    // Enter removes the middle line, as the button does.
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(2);
    }, SLOW_MS);
    // The last line's trash cell: ↓ goes to the "+", Enter adds.
    await user.click(cellOf(salesGrid(language), 1, 'qty'));
    await reachTrash(user, () => salesGrid(language));
    await user.keyboard('{ArrowDown}');
    expect(lineButton()).toHaveFocus();
    await user.keyboard('{ArrowUp}');
    expect(focusedIn(salesGrid(language))).toBe('1:__remove');
    await user.keyboard('{ArrowDown}');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(3);
    }, SLOW_MS);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('2:buyer');
    }, SLOW_MS);
  });

  it('the Enter path of a sale line never stops on the trash cell: from the quantity it is at most 4 stops to the next line', async () => {
    seedRows([2, 1]);
    const user = await open();
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(2);
    });
    await user.click(cellOf(salesGrid(language), 0, 'qty'));
    const seen: (string | null)[] = [];
    for (let enters = 0; enters < 4 && focusedIn(salesGrid(language)) !== '1:buyer'; enters += 1) {
      await user.keyboard('{Enter}');
      seen.push(focusedIn(salesGrid(language)));
    }
    expect(seen.some((place) => (place ?? '').endsWith(':__remove'))).toBe(false);
    expect(focusedIn(salesGrid(language))).toBe('1:buyer');
  });

  it('in the whole-sale view the arrows reach the trash cell too, and the "+" adds to the lot of the focused line', async () => {
    seedRows([2, 3]);
    const user = await open();
    await user.click(await wholeSwitch());
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(5);
    }, SLOW_MS);
    await user.click(cellOf(salesGrid(language), 4, 'qty'));
    await reachTrash(user, () => salesGrid(language));
    expect(focusedIn(salesGrid(language))).toBe('4:__remove');
    await user.keyboard('{ArrowDown}');
    expect(lineButton()).toHaveFocus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rowsOf(salesGrid(language))).toBe(6);
    }, SLOW_MS);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('5:buyer');
    }, SLOW_MS);
  });

  it(`offers a lot at ${String(MAX_LOTS - 1)} lots and stops at ${String(MAX_LOTS)}, with the words`, async () => {
    seedRows(Array.from({ length: MAX_LOTS - 1 }, () => 1));
    const user = await open();
    expect(lotButton()).toBeEnabled();
    expect(
      screen.queryByText((text) => visible(text) === m('bikri-lots-limit', { max: '50' })),
    ).toBeNull();
    await user.click(lotButton());
    await waitFor(() => {
      expect(lotButton()).toBeDisabled();
    }, SLOW_MS);
    const words = m('bikri-lots-limit', { max: '50' });
    expect(
      screen.getByText(
        (_, element) => element?.tagName === 'P' && visible(element.textContent) === words,
      ),
    ).toBeInTheDocument();
    // The old "Add a lot" button above the grid is gone (Umar): the "+" is the one way to add with the mouse.
    expect(screen.getAllByRole('button', { name: named(m('bikri-lots-add-row')) })).toHaveLength(1);
  });

  it(`offers a line at ${String(MAX_SALE_LINES - 1)} lines and stops at ${String(MAX_SALE_LINES)}, with the words`, async () => {
    // 50 lots of 20 rows, the last one short by one: 999 rows in all. Only the lot shown is drawn.
    const rows = Array.from({ length: 50 }, () => 20);
    rows[49] = 19;
    seedRows(rows);
    const user = await open();
    await waitFor(() => {
      expect(allRowsOf(salesGrid(language))).toBe(20);
    }, SLOW_MS);
    expect(lineButton()).toBeEnabled();
    expect(
      screen.queryByText((text) => visible(text) === m('bikri-sales-limit', { max: '1,000' })),
    ).toBeNull();
    await user.click(lineButton());
    await waitFor(() => {
      expect(lineButton()).toBeDisabled();
    }, SLOW_MS);
    const words = m('bikri-sales-limit', { max: '1,000' });
    expect(
      screen.getByText(
        (_, element) => element?.tagName === 'P' && visible(element.textContent) === words,
      ),
    ).toBeInTheDocument();
    // The line that was added is there.
    expect(allRowsOf(salesGrid(language))).toBe(21);
  });
});
