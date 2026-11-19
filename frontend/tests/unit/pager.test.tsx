// The page buttons under every list (PRD FR-19.11, plan 38 R1): one page at a time from the backend,
// "26 to 50 of 340", rows per page (25, 50, 100) remembered for the screen, "Page 2 of 14", First,
// Previous, Next and Last, Page Up and Page Down in the table, a new filter back on page 1, the
// totals over the whole filter on every page, and a language switch that keeps the page. The backend
// is a fake that counts the pages it is asked for.

import { Localized } from '@fluent/react';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders, createQueryClient } from '../../src/app/providers';
import { PagedList, type ListColumn } from '../../src/shared/grid';
import { pageWindow, useClientPager, usePagedQuery } from '../../src/shared/hooks';
import { useLanguageStore } from '../../src/shared/i18n';
import * as screenApi from '../../src/shared/screen-state/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { message } from '../helpers/render';

type Which = 'a' | 'b';

interface Row {
  id: number;
  name: string;
}
interface Page {
  rows: Row[];
  total: number;
  /** The whole filter's figure, the same on every page. */
  sum: string;
}

const COLUMNS: ListColumn<Row>[] = [
  { field: 'name', type: 'text', headerId: 'common-back', value: (row) => row.name },
];

/** The fake backend: `count` rows named "Row 1" ..., and a log of every page asked for. */
function fakeBackend(count: number) {
  const calls: { filter: string; offset: number; limit: number }[] = [];
  const fetchPage = (filter: string, offset: number, limit: number): Promise<Page> => {
    calls.push({ filter, offset, limit });
    const all = Array.from({ length: count }, (_, index) => ({
      id: index + 1,
      name: `Row ${String(index + 1)}`,
    }));
    return Promise.resolve({
      rows: all.slice(offset, offset + limit),
      total: count,
      sum: '12,345',
    });
  };
  return { calls, fetchPage };
}

function List({
  which,
  fetchPage,
}: {
  which: Which;
  fetchPage: (filter: string, offset: number, limit: number) => Promise<Page>;
}) {
  const query = usePagedQuery({
    screen: 'test.list',
    queryKey: ['test-list', which],
    fetchPage: (offset, limit) => fetchPage(which, offset, limit),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total,
  });
  const page = query.data?.pages[0];
  return (
    <PagedList<Row>
      columns={COLUMNS}
      rows={page?.rows}
      getRowId={(row) => String(row.id)}
      labelId="common-back"
      failedId="common-back"
      error={null}
      onRetry={() => undefined}
      pager={query.pager}
      empty={<Localized id="common-back" />}
      totals={() => ({ labelField: 'name', values: { name: page?.sum ?? '' } })}
    />
  );
}

function Filtered({
  fetchPage,
}: {
  fetchPage: (filter: string, offset: number, limit: number) => Promise<Page>;
}) {
  const [filter, setFilter] = useState<Which>('a');
  return (
    <>
      <button
        type="button"
        onClick={() => {
          setFilter('b');
        }}
      >
        <Localized id="common-retry" />
      </button>
      <List which={filter} fetchPage={fetchPage} />
    </>
  );
}

/** The filter of the harness list: a fixed choice, so it is a value of code, not a word. */
const FILTER_A: Which = 'a';

function mount(node: React.ReactElement) {
  return render(<AppProviders queryClient={createQueryClient()}>{node}</AppProviders>);
}

/**
 * The ids of the rows the grid has drawn, by their place in the list. The grid keeps the elements
 * of rows that stay when the rows change, so the order of the elements is not the order of the rows.
 */
function rowIds(): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-id]'))
    .flatMap((row) => {
      const id = row.getAttribute('row-id');
      const index = Number(row.getAttribute('row-index'));
      return id === null || row.classList.contains('ag-row-pinned') ? [] : [{ id, index }];
    })
    .sort((a, b) => a.index - b.index)
    .map((row) => row.id);
}

