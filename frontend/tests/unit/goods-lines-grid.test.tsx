// The grid of item lines of an entry (plan 35, PRD FR-12.41, Code Standards F30, F39): a trip
// purchase entry and a garden's goods arrived entry. The keys are the shared grid's (PRD 7.2): the
// test types a purchase of three lines with the keyboard only, Enter stopping on every cell that can
// be typed in and skipping the two the backend works out; a blank line is not a line; what the
// backend refused is on the cell; and the lines live in the store, so a new grid (a language switch)
// shows the same lines. Both languages, against a fake search of items and packings.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useMemo } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { FieldError } from '../../src/bindings/FieldError';
import {
  GoodsLinesGrid,
  LINE,
  filledLines,
  goodsLineColumns,
  lineCellErrors,
  lineIsBlank,
  linesAreFilled,
} from '../../src/features/purchases';
import { LANGUAGES, type Language } from '../../src/i18n';
import { createEntryStore, isEnterStop, type EntryStore } from '../../src/shared/grid';
import { useLatest } from '../../src/shared/hooks';
import { useLanguageStore } from '../../src/shared/i18n';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import type { SearchHit } from '../../src/shared/search';
import { message, renderIn, visible } from '../helpers/render';
import realItems from '../fixtures/search_masters_items_sold_by_weight.json';

vi.mock('../../src/shared/search/api');

const hit = (
  id: number,
  scope: 'item' | 'packing',
  code: string,
  ur: string,
  en: string,
  soldByWeight = false,
): SearchHit => ({
  id,
  kind: scope,
  code,
  name_ur: ur,
  name_en: en,
  extra_ur: '',
  extra_en: '',
  is_active: true,
  // True only for an item sold by weight: the search hit says it, so the row knows at once.
  sold_by_weight: soldByWeight,
});

const ITEMS = [
  hit(61, 'item', '0044', 'مالٹا', 'Orange'),
  hit(62, 'item', '0045', 'کیلا', 'Banana'),
  hit(63, 'item', '0046', 'سیب', 'Apple'),
  hit(64, 'item', '0047', 'انار', 'Pomegranate', true),
];
const PACKINGS = [
  hit(71, 'packing', '0050', 'پیٹی', 'Box'),
  hit(72, 'packing', '0051', 'کریٹ', 'Crate'),
];

/** The search as the backend answers it: the records of the kind whose code or name has the text. */
function fakeSearch() {
  const of = (scope: string) => (scope === 'item' ? ITEMS : scope === 'packing' ? PACKINGS : []);
  vi.mocked(searchApi.searchMasters).mockImplementation(({ scope, text }) => {
    const wanted = text.trim().toLowerCase();
    return Promise.resolve(
      of(scope).filter(
        (row) =>
          wanted === '' ||
          [row.code, row.name_ur, row.name_en].some((part) => part.toLowerCase().includes(wanted)),
      ),
    );
  });
  vi.mocked(searchApi.searchRecent).mockImplementation(({ scope }) => Promise.resolve(of(scope)));
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}

function Harness({
  store,
  kind,
  errors,
  weightRowIds,
}: {
  store: EntryStore;
  kind: 'purchase' | 'arrival';
  errors?: readonly FieldError[];
  /** The rows whose item is sold by weight, as the backend would tell. */
  weightRowIds?: ReadonlySet<string>;
}) {
  useEscapeDispatcher();
  const weightRows = useLatest(weightRowIds ?? new Set<string>());
  const rows = store((state) => state.rows);
  const cellErrors = useMemo(
    () =>
      errors === undefined
        ? undefined
        : lineCellErrors(errors, filledLines(rows), (item) => item.code),
    [errors, rows],
  );
  return (
    <GoodsLinesGrid
      kind={kind}
      store={store}
      weightRows={weightRows}
      {...(cellErrors === undefined ? {} : { cellErrors })}
    />
  );
}

