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
import { createEntryStore, type EntryStore } from '../../src/shared/grid';
import { useLanguageStore } from '../../src/shared/i18n';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import type { SearchHit } from '../../src/shared/search';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/search/api');

const hit = (
  id: number,
  scope: 'item' | 'packing',
  code: string,
  ur: string,
  en: string,
): SearchHit => ({
  id,
  kind: scope,
  code,
  name_ur: ur,
  name_en: en,
  extra_ur: '',
  extra_en: '',
  is_active: true,
});

const ITEMS = [
  hit(61, 'item', '0044', 'مالٹا', 'Orange'),
  hit(62, 'item', '0045', 'کیلا', 'Banana'),
  hit(63, 'item', '0046', 'سیب', 'Apple'),
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
}: {
  store: EntryStore;
  kind: 'purchase' | 'arrival';
  errors?: readonly FieldError[];
}) {
  useEscapeDispatcher();
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
      {...(cellErrors === undefined ? {} : { cellErrors })}
    />
  );
}

function render(
  language: Language,
  options: { kind?: 'purchase' | 'arrival'; errors?: readonly FieldError[] } = {},
) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  const store = createEntryStore();
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Harness
        store={store}
        kind={options.kind ?? 'purchase'}
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

  it('a purchase of three lines is typed with the keys only: Enter stops on every typed cell and skips the two calculated ones', async () => {
    const { store, user } = render(language);
    act(() => {
      store.getState().setPendingFocus({ rowIndex: 0, field: LINE.item });
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
    // Line 1: oranges, 80 boxes, 1,200, freight 400, 30 of them for another shop.
    await pick(user, 'Ora');
    await pick(user, 'Box');
    await user.keyboard('80{Enter}');
    await user.keyboard('{Enter}'); // weight: not an item sold by weight
    await user.keyboard('1200{Enter}400{Enter}');
    await user.keyboard('{Enter}'); // labour
    await user.keyboard('{Enter}'); // other cost
    await user.keyboard('30{Enter}Haji Shop{Enter}');
    // Enter went on to the item of the next line: the two calculated cells were skipped.
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-index="1"] .ag-cell-focus')).not.toBeNull();
    });
    expect(store.getState().rows).toHaveLength(2);
    // Line 2: bananas, 40 boxes at 900, all for our shop.
    await pick(user, 'Ban');
    await pick(user, 'Box');
    await user.keyboard('40{Enter}{Enter}900{Enter}');
    for (let stop = 0; stop < 5; stop += 1) await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(store.getState().rows).toHaveLength(3);
    });
    // Line 3: apples, 10 crates at 2,000, 4 for another shop.
    await pick(user, 'App');
    await pick(user, 'Crate');
    await user.keyboard('10{Enter}{Enter}2000{Enter}{Enter}{Enter}{Enter}4{Enter}');

    const rows = store.getState().rows;
    expect(rows.map((row) => pickedOf(store, rows.indexOf(row), LINE.item))).toEqual([61, 62, 63]);
    expect(rows.map((row) => pickedOf(store, rows.indexOf(row), LINE.packing))).toEqual([
      71, 71, 72,
    ]);
    expect([0, 1, 2].map((index) => typedOf(store, index, LINE.qty))).toEqual(['80', '40', '10']);
    expect([0, 1, 2].map((index) => typedOf(store, index, LINE.rate))).toEqual([
      '1200',
      '900',
      '2000',
    ]);
    expect(typedOf(store, 0, LINE.freight)).toBe('400');
    expect([0, 1, 2].map((index) => typedOf(store, index, LINE.otherQty))).toEqual([
      '30',
      undefined,
      '4',
    ]);
    expect(typedOf(store, 0, LINE.shop)).toBe('Haji Shop');
    // Nothing was worked out here: the quantity left for our shop is not in the rows.
    expect(typedOf(store, 0, LINE.ourQty)).toBeUndefined();
  });

  it('goods arrived have no rate or costs: Enter goes from the weight to the split', async () => {
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
    await user.keyboard('{Enter}'); // weight
    await user.keyboard('40{Enter}');
    expect(typedOf(store, 0, LINE.qty)).toBe('100');
    expect(typedOf(store, 0, LINE.otherQty)).toBe('40');
    expect(document.querySelector('.ag-cell[col-id="rate"]')).toBeNull();
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
    await user.keyboard('80{Enter}{Enter}1200{Enter}');
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