/** Waits until the first row on screen is row `id` (a cell with a number is drawn in pieces). */
async function expectFirstRow(id: number) {
  await waitFor(() => {
    expect(rowIds()[0]).toBe(String(id));
  });
}

/** The words of an element without the isolation marks Fluent puts round numbers. */
function plain(element: HTMLElement): string {
  return element.textContent.replace(/[⁦-⁩]/g, '');
}

function navigationOf(language: Language) {
  return screen.findByRole('navigation', { name: message(language, 'common-pager-label') });
}

const DEFAULT_GET = ({ screen: name }: { screen: string }) =>
  Promise.resolve({ screen: name, value: null });

beforeEach(() => {
  act(() => {
    useLanguageStore.setState({ language: 'en' });
  });
  vi.mocked(screenApi.prefsScreenGet).mockImplementation(DEFAULT_GET);
  vi.mocked(screenApi.prefsScreenSet).mockClear();
});

describe('the page window', () => {
  it('gives the rows of a page and the number of pages', () => {
    expect(pageWindow(1, 25, 340)).toEqual({ page: 1, pageCount: 14, from: 1, to: 25 });
    expect(pageWindow(14, 25, 340)).toEqual({ page: 14, pageCount: 14, from: 326, to: 340 });
    expect(pageWindow(2, 50, 340)).toEqual({ page: 2, pageCount: 7, from: 51, to: 100 });
  });

  it('keeps a page inside the pages, and an empty list on one page', () => {
    expect(pageWindow(99, 25, 340).page).toBe(14);
    expect(pageWindow(0, 25, 340).page).toBe(1);
    expect(pageWindow(1, 25, 0)).toEqual({ page: 1, pageCount: 1, from: 0, to: 0 });
  });
});