function render(
  language: Language,
  options: {
    kind?: 'purchase' | 'arrival';
    errors?: readonly FieldError[];
    /** Row numbers (0 is the first) of the rows whose item is sold by weight. */
    weightRows?: readonly number[];
  } = {},
) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  const store = createEntryStore();
  const weightRowIds = new Set(
    (options.weightRows ?? []).flatMap((index) => {
      const row = store.getState().rows[index];
      return row === undefined ? [] : [row.rowId];
    }),
  );
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Harness
        store={store}
        kind={options.kind ?? 'purchase'}
        weightRowIds={weightRowIds}
        {...(options.errors === undefined ? {} : { errors: options.errors })}
      />
    </QueryClientProvider>,
  );
  const user = userEvent.setup();
  const cell = (row: number, field: string) => {
    const found = document.querySelector<HTMLElement>(
      `.ag-row[row-index="${String(row)}"] .ag-cell[col-id="${field}"]`,
    );
    if (found === null) throw new Error(`no cell ${String(row)}:${field}`);
    return found;
  };
  return { store, user, cell };
}

beforeEach(() => {
  vi.clearAllMocks();
  fakeSearch();
});

afterEach(() => {
  document.body.innerHTML = '';
});

const typedOf = (store: EntryStore, row: number, field: string) =>
  store.getState().rows[row]?.typed[field];
const pickedOf = (store: EntryStore, row: number, field: string) =>
  store.getState().rows[row]?.picked?.[field]?.id;

describe('the columns of the lines', () => {
  it('a purchase has the costs and the value; goods arrived have neither; both have the split', () => {
    expect(goodsLineColumns('purchase', 'normal').map((column) => column.field)).toEqual([
      'item',
      'packing',
      'qty',
      'weight',
      'rate',
      'freight',
      'labour',
      'otherCost',
      'otherQty',
      'shop',
      'ourQty',
      'value',
    ]);
    expect(goodsLineColumns('arrival', 'normal').map((column) => column.field)).toEqual([
      'item',
      'packing',
      'qty',
      'weight',
      'otherQty',
      'shop',
      'ourQty',
    ]);
  });

  it('at the narrowest window the value of the line is left out, and every typed cell stays', () => {
    const compact = goodsLineColumns('purchase', 'compact').map((column) => column.field);
    expect(compact).not.toContain('value');
    expect(compact).toEqual(
      goodsLineColumns('purchase', 'normal')
        .map((c) => c.field)
        .filter((f) => f !== 'value'),
    );
  });

  it('the quantity left for our shop and the value are the backend’s: they are never typed in', () => {
    const calc = goodsLineColumns('purchase', 'normal').filter((column) => column.type === 'calc');
    expect(calc.map((column) => column.field)).toEqual(['ourQty', 'value']);
  });

  it('Enter stops only at the item, the packing, the quantity and, for a purchase, the rate; the optional cells are editable but not stops (plan 35, D3)', () => {
    const row = { rowId: 'r', typed: {} };
    const stops = (kind: 'purchase' | 'arrival', current = row) =>
      goodsLineColumns(kind, 'normal')
        .filter((column) => isEnterStop(column, current))
        .map((column) => column.field);
    expect(stops('purchase')).toEqual(['item', 'packing', 'qty', 'rate']);
    expect(stops('arrival')).toEqual(['item', 'packing', 'qty']);
    // The optional cells are still typed cells: only the calculated two are not.
    const optional = goodsLineColumns('purchase', 'normal').filter(
      (column) => !isEnterStop(column, row) && column.type !== 'calc',
    );
    expect(optional.map((column) => column.field)).toEqual([
      'weight',
      'freight',
      'labour',
      'otherCost',
      'otherQty',
      'shop',
    ]);
  });

  it('the weight is a stop on a row whose item is sold by weight, and on a row that has a weight typed', () => {
    const weight = (typed: Record<string, string>, weightRows: string[]) =>
      goodsLineColumns('purchase', 'normal', () => new Set(weightRows))
        .filter((column) => isEnterStop(column, { rowId: 'r', typed }))
        .map((column) => column.field);
    expect(weight({}, ['r'])).toEqual(['item', 'packing', 'qty', 'weight', 'rate']);
    expect(weight({ weight: '12.5' }, [])).toEqual(['item', 'packing', 'qty', 'weight', 'rate']);
    expect(weight({}, ['other'])).toEqual(['item', 'packing', 'qty', 'rate']);
  });

  it('a line with nothing typed or chosen is blank, and only the lines that hold something are sent', () => {
    const store = createEntryStore([{}, { qty: '5' }, { shop: '  ' }]);
    const rows = store.getState().rows;
    expect(rows.map((row) => lineIsBlank(row))).toEqual([true, false, true]);
    expect(filledLines(rows)).toHaveLength(1);
    expect(linesAreFilled(rows)).toBe(true);
    expect(linesAreFilled(createEntryStore().getState().rows)).toBe(false);
  });
});

