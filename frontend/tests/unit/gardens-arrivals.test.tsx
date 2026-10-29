// Goods arrived on the garden screen (plan 33 version 1.1 and plan 35, PRD FR-12.40, FR-12.41,
// FR-7.67), in the Urdu and the English app. The type list has a fourth type, مال آیا; its form has
// the day to sell and a grid of item lines (item, packing, quantity, weight, and the split between
// our shop and another shop), and the screen says that it posts no money. The list of the garden's
// entries shows what is left in each part and the day to sell; the card has the actions: change the
// day to sell, change the split, write off, undo a write-off, cancel. The answers of the lists and
// of the other commands are the backend's real files (arrivals_fixtures.rs).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { act } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { CommandError } from '../../src/shared/api';
import * as urduApi from '../../src/shared/urdu/api';
import * as arrivals from '../helpers/arrivals-real';
import { named } from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardens } from '../helpers/gardens';
import { lineCell, lineCellText, linesSection, typeArrivalLine } from '../helpers/goods-lines';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { fakeItemPickers } from '../helpers/trips';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeItemPickers();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
type User = ReturnType<typeof userEvent.setup>;

/** The check and the save of the goods arrived entry as a healthy backend answers them. */
function fakeArrivalEntry() {
  vi.mocked(gardensApi.gardensArrivalEntryPreview).mockResolvedValue({ errors: [], warnings: [] });
  vi.mocked(gardensApi.gardensArrivalEntrySave).mockImplementation((input) =>
    Promise.resolve({
      rows: input.lines.map((_, index) => ({
        ...arrivals.ARRIVAL_SAVE.row,
        id: index + 1,
        number: index + 1,
      })),
      warnings: [],
    }),
  );
}