describe('the pager under a list', () => {
  it('opens on page 1 of 25 rows with the count of the whole filter', async () => {
    const backend = fakeBackend(340);
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf('en');
    // The grid draws the rows in view; the pager says how many the page holds.
    await expectFirstRow(1);
    expect(rowIds()).not.toContain('26');
    expect(plain(nav)).toContain('1 to 25 of 340');
    expect(plain(nav)).toContain('Page 1 of 14');
    expect(backend.calls).toEqual([{ filter: 'a', offset: 0, limit: 25 }]);
  });

  it('Next, Last, Previous and First move by the page and ask the backend for each', async () => {
    const backend = fakeBackend(340);
    const user = userEvent.setup();
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf('en');
    const button = (name: string) => within(nav).getByRole('button', { name });
    await expectFirstRow(1);
    expect(button('First page')).toBeDisabled();
    expect(button('Previous page')).toBeDisabled();

    await user.click(button('Next page'));
    await expectFirstRow(26);
    expect(plain(nav)).toContain('26 to 50 of 340');
    expect(plain(nav)).toContain('Page 2 of 14');

    await user.click(button('Last page'));
    await expectFirstRow(326);
    expect(plain(nav)).toContain('326 to 340 of 340');
    expect(button('Next page')).toBeDisabled();
    expect(button('Last page')).toBeDisabled();

    await user.click(button('Previous page'));
    await expectFirstRow(301);
    await user.click(button('First page'));
    await expectFirstRow(1);
    expect(backend.calls.map((call) => call.offset)).toEqual([0, 25, 325, 300, 0]);
    expect(backend.calls.every((call) => call.limit === 25)).toBe(true);
  });

  it('the totals row is the backend figure of the whole filter on every page', async () => {
    const backend = fakeBackend(340);
    const user = userEvent.setup();
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf('en');
    expect(await screen.findAllByText('12,345')).toHaveLength(1);
    await user.click(within(nav).getByRole('button', { name: 'Next page' }));
    await expectFirstRow(26);
    expect(await screen.findAllByText('12,345')).toHaveLength(1);
  });

  it('rows per page asks for that many from page 1 and is remembered for the screen', async () => {
    const backend = fakeBackend(340);
    const user = userEvent.setup();
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf('en');
    await user.click(within(nav).getByRole('button', { name: 'Next page' }));
    await expectFirstRow(26);
    await user.click(within(nav).getByRole('combobox', { name: 'Rows per page' }));
    await user.click(await screen.findByRole('option', { name: '50' }));
    await expectFirstRow(1);
    expect(plain(nav)).toContain('1 to 50 of 340');
    expect(plain(nav)).toContain('Page 1 of 7');
    expect(backend.calls.at(-1)).toEqual({ filter: 'a', offset: 0, limit: 50 });
    await waitFor(() => {
      expect(screenApi.prefsScreenSet).toHaveBeenCalledWith({
        screen: 'pager.test.list',
        value: '{"size":50}',
      });
    });
  });

  it('starts with the rows per page that was kept for the screen', async () => {
    vi.mocked(screenApi.prefsScreenGet).mockImplementation(({ screen: name }) =>
      Promise.resolve({ screen: name, value: name === 'pager.test.list' ? '{"size":100}' : null }),
    );
    const backend = fakeBackend(340);
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf('en');
    await expectFirstRow(1);
    expect(plain(nav)).toContain('Page 1 of 4');
    // The first page is asked for once, with the size that was kept.
    expect(backend.calls).toEqual([{ filter: 'a', offset: 0, limit: 100 }]);
  });

  it('a new filter goes back to page 1', async () => {
    const backend = fakeBackend(340);
    const user = userEvent.setup();
    mount(<Filtered fetchPage={backend.fetchPage} />);
    const nav = await navigationOf('en');
    await user.click(within(nav).getByRole('button', { name: 'Next page' }));
    await user.click(within(nav).getByRole('button', { name: 'Next page' }));
    await expectFirstRow(51);
    await user.click(screen.getByRole('button', { name: message('en', 'common-retry') }));
    await expectFirstRow(1);
    // The list drew itself again for the new filter, so its pager is a new element.
    expect(plain(await navigationOf('en'))).toContain('Page 1 of 14');
    expect(backend.calls.at(-1)).toEqual({ filter: 'b', offset: 0, limit: 25 });
    // The page of the old filter is never asked for with the new filter.
    expect(backend.calls.filter((call) => call.filter === 'b' && call.offset !== 0)).toEqual([]);
  });

  it('a list that fits one page shows its count and no buttons', async () => {
    const backend = fakeBackend(10);
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf('en');
    await expectFirstRow(1);
    expect(plain(nav)).toContain('1 to 10 of 10');
    expect(within(nav).queryByRole('button')).not.toBeInTheDocument();
    expect(within(nav).queryByRole('combobox')).not.toBeInTheDocument();
  });

  it('comes back to the page it was on when the screen opens again', async () => {
    const backend = fakeBackend(340);
    const user = userEvent.setup();
    const first = mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf('en');
    await user.click(within(nav).getByRole('button', { name: 'Next page' }));
    await user.click(within(nav).getByRole('button', { name: 'Next page' }));
    await expectFirstRow(51);
    first.unmount();
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    await expectFirstRow(51);
    expect(backend.calls.at(-1)).toEqual({ filter: 'a', offset: 50, limit: 25 });
  });
});