describe('what the backend refused on a line', () => {
  it('goes to the cell of the line it names, counting the lines that were sent', () => {
    // The blank line in the middle is not sent, so `lines[1]` is the third row.
    const store = createEntryStore([{ qty: '1' }, {}, { qty: '', rate: '5' }]);
    const rows = store.getState().rows;
    const found = lineCellErrors(
      [
        { code: 'ERR_REQUIRED', field: 'lines[1].qty', values: {} },
        { code: 'ERR_NOT_A_NUMBER', field: 'lines[0].other_qty', values: {} },
        // No cell for this one, and none for a problem of the whole entry.
        { code: 'ERR_REQUIRED', field: 'lines[0].supplier', values: {} },
        { code: 'ERR_REQUIRED', field: 'date', values: {} },
      ],
      filledLines(rows),
      (item) => item.code,
    );
    expect(found).toEqual({
      [rows[2]?.rowId ?? '']: { qty: 'ERR_REQUIRED' },
      [rows[0]?.rowId ?? '']: { otherQty: 'ERR_NOT_A_NUMBER' },
    });
  });
});

describe.each(LANGUAGES)('the lines grid with the keyboard (%s)', (language) => {
  /** Types a picker cell and takes the first match: waits for the list, so Enter finds it. */
  async function pick(user: ReturnType<typeof userEvent.setup>, text: string) {
    await user.keyboard(text);
    await screen.findAllByRole('option');
    await user.keyboard('{Enter}');
  }

  const forward = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';

  it('a purchase of three lines is typed with the keys only: four Enters for each line (item, packing, quantity, rate), and Enter on the rate goes to the item of the next line', async () => {
    const { store, user } = render(language);
    act(() => {
      store.getState().setPendingFocus({ rowIndex: 0, field: LINE.item });
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
    const focused = () => {
      const cell = document.querySelector('.ag-cell-focus');
      return `${cell?.closest('.ag-row')?.getAttribute('row-index') ?? ''}:${cell?.getAttribute('col-id') ?? ''}`;
    };
    // Line 1: oranges, 80 boxes at 1,200. Enter in the picker takes the match and goes on.
    await pick(user, 'Ora');
    await pick(user, 'Box');
    await user.keyboard('80{Enter}');
    // The cursor is on the rate: the weight, which this item has none of, was passed over.
    await waitFor(() => {
      expect(focused()).toBe('0:rate');
    });
    await user.keyboard('1200{Enter}');
    await waitFor(() => {
      expect(focused()).toBe('1:item');
    });
    expect(store.getState().rows).toHaveLength(2);
    // Line 2: bananas, 40 boxes at 900.
    await pick(user, 'Ban');
    await pick(user, 'Box');
    await user.keyboard('40{Enter}900{Enter}');
    await waitFor(() => {
      expect(focused()).toBe('2:item');
    });
    // Line 3: apples, 10 crates at 2,000.
    await pick(user, 'App');
    await pick(user, 'Crate');
    await user.keyboard('10{Enter}2000{Enter}');
    await waitFor(() => {
      expect(focused()).toBe('3:item');
    });

    const rows = store.getState().rows;
    expect(rows.map((_, index) => pickedOf(store, index, LINE.item))).toEqual([
      61,
      62,
      63,
      undefined,
    ]);
    expect([0, 1, 2].map((index) => pickedOf(store, index, LINE.packing))).toEqual([71, 71, 72]);
    expect([0, 1, 2].map((index) => typedOf(store, index, LINE.qty))).toEqual(['80', '40', '10']);
    expect([0, 1, 2].map((index) => typedOf(store, index, LINE.rate))).toEqual([
      '1200',
      '900',
      '2000',
    ]);
    // Nothing else was typed, and nothing was worked out here: the calculated cells are not in the rows.
    expect(typedOf(store, 0, LINE.freight)).toBeUndefined();
    expect(typedOf(store, 0, LINE.ourQty)).toBeUndefined();
  });

  it('goods arrived: three stops, item, packing and quantity, then the item of the next line', async () => {
    const { store, user } = render(language, { kind: 'arrival' });
    act(() => {
      store.getState().setPendingFocus({ rowIndex: 0, field: LINE.item });
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
    await pick(user, 'Ban');
    await pick(user, 'Box');
    await user.keyboard('100{Enter}');
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-index="1"] .ag-cell-focus')).not.toBeNull();
    });
    expect(typedOf(store, 0, LINE.qty)).toBe('100');
    expect(document.querySelector('.ag-cell[col-id="rate"]')).toBeNull();
    expect(store.getState().rows).toHaveLength(2);
  });

  it('an item sold by weight: Enter also stops at its weight, between the quantity and the rate', async () => {
    const { store, user } = render(language, { weightRows: [0] });
    act(() => {
      store.getState().setPendingFocus({ rowIndex: 0, field: LINE.item });
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
    await pick(user, 'Ora');
    await pick(user, 'Box');
    await user.keyboard('80{Enter}');
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')?.getAttribute('col-id')).toBe('weight');
    });
    await user.keyboard('1234.5{Enter}');
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')?.getAttribute('col-id')).toBe('rate');
    });
    await user.keyboard('100{Enter}');
    expect(typedOf(store, 0, LINE.weight)).toBe('1234.5');
    expect(typedOf(store, 0, LINE.rate)).toBe('100');
  });

  it('D3: an item sold by weight is known the moment it is chosen: Enter on the quantity stops at the weight at once, with no preview and no answer of the backend', async () => {
    // This grid has no preview at all and is given no weight rows: only the chosen item can say it.
    const { store, user } = render(language);
    act(() => {
      store.getState().setPendingFocus({ rowIndex: 0, field: LINE.item });
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
    await pick(user, 'Pom');
    await pick(user, 'Box');
    await user.keyboard('80{Enter}');
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')?.getAttribute('col-id')).toBe('weight');
    });
    // The chosen record keeps what its hit said, and a plain item does not carry it.
    expect(store.getState().rows[0]?.picked?.[LINE.item]?.soldByWeight).toBe(true);
    await user.keyboard('1234{Enter}');
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')?.getAttribute('col-id')).toBe('rate');
    });
    // The next line, a plain item: Enter on the quantity goes straight to the rate.
    await user.keyboard('100{Enter}');
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-index="1"] .ag-cell-focus')).not.toBeNull();
    });
    await pick(user, 'Ora');
    await pick(user, 'Box');
    await user.keyboard('40{Enter}');
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')?.getAttribute('col-id')).toBe('rate');
    });
    expect(store.getState().rows[1]?.picked?.[LINE.item]?.soldByWeight).toBeUndefined();
  });

  it('D3: the backend’s own search answer for the items says which are sold by weight, and the row takes it from the chosen hit', async () => {
    const real = realItems as SearchHit[];
    const byWeight = real.find((row) => row.sold_by_weight);
    if (byWeight === undefined) throw new Error('the file has no item sold by weight');
    vi.mocked(searchApi.searchMasters).mockImplementation(({ scope, text }) =>
      Promise.resolve(
        scope === 'item'
          ? real.filter((row) => row.name_en.toLowerCase().includes(text.trim().toLowerCase()))
          : PACKINGS,
      ),
    );
    const { store, user } = render(language);
    act(() => {
      store.getState().setPendingFocus({ rowIndex: 0, field: LINE.item });
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
    await pick(user, byWeight.name_en);
    await pick(user, 'Box');
    await user.keyboard('80{Enter}');
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')?.getAttribute('col-id')).toBe('weight');
    });
    expect(store.getState().rows[0]?.picked?.[LINE.item]?.id).toBe(byWeight.id);
  });

  it('the arrow keys reach the optional cells: freight, labour, other cost, the part for another shop and the shop name', async () => {
    const { store, user, cell } = render(language);
    act(() => {
      store.getState().setPendingFocus({ rowIndex: 0, field: LINE.item });
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
    await pick(user, 'Ora');
    await pick(user, 'Box');
    await user.keyboard('80{Enter}1200{Enter}');
    // Back to the rate of line 1 with a click, then the arrow that goes on in the reading direction.
    await user.click(cell(0, LINE.rate));
    await user.keyboard(forward);
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')?.getAttribute('col-id')).toBe('freight');
    });
    // Typing starts the cell; an arrow down and up finishes it and comes back to the same cell.
    await user.keyboard('400{ArrowDown}{ArrowUp}');
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-index="0"] .ag-cell-focus')).not.toBeNull();
    });
    await user.keyboard(forward);
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')?.getAttribute('col-id')).toBe('labour');
    });
    await user.keyboard(`${forward}${forward}${forward}`);
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')?.getAttribute('col-id')).toBe('shop');
    });
    await user.keyboard('Haji Shop');
    await user.keyboard('{Enter}');
    expect(typedOf(store, 0, LINE.freight)).toBe('400');
    expect(typedOf(store, 0, LINE.shop)).toBe('Haji Shop');
    // The values typed in the optional cells are in the line as before.
    expect(store.getState().rows[0]?.typed[LINE.rate]).toBe('1200');
  });

  it('"Add a line" adds a line and puts the cursor on its item; Ctrl+Delete removes the line', async () => {
    const { store, user } = render(language);
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name) === message(language, 'goods-lines-add'),
      }),
    );
    expect(store.getState().rows).toHaveLength(2);
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-index="1"] .ag-cell-focus')).not.toBeNull();
    });
    await user.keyboard('{Control>}{Delete}{/Control}');
    await waitFor(() => {
      expect(store.getState().rows).toHaveLength(1);
    });
  });

  it('what the backend refused is on the cell it names, and only on that cell', async () => {
    const { store, cell } = render(language, {
      errors: [{ code: 'ERR_QTY_EXCEEDS_LOT', field: 'lines[0].other_qty', values: {} }],
    });
    act(() => {
      store.getState().patchRow(store.getState().rows[0]?.rowId ?? '', {
        typed: { qty: '80', otherQty: '90' },
      });
    });
    await waitFor(() => {
      expect(cell(0, LINE.otherQty).className).toContain('cell-error');
    });
    // The other cells of the line are not marked.
    expect(cell(0, LINE.qty).className).not.toContain('cell-error');
  });

  it('a language switch makes a new grid and the lines are the same (they live in the store)', async () => {
    const { store, user } = render(language);
    act(() => {
      store.getState().setPendingFocus({ rowIndex: 0, field: LINE.item });
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
    await pick(user, 'Ora');
    await pick(user, 'Box');
    await user.keyboard('80{Enter}1200{Enter}');
    const before = JSON.stringify(store.getState().rows);
    // The other language: the headers and the names change, the lines do not.
    const other: Language = language === 'ur' ? 'en' : 'ur';
    document.body.innerHTML = '';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    renderIn(
      other,
      <QueryClientProvider client={new QueryClient()}>
        <Harness store={store} kind="purchase" />
      </QueryClientProvider>,
    );
    await waitFor(() => {
      expect(document.querySelector('.ag-header-cell[col-id="item"]')?.textContent).toContain(
        message(other, 'glossary-qism-maal'),
      );
    });
    expect(JSON.stringify(store.getState().rows)).toBe(before);
    const shown = (field: string) =>
      visible(
        document.querySelector(`.ag-row[row-index="0"] .ag-cell[col-id="${field}"]`)?.textContent,
      );
    expect(shown('item')).toBe(other === 'ur' ? 'مالٹا' : 'Orange');
    expect(shown('qty')).toBe('80');
    expect(shown('rate')).toBe('1,200');
  });
});