describe.each(LANGUAGES)('goods arrived on the garden screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const open = async () => {
    const view = await renderApp('/gardens/1');
    await screen.findByText(said(message(language, 'gardens-detail-title', { code: '0080' })));
    return { ...view, user: userEvent.setup() };
  };

  const typeList = () =>
    screen.getByRole('group', { name: named(message(language, 'gardens-type-label')) });
  const chooseArrival = async (user: User) => {
    await user.click(
      within(typeList()).getByRole('button', {
        name: named(message(language, 'gardens-type-arrival')),
      }),
    );
  };
  const field = (id: string) => {
    const label = named(message(language, id));
    return (
      screen.queryByRole('textbox', { name: label }) ??
      screen.getByRole('combobox', { name: label })
    );
  };
  const dayText = (box: HTMLElement) => (box as HTMLInputElement).value;

  it('the type list has the goods arrived; its form has the day to sell and the grid of lines, and the screen says it posts no money', async () => {
    fakeGardens();
    const { user } = await open();
    expect(
      within(typeList())
        .getAllByRole('button')
        .map((button) => visible(button.textContent)),
    ).toContain(message(language, 'gardens-type-arrival'));
    await chooseArrival(user);
    // The grid: the goods, the quantity, the weight and the split with what is left for our shop.
    const section = linesSection('arrival');
    expect(section).not.toBeNull();
    const headers = Array.from(section?.querySelectorAll('.ag-header-cell') ?? []).map((cell) =>
      cell.getAttribute('col-id'),
    );
    expect(headers).toEqual(['item', 'packing', 'qty', 'weight', 'otherQty', 'shop', 'ourQty']);
    // The day to sell starts as the day of the arrival, visible like a typed value (NFR-USE-27).
    const sell = field('glossary-sell-date');
    const day = field('gardens-field-date');
    expect(dayText(sell)).toBe(dayText(day));
    expect(
      screen.getByText(said(message(language, 'gardens-arrival-posts-nothing'))),
    ).toBeInTheDocument();
    // No sources of money: nothing is paid.
    expect(screen.queryByText(said(message(language, 'gardens-sources-title')))).toBeNull();
    // It follows the day of the arrival while nobody has typed in it.
    await user.click(day);
    await user.keyboard('{Control>}a{/Control}09-10-2026');
    await waitFor(() => {
      expect(dayText(sell)).toBe('09-10-2026');
    });
  });

  // The fake pickers know one item and one box.
  const oranges = {
    item: language === 'ur' ? 'م' : 'Ora',
    packing: language === 'ur' ? 'پ' : 'Box',
  };

  it('F10 saves the goods arrived as typed, with the day to sell the user typed, and starts the next one', async () => {
    fakeGardens();
    fakeArrivalEntry();
    const { user } = await open();
    await chooseArrival(user);
    await typeArrivalLine(user, 0, { ...oranges, qty: '100' });
    const sell = field('glossary-sell-date');
    await user.click(sell);
    await user.keyboard('{Control>}a{/Control}12-10-2026');
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensArrivalEntrySave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(gardensApi.gardensArrivalEntrySave).mock.calls[0]?.[0]).toEqual({
      garden_id: 1,
      date: '10-10-2026',
      sell_date: '12-10-2026',
      lines: [
        {
          key: expect.any(String) as string,
          item_id: expect.any(Number) as number,
          packing_id: expect.any(Number) as number,
          qty: '100',
          weight: null,
          note: null,
          // Nothing for another shop: all of it is for our shop (PRD FR-12.41).
          other_qty: null,
          shop_name: null,
        },
      ],
    });
    expect(
      await screen.findByText(
        said(
          message(language, 'gardens-saved-arrival', {
            number: message(language, 'gardens-arrival-number', { number: 1 }),
          }),
        ),
      ),
    ).toBeInTheDocument();
    // The next entry starts empty and its day to sell is the day again.
    await waitFor(() => {
      expect(lineCellText(0, 'qty', 'arrival')).toBe('');
    });
  });

  it('R2, R3: one entry of two items saves both lines in one call, with the split of each line as typed', async () => {
    fakeGardens();
    fakeArrivalEntry();
    const { user } = await open();
    await chooseArrival(user);
    // Line 1: 100 boxes of bananas, 40 of them for another shop, "Haji Shop".
    await typeArrivalLine(user, 0, { ...oranges, qty: '100', otherQty: '40', shop: 'Haji Shop' });
    // Enter on the last cell went on to a new line: line 2, all for our shop.
    await waitFor(() => {
      expect(lineCell(1, 'item', 'arrival')).toBeInTheDocument();
    });
    await typeArrivalLine(user, 1, { ...oranges, qty: '50' });
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensArrivalEntrySave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(gardensApi.gardensArrivalEntrySave).mock.calls[0]?.[0];
    // The blank row at the end is not sent: two lines, in the order of the grid.
    expect(sent?.lines).toHaveLength(2);
    expect(sent?.lines[0]).toMatchObject({ qty: '100', other_qty: '40', shop_name: 'Haji Shop' });
    expect(sent?.lines[1]).toMatchObject({ qty: '50', other_qty: null, shop_name: null });
    expect(
      await screen.findByText(
        said(
          message(language, 'gardens-saved-arrivals', {
            count: 2,
            first: message(language, 'gardens-arrival-number', { number: 1 }),
            last: message(language, 'gardens-arrival-number', { number: 2 }),
          }),
        ),
      ),
    ).toBeInTheDocument();
  });

  it('a mistake of one line is on the cell of that line, in plain words, and nothing typed is lost', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalEntryPreview).mockResolvedValue({
      errors: [],
      warnings: [],
    });
    vi.mocked(gardensApi.gardensArrivalEntrySave).mockRejectedValue(
      fail({ code: 'ERR_SPLIT_ABOVE_QTY', field: 'lines[1].other_qty', values: { qty: 50 } }),
    );
    const { user } = await open();
    await chooseArrival(user);
    await typeArrivalLine(user, 0, { ...oranges, qty: '100' });
    await waitFor(() => {
      expect(lineCell(1, 'item', 'arrival')).toBeInTheDocument();
    });
    await typeArrivalLine(user, 1, { ...oranges, qty: '50', otherQty: '90' });
    press('F10');
    // The words are in the list under the form, and the cell of line 2 is marked.
    expect(
      await screen.findByText(said(message(language, 'err-split-above-qty', { qty: 50 }))),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(lineCell(1, 'otherQty', 'arrival').className).toContain('cell-error');
    });
    expect(lineCell(0, 'otherQty', 'arrival').className).not.toContain('cell-error');
    expect(lineCellText(0, 'qty', 'arrival')).toBe('100');
    expect(lineCellText(1, 'otherQty', 'arrival')).toBe('90');
  });

  it('R2: the real check of two lines gives the quantity left for our shop of each line, found by the key of the row', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalEntryPreview).mockImplementation((sent) =>
      Promise.resolve({
        ...arrivals.ARRIVAL_ENTRY_PREVIEW_TWO_LINES,
        // The backend echoes the keys it was sent; its file has its own, so it gets the screen's.
        lines: arrivals.ARRIVAL_ENTRY_PREVIEW_TWO_LINES.lines.map((line, index) => ({
          ...line,
          key: sent.lines[index]?.key ?? line.key,
        })),
      }),
    );
    const { user } = await open();
    await chooseArrival(user);
    // The lines of the backend's own file: 100 with 40 for another shop, and 30 all for our shop.
    await typeArrivalLine(user, 0, { ...oranges, qty: '100', otherQty: '40', shop: 'Haji Shop' });
    await typeArrivalLine(user, 1, { ...oranges, qty: '30' });
    await waitFor(() => {
      expect(lineCellText(0, 'ourQty', 'arrival')).toBe('60');
    });
    expect(lineCellText(1, 'ourQty', 'arrival')).toBe('30');
  });

  it('the real refusal of a line of the save is on the cell of that line', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalEntryPreview).mockResolvedValue({
      errors: [],
      warnings: [],
    });
    vi.mocked(gardensApi.gardensArrivalEntrySave).mockRejectedValue(
      new CommandError(arrivals.ARRIVAL_ENTRY_SAVE_LINE_ERROR),
    );
    const { user } = await open();
    await chooseArrival(user);
    await typeArrivalLine(user, 0, { ...oranges, qty: '100' });
    await typeArrivalLine(user, 1, { ...oranges, qty: '10', otherQty: '20' });
    press('F10');
    await waitFor(() => {
      expect(lineCell(1, 'otherQty', 'arrival').className).toContain('cell-error');
    });
    expect(lineCell(0, 'otherQty', 'arrival').className).not.toContain('cell-error');
    expect(lineCellText(0, 'qty', 'arrival')).toBe('100');
  });

  it('the screen builds the request the backend tested for two lines, key by key', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalEntryPreview).mockResolvedValue({
      errors: [],
      warnings: [],
    });
    const { user } = await open();
    await chooseArrival(user);
    await typeArrivalLine(user, 0, { ...oranges, qty: '100', otherQty: '40', shop: 'حاجی شاپ' });
    await typeArrivalLine(user, 1, { ...oranges, qty: '30' });
    await waitFor(() => {
      expect(gardensApi.gardensArrivalEntryPreview).toHaveBeenCalled();
    });
    const sent = vi.mocked(gardensApi.gardensArrivalEntryPreview).mock.calls.at(-1)?.[0];
    const request = arrivals.ARRIVAL_ENTRY_PREVIEW_TWO_LINES_REQUEST;
    // The same fields and the same texts as the backend's own file; only the ids of the pickers
    // (fake here) and the keys (the rows' own) differ.
    expect(Object.keys(sent ?? {}).sort()).toEqual(Object.keys(request).sort());
    expect(Object.keys(sent?.lines[0] ?? {}).sort()).toEqual(
      Object.keys(request.lines[0] ?? {}).sort(),
    );
    expect(sent?.lines.map((line) => [line.qty, line.other_qty, line.shop_name])).toEqual(
      request.lines.map((line) => [line.qty, line.other_qty, line.shop_name]),
    );
    expect(sent?.lines.every((line) => line.key !== '')).toBe(true);
  });

  it('an untouched day to sell is sent as nothing: the backend reads it as the day of the arrival', async () => {
    fakeGardens();
    fakeArrivalEntry();
    const { user } = await open();
    await chooseArrival(user);
    await typeArrivalLine(user, 0, { ...oranges, qty: '100' });
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensArrivalEntrySave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(gardensApi.gardensArrivalEntrySave).mock.calls[0]?.[0].sell_date).toBeNull();
  });

  it('a day to sell before the day of the arrival is refused at the field, and what was typed stays', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalEntryPreview).mockResolvedValue({
      errors: [],
      warnings: [],
    });
    vi.mocked(gardensApi.gardensArrivalEntrySave).mockRejectedValue(
      fail({
        code: 'ERR_SELL_DATE_BEFORE_ENTRY',
        field: 'sell_date',
        values: { entry_date: '2026-10-11' },
      }),
    );
    const { user } = await open();
    await chooseArrival(user);
    await typeArrivalLine(user, 0, { ...oranges, qty: '100' });
    await user.click(field('glossary-sell-date'));
    await user.keyboard('{Control>}a{/Control}01-01-2020');
    press('F10');
    expect(
      await screen.findByText(
        said(message(language, 'err-sell-date-before-entry', { entry_date: '11-10-2026' })),
      ),
    ).toBeInTheDocument();
    expect(lineCellText(0, 'qty', 'arrival')).toBe('100');
    expect(dayText(field('glossary-sell-date'))).toBe('01-01-2020');
  });

  it('a language switch in the middle of an entry keeps every line and the split', async () => {
    fakeGardens();
    fakeArrivalEntry();
    const { user } = await open();
    await chooseArrival(user);
    await typeArrivalLine(user, 0, { ...oranges, qty: '100', otherQty: '40', shop: 'Haji Shop' });
    const other = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    await waitFor(() => {
      expect(lineCellText(0, 'item', 'arrival')).toBe(other === 'ur' ? 'مالٹا' : 'Orange');
    });
    expect(lineCellText(0, 'qty', 'arrival')).toBe('100');
    expect(lineCellText(0, 'otherQty', 'arrival')).toBe('40');
    expect(lineCellText(0, 'shop', 'arrival')).toBe('Haji Shop');
  });

  it('lists the entries of the garden with what is left, and the card has the figures and the day to sell', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue(arrivals.ARRIVALS_IN_STOCK);
    await open();
    const row = await waitFor(() => {
      const found = document.querySelector('.ag-row[row-id="1"]');
      if (found === null) throw new Error('no row');
      return found;
    });
    expect(visible(row.querySelector('[col-id="number"]')?.textContent)).toBe(
      message(language, 'gardens-arrival-number', { number: 1 }),
    );
    expect(visible(row.querySelector('[col-id="qty_left"]')?.textContent)).toBe('100');
    const card = (
      await screen.findByText(
        said(
          message(language, 'gardens-arrival-card-title', {
            number: message(language, 'gardens-arrival-number', { number: 1 }),
            item: language === 'ur' ? 'کیلا' : 'Banana',
          }),
        ),
      )
    ).parentElement;
    const text = visible(card?.textContent);
    for (const figure of ['11-10-2026', '12-10-2026', '100']) expect(text).toContain(figure);
    // The garden's list was asked for this garden, the first page.
    expect(gardensApi.gardensArrivalsList).toHaveBeenCalledWith(
      expect.objectContaining({ garden_id: 1, offset: 0 }),
    );
  });

  it('FR-12.41: a sale in another shop picks the goods arrived it takes from; the option says what is left in each part', async () => {
    const fake = fakeGardens();
    const first = arrivals.ARRIVALS_IN_STOCK.rows[0];
    if (first === undefined) throw new Error('the fixture has no arrival');
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue({
      ...arrivals.ARRIVALS_IN_STOCK,
      rows: [
        {
          ...first,
          qty_left: 100,
          qty_other: 40,
          shop_name: 'Haji Shop',
          qty_our_left: 60,
          qty_other_left: 40,
        },
      ],
    });
    const { user } = await open();
    await user.click(
      within(typeList()).getByRole('button', {
        name: named(message(language, 'gardens-type-other-sale')),
      }),
    );
    await user.click(
      screen.getByRole('combobox', { name: named(message(language, 'gardens-sale-arrival')) }),
    );
    const option = await screen.findByRole('option');
    // 40 are set for another shop and 60 for ours: both are told (the other shop's part is sold first).
    const words = visible(option.textContent);
    expect(words).toContain('40');
    expect(words).toContain('60');
    await user.click(option);
    await user.type(field('gardens-sale-shop'), 'Haji Shop');
    await user.type(field('gardens-sale-qty'), '20');
    await user.type(field('gardens-sale-amount'), '10000');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toMatchObject({
      type: 'other_sale',
      arrival_id: first.id,
      qty: '20',
    });
  });

  it('R3, R6: the list and the card show the goods left in each part; "change the split" sends the new part with the version', async () => {
    fakeGardens();
    const first = arrivals.ARRIVALS_IN_STOCK.rows[0];
    if (first === undefined) throw new Error('the fixture has no arrival');
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue({
      ...arrivals.ARRIVALS_IN_STOCK,
      rows: [
        {
          ...first,
          qty_left: 100,
          qty_other: 40,
          shop_name: 'Haji Shop',
          qty_our_sold: 0,
          qty_other_sold: 0,
          qty_our_left: 60,
          qty_other_left: 40,
          can_change_split: true,
        },
      ],
    });
    vi.mocked(gardensApi.gardensArrivalChangeSplit).mockResolvedValue({
      id: first.id,
      version: first.version + 1,
      qty_other: 50,
      shop_name: 'Haji Shop',
      qty_our_left: 50,
      qty_other_left: 50,
    });
    const { user } = await open();
    const row = await waitFor(() => {
      const found = document.querySelector('.ag-row[row-id="1"]');
      if (found === null) throw new Error('no row');
      return found;
    });
    expect(visible(row.querySelector('[col-id="qty_our_left"]')?.textContent)).toBe('60');
    expect(visible(row.querySelector('[col-id="qty_other_left"]')?.textContent)).toBe('40');
    await user.click(
      await screen.findByRole('button', { name: said(message(language, 'split-change')) }),
    );
    const dialog = await screen.findByRole('dialog');
    const qty = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(message(language, 'split-other-qty')),
    });
    // The box starts with the part now (40) and the shop's name (NFR-USE-27).
    expect(qty.value).toBe('40');
    expect(
      within(dialog).getByRole<HTMLInputElement>('combobox', {
        name: named(message(language, 'split-shop')),
      }).value,
    ).toBe('Haji Shop');
    await user.click(qty);
    await user.keyboard('{Control>}a{/Control}50');
    await user.click(
      within(dialog).getByRole('button', { name: said(message(language, 'common-save')) }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensArrivalChangeSplit).toHaveBeenCalledWith({
        id: first.id,
        version: first.version,
        other_qty: '50',
        shop_name: 'Haji Shop',
      });
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
  });

  it('D1: a sale in another shop that needs goods from our shop says how many move, before the save', async () => {
    fakeGardens();
    const first = arrivals.ARRIVALS_IN_STOCK.rows[0];
    if (first === undefined) throw new Error('the fixture has no arrival');
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue({
      ...arrivals.ARRIVALS_IN_STOCK,
      rows: [{ ...first, qty_left: 100, qty_our_left: 60, qty_other_left: 40 }],
    });
    // The backend's own preview of a sale linked to an arrival, with the part that moves set to 4.
    const real = arrivals.GARDEN_OTHER_SALE_WITH_ARRIVAL;
    if (real.numbers === null) throw new Error('the file has no numbers');
    vi.mocked(gardensApi.gardensEntryPreview).mockResolvedValue({
      ...real,
      numbers: { ...real.numbers, qty_from_our_shop: 4 },
    });
    const { user } = await open();
    await user.click(
      within(typeList()).getByRole('button', {
        name: named(message(language, 'gardens-type-other-sale')),
      }),
    );
    await user.click(
      screen.getByRole('combobox', { name: named(message(language, 'gardens-sale-arrival')) }),
    );
    await user.click(await screen.findByRole('option'));
    await user.type(field('gardens-sale-shop'), 'Haji Shop');
    await user.type(field('gardens-sale-qty'), '44');
    await user.type(field('gardens-sale-amount'), '20000');
    expect(
      await screen.findByText(said(message(language, 'split-moved-from-our', { count: 4 }))),
    ).toBeInTheDocument();
  });

  it('a garden with no goods arrived says what to do', async () => {
    fakeGardens();
    await open();
    expect(
      await screen.findByText(said(message(language, 'gardens-arrival-empty'))),
    ).toBeInTheDocument();
  });

  it('change the day to sell: the box starts with the day now, and the new day goes with the version', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue(arrivals.ARRIVALS_IN_STOCK);
    vi.mocked(gardensApi.gardensArrivalSetSellDate).mockResolvedValue(
      arrivals.ARRIVAL_SET_SELL_DATE,
    );
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', {
        name: said(message(language, 'purchases-change-sell-date')),
      }),
    );
    const dialog = await screen.findByRole('dialog');
    const box = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(message(language, 'glossary-sell-date')),
    });
    expect(box.value).toBe('12-10-2026');
    await user.click(box);
    await user.keyboard('{Control>}a{/Control}13-10-2026');
    await user.click(
      within(dialog).getByRole('button', { name: said(message(language, 'common-save')) }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensArrivalSetSellDate).toHaveBeenCalledWith({
        id: 1,
        version: 3,
        sell_date: '13-10-2026',
      });
    });
  });

  it('write off: the quantity and the reason go to the backend; more than is left is refused in the box', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue(arrivals.ARRIVALS_IN_STOCK);
    vi.mocked(gardensApi.gardensArrivalWriteOff).mockRejectedValueOnce(
      fail({ code: 'ERR_WRITE_OFF_EXCEEDS_BALANCE', field: 'qty', values: { available: 100 } }),
    );
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', { name: said(message(language, 'purchases-write-off')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('textbox', {
        name: named(message(language, 'gardens-arrival-write-off-qty')),
      }),
      '500',
    );
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'gardens-arrival-reason')),
      }),
      'Spoiled',
    );
    await user.click(
      within(dialog).getByRole('button', {
        name: said(message(language, 'gardens-arrival-write-off-confirm')),
      }),
    );
    expect(
      await within(dialog).findByText(
        said(message(language, 'err-write-off-exceeds-balance', { available: 100 })),
      ),
    ).toBeInTheDocument();
    expect(gardensApi.gardensArrivalWriteOff).toHaveBeenCalledWith({
      arrival_id: 1,
      qty: '500',
      reason: 'Spoiled',
    });
  });

  it('cancel: the reason goes to the backend with the version; an entry with sales cannot be cancelled', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue(arrivals.ARRIVALS_IN_STOCK);
    vi.mocked(gardensApi.gardensArrivalCancel).mockResolvedValue(arrivals.ARRIVAL_CANCEL);
    const { user } = await open();
    await screen.findByText(
      said(
        message(language, 'gardens-arrival-card-title', {
          number: message(language, 'gardens-arrival-number', { number: 1 }),
          item: language === 'ur' ? 'کیلا' : 'Banana',
        }),
      ),
    );
    await user.click(
      await screen.findByRole('button', {
        name: said(message(language, 'gardens-arrival-cancel')),
      }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'gardens-arrival-reason')),
      }),
      'Wrong',
    );
    await user.click(
      within(dialog).getByRole('button', {
        name: said(message(language, 'gardens-arrival-cancel-confirm')),
      }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensArrivalCancel).toHaveBeenCalledWith({
        id: 1,
        version: 3,
        reason: 'Wrong',
      });
    });
  });

  it('an entry that was sold in part cannot be cancelled: the button is dimmed', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue({
      ...arrivals.ARRIVALS_IN_STOCK,
      rows: [arrivals.ARRIVAL_AFTER_A_SALE],
    });
    await open();
    expect(
      await screen.findByRole('button', {
        name: said(message(language, 'gardens-arrival-cancel')),
      }),
    ).toBeDisabled();
  });
});