describe('Page Up and Page Down in the table', () => {
  it('Page Down turns to the next page and puts focus on its first row', async () => {
    const backend = fakeBackend(340);
    const user = userEvent.setup();
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf('en');
    await expectFirstRow(1);
    const cell = document.querySelector('.ag-row[row-id="3"] [col-id="name"]');
    if (cell === null) throw new Error('no cell');
    await user.click(cell);
    await user.keyboard('{PageDown}');
    await expectFirstRow(26);
    expect(plain(nav)).toContain('Page 2 of 14');
    await waitFor(() => {
      expect(
        document.querySelector('.ag-cell-focus')?.closest('.ag-row')?.getAttribute('row-id'),
      ).toBe('26');
      // The browser focus is on that cell too: the next key goes to it.
      expect(document.activeElement?.closest('.ag-row')?.getAttribute('row-id')).toBe('26');
    });
    await user.keyboard('{PageUp}');
    await expectFirstRow(1);
    expect(plain(nav)).toContain('Page 1 of 14');
  });

  it('Page Up on the first page and Page Down on the last page change nothing', async () => {
    const backend = fakeBackend(30);
    const user = userEvent.setup();
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    await expectFirstRow(1);
    const cell = document.querySelector('.ag-row[row-id="1"] [col-id="name"]');
    if (cell === null) throw new Error('no cell');
    await user.click(cell);
    await user.keyboard('{PageUp}');
    expect(backend.calls).toHaveLength(1);
  });
});

describe.each(LANGUAGES)('the pager in %s', (language) => {
  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  it('says the count and the page in the app language, and mirrors the arrows in Urdu', async () => {
    const backend = fakeBackend(340);
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf(language);
    await expectFirstRow(1);
    const range = language === 'ur' ? '340 میں سے 1 تا 25' : '1 to 25 of 340';
    const pageOf = language === 'ur' ? 'صفحہ 1 از 14' : 'Page 1 of 14';
    expect(plain(nav)).toContain(range);
    expect(plain(nav)).toContain(pageOf);
    for (const id of ['common-pager-first', 'common-pager-next']) {
      const icon = within(nav)
        .getByRole('button', { name: message(language, id) })
        .querySelector('svg');
      // The classes of an arrow turn it round in right-to-left (Tailwind `rtl:rotate-180`).
      expect(icon?.getAttribute('class')).toContain('rtl:rotate-180');
    }
  });

  it('a language switch keeps the page', async () => {
    const backend = fakeBackend(340);
    const user = userEvent.setup();
    mount(<List which={FILTER_A} fetchPage={backend.fetchPage} />);
    const nav = await navigationOf(language);
    await user.click(
      within(nav).getByRole('button', { name: message(language, 'common-pager-next') }),
    );
    await expectFirstRow(26);
    const other = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    const after = await navigationOf(other);
    await expectFirstRow(26);
    expect(plain(after)).toContain(other === 'ur' ? 'صفحہ 2 از 14' : 'Page 2 of 14');
    // The page is the same one: it was not asked for again from the start.
    expect(backend.calls.map((call) => call.offset)).not.toContain(300);
  });
});

describe('a list that is already in memory', () => {
  function Short() {
    const rows = Array.from({ length: 60 }, (_, index) => ({
      id: index + 1,
      name: `Row ${String(index + 1)}`,
    }));
    const [text, setText] = useState('');
    const shown = text === '' ? rows : rows.filter((row) => row.name.endsWith('5'));
    const client = useClientPager('test.short', text, shown);
    return (
      <>
        <button
          type="button"
          onClick={() => {
            setText('5');
          }}
        >
          <Localized id="common-save" />
        </button>
        <PagedList<Row>
          columns={COLUMNS}
          rows={client.pageRows}
          getRowId={(row) => String(row.id)}
          labelId="common-back"
          failedId="common-back"
          error={null}
          onRetry={() => undefined}
          pager={client.pager}
          empty={<Localized id="common-back" />}
        />
      </>
    );
  }

  it('cuts the page from the rows and goes back to page 1 for a new filter', async () => {
    const user = userEvent.setup();
    mount(<Short />);
    const nav = await navigationOf('en');
    await expectFirstRow(1);
    expect(plain(nav)).toContain('1 to 25 of 60');
    await user.click(within(nav).getByRole('button', { name: 'Next page' }));
    await expectFirstRow(26);
    expect(rowIds()).not.toContain('1');
    await user.click(screen.getByRole('button', { name: message('en', 'common-save') }));
    await expectFirstRow(5);
    expect(plain(nav)).toContain('1 to 6 of 6');
  });
});
