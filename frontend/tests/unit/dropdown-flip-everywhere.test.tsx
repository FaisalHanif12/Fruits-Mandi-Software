// Plan 86 (Umar, 20-11-2026): "when the dropdown is open, if there is less space under, it must open upward so
// all the options are visible". Every list that drops down opens below its field when it fits and above it
// when there is no room below and more above, with its height cut to the room, inside the window, not cut
// by a card or a grid that hides what leaves it, and with the same keys as before (PRD FR-7.16). Here the
// real components are drawn: the master picker, the suggestion list, the Urdu spellings, the key map, the
// quality cell of a grid and the picker cell of a grid. The box itself is tested in dropdown-popup.test.tsx
// and its rules in dropdown-position.test.ts. Urdu and English. jsdom has no layout, so the field's place
// and the size of the list are given (tests/helpers/dropdown-layout.ts).

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState, type ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import type { SearchHit } from '../../src/shared/search';
import * as searchApi from '../../src/shared/search/api';
import {
  Field,
  Picker,
  RomanUrduInput,
  SuggestInput,
  radixCollisionPadding,
  type PickerValue,
} from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { mockDropdownLayout, type DropdownLayout, type RectSpec } from '../helpers/dropdown-layout';
import { GoodsLinesGrid } from '../../src/features/purchases';
import { createEntryStore } from '../../src/shared/grid';
import { renderGrid } from '../helpers/entry-grid';
import { message, renderIn } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** The window of the smallest supported laptop and a 32 px status bar. */
const WINDOW = { width: 1366, height: 768 };
const STATUS_BAR: RectSpec = { top: 736, left: 0, width: 1366, height: 32 };
const FIELD_SIZE = { width: 280, height: 44 };
/** A list of 12 rows, taller than the room on either side. */
const LONG_LIST = { width: 280, height: 700 };
const SHORT_LIST = { width: 280, height: 200 };
const NEAR_TOP = { top: 100 };
const NEAR_BOTTOM = { top: 620 };

let layout: DropdownLayout | null = null;
afterEach(() => {
  layout?.restore();
  layout = null;
});

/** Fixes the layout; `field` finds the element the list hangs from, when the page is drawn. */
function placeField(
  field: () => Element | null,
  at: Partial<RectSpec>,
  list: { width: number; height: number },
): void {
  layout = mockDropdownLayout({
    window: WINDOW,
    field,
    fieldRect: { top: 100, left: 300, ...FIELD_SIZE, ...at },
    list,
    bars: { 'status-bar': STATUS_BAR },
  });
}

/** The box of the open list: the one that is shown. */
const openBox = () => {
  const boxes = [...document.querySelectorAll<HTMLElement>('[data-dropdown-popup]')].filter(
    (box) => !box.classList.contains('hidden'),
  );
  if (boxes.length !== 1) throw new Error(`${String(boxes.length)} lists are open, one was meant`);
  const [box] = boxes;
  if (box === undefined) throw new Error('no list is open');
  return box;
};
const comboField = () => screen.getByRole<HTMLInputElement>('combobox');
/** The box that holds an input and its list, which the list hangs from. */
const boxOf = (input: HTMLElement) => input.parentElement;

/** A card or a grid that cuts what leaves it. */
function Card({ children }: { children: ReactNode }) {
  return (
    <div>
      <footer data-testid="status-bar" data-dropdown-avoid="" />
      <div data-testid="card" style={{ overflow: 'hidden' }}>
        {children}
      </div>
    </div>
  );
}

function city(id: number, code: string, ur: string, en: string): SearchHit {
  return {
    id,
    kind: 'city',
    code,
    name_ur: ur,
    name_en: en,
    extra_ur: '',
    extra_en: '',
    is_active: true,
    sold_by_weight: false,
  };
}
const CITIES = [
  city(1, '0001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax'),
  city(2, '0002', 'ارشد ملتانی', 'Arshad Multani'),
  city(3, '0003', 'اسلم', 'Aslam'),
];

/** The name of the second record in each language: the field shows the one of the app language. */
const ARSHAD = { ur: 'ارشد ملتانی', en: 'Arshad Multani' } as const;

function PickerHarness() {
  const [value, setValue] = useState<PickerValue | null>(null);
  useEscapeDispatcher();
  return (
    <Card>
      <form {...formNavigationProps}>
        <Field label={message('en', 'common-back')}>
          {(control) => (
            <Picker
              {...control}
              scope="city"
              value={value}
              onChange={(next) => {
                setValue(
                  next === null
                    ? null
                    : {
                        id: next.id,
                        code: next.code,
                        name_ur: next.name_ur,
                        name_en: next.name_en,
                      },
                );
              }}
            />
          )}
        </Field>
        <input data-testid="next" />
      </form>
    </Card>
  );
}

function openPicker(language: Language, at: Partial<RectSpec>, list = SHORT_LIST) {
  vi.mocked(searchApi.searchMasters).mockImplementation(({ text }) =>
    Promise.resolve(CITIES.filter((row) => row.name_en.toLowerCase().includes(text.toLowerCase()))),
  );
  vi.mocked(searchApi.searchRecent).mockResolvedValue(CITIES);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
  act(() => {
    useLanguageStore.setState({ language });
  });
  placeField(() => boxOf(screen.queryByRole('combobox') ?? document.body), at, list);
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  renderIn(
    language,
    <QueryClientProvider client={client}>
      <PickerHarness />
    </QueryClientProvider>,
  );
  return userEvent.setup();
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe.each(LANGUAGES)('the master picker flips its list (%s)', (language) => {
  it('opens below the field near the top of the window', async () => {
    const user = openPicker(language, NEAR_TOP);
    await user.click(comboField());
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(3);
    });
    expect(openBox()).toHaveAttribute('data-side', 'bottom');
  });

  it('opens above the field near the bottom of the window, with every option reachable', async () => {
    const user = openPicker(language, NEAR_BOTTOM);
    await user.click(comboField());
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(3);
    });
    const box = openBox();
    expect(box).toHaveAttribute('data-side', 'top');
    // tied to the top of the field, so it grows upward: 768 - (620 - 4)
    expect(box.style.insetBlockEnd).toBe('152px');
  });

  it('cuts its height to the room, clear of the status bar', async () => {
    const user = openPicker(language, { top: 300 }, LONG_LIST);
    await user.click(comboField());
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(3);
    });
    const box = openBox();
    // the status bar starts at 736; 8 px edge; field bottom 344 and a 4 px gap
    expect(box).toHaveAttribute('data-side', 'bottom');
    expect(box.style.maxHeight).toBe('380px');
  });

  it('is not cut by a card that hides what leaves it', async () => {
    const user = openPicker(language, NEAR_BOTTOM);
    await user.click(comboField());
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(3);
    });
    const card = screen.getByTestId('card');
    expect(card.contains(screen.getByRole('listbox'))).toBe(false);
    expect(openBox().parentElement).toBe(document.body);
  });

  it('keeps its keys: ↓ moves, Enter picks and moves on, Esc closes the list', async () => {
    const user = openPicker(language, NEAR_BOTTOM);
    await user.click(comboField());
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(3);
    });
    await user.keyboard('{ArrowDown}{ArrowDown}');
    expect(screen.getAllByRole('option')[1]).toHaveAttribute('aria-selected', 'true');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(comboField().value).toContain(ARSHAD[language]);
    });
    // Enter moved on to the next field
    expect(screen.getByTestId('next')).toHaveFocus();
    expect(comboField()).toHaveAttribute('aria-expanded', 'false');

    // Esc closes an open list and keeps the field
    await user.click(comboField());
    await waitFor(() => {
      expect(comboField()).toHaveAttribute('aria-expanded', 'true');
    });
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(comboField()).toHaveAttribute('aria-expanded', 'false');
    });
  });

  it('a click on a row picks it, though the list is drawn on the page and not in the field', async () => {
    const user = openPicker(language, NEAR_BOTTOM);
    await user.click(comboField());
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(3);
    });
    const [, second] = screen.getAllByRole('option');
    if (second === undefined) throw new Error('no second row');
    await user.click(second);
    await waitFor(() => {
      expect(comboField().value).toContain(ARSHAD[language]);
    });
  });
});

describe.each(LANGUAGES)('the suggestion list flips (%s)', (language) => {
  function Rig() {
    const [value, setValue] = useState('');
    return (
      <Card>
        <SuggestInput
          id="truck"
          value={value}
          onChange={setValue}
          suggestions={[{ value: 'LES-1234' }, { value: 'LES-5678' }]}
        />
      </Card>
    );
  }

  it.each([
    ['near the top', NEAR_TOP, 'bottom'],
    ['near the bottom', NEAR_BOTTOM, 'top'],
  ] as const)('opens %s: %s', async (_name, at, side) => {
    placeField(() => boxOf(screen.queryByRole('combobox') ?? document.body), at, SHORT_LIST);
    renderIn(language, <Rig />);
    const user = userEvent.setup();
    await user.click(comboField());
    await user.keyboard('LES');
    expect(await screen.findAllByRole('option')).toHaveLength(2);
    expect(openBox()).toHaveAttribute('data-side', side);
    expect(screen.getByTestId('card').contains(openBox())).toBe(false);
  });
});

describe.each(LANGUAGES)('the Urdu spellings and the key map flip (%s)', (language) => {
  function Rig({ mode }: { mode: 'roman' | 'keyboard' }) {
    const [value, setValue] = useState('');
    useEscapeDispatcher();
    return (
      <Card>
        <Field label={message('en', 'common-back')}>
          {(control) => (
            <RomanUrduInput {...control} value={value} onChange={setValue} mode={mode} />
          )}
        </Field>
      </Card>
    );
  }

  it('lists the spellings below the field near the top and above it near the bottom', async () => {
    vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
      Promise.resolve(suggestionsFor({ toUrdu: { tariq: ['طارق', 'تارق'] } }, text, direction)),
    );
    for (const [at, side] of [
      [NEAR_TOP, 'bottom'],
      [NEAR_BOTTOM, 'top'],
    ] as const) {
      placeField(() => boxOf(screen.queryByRole('combobox') ?? document.body), at, SHORT_LIST);
      const view = renderIn(language, <Rig mode="roman" />);
      const user = userEvent.setup();
      await user.click(comboField());
      await user.keyboard('tariq');
      await waitFor(() => {
        expect(screen.getAllByRole('option')).toHaveLength(2);
      });
      expect(openBox()).toHaveAttribute('data-side', side);
      // the field is written right to left in both apps: the list carries that direction too
      expect(openBox()).toHaveAttribute('dir', 'rtl');
      layout?.restore();
      view.unmount();
    }
  });

  it('opens the key map above the field near the bottom of the window', async () => {
    placeField(() => boxOf(screen.queryByRole('combobox') ?? document.body), NEAR_BOTTOM, {
      width: 280,
      height: 220,
    });
    renderIn(language, <Rig mode="keyboard" />);
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', { name: message(language, 'translit-keymap-open') }),
    );
    expect(
      await screen.findByRole('group', { name: message(language, 'translit-keymap-title') }),
    ).toBeInTheDocument();
    expect(openBox()).toHaveAttribute('data-side', 'top');
  });
});

describe.each(LANGUAGES)('a list in a grid cell flips (%s)', (language) => {
  it('the quality list of a cell near the bottom hangs from the cell and opens above it', async () => {
    const rig = await renderGrid(language, { withSelect: true, rows: [{}] });
    placeField(() => rig.cell(0, 'grade'), { top: 640, height: 44 }, { width: 200, height: 176 });
    await rig.focusCell(0, 'rate');
    await rig.user.keyboard(language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');
    await rig.user.keyboard(' ');
    await waitFor(() => {
      expect(screen.getByRole('listbox')).toBeInTheDocument();
    });
    const box = openBox();
    expect(box).toHaveAttribute('data-side', 'top');
    // drawn on the page, not in the grid, so the grid cannot cut it
    expect(document.querySelector('.entry-grid')?.contains(box)).toBe(false);
    expect(screen.getAllByRole('option').length).toBeGreaterThan(1);
    // the keys are as they were: ↓ ↓ Enter chooses the second word and closes the list
    await rig.user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    expect(rig.typed(0, 'grade')).toBe('second');
    expect(screen.queryByRole('listbox')).toBeNull();
  });

  it('the item cell of the garden goods arrived line, near the bottom, opens its list above its field', async () => {
    vi.mocked(searchApi.searchMasters).mockImplementation(({ text }) =>
      Promise.resolve(
        CITIES.filter((row) => row.name_en.toLowerCase().includes(text.toLowerCase())),
      ),
    );
    vi.mocked(searchApi.searchRecent).mockResolvedValue(CITIES);
    vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
    act(() => {
      useLanguageStore.setState({ language });
    });
    const store = createEntryStore();
    placeField(
      // the field of the picker is drawn over the cell, so the list hangs from that field
      () => document.querySelector('input[role="combobox"]')?.parentElement ?? null,
      NEAR_BOTTOM,
      LONG_LIST,
    );
    renderIn(
      language,
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <GoodsLinesGrid kind="arrival" store={store} />
      </QueryClientProvider>,
    );
    const user = userEvent.setup();
    await waitFor(() => {
      expect(
        document.querySelector('.ag-row[row-index="0"] .ag-cell[col-id="item"]'),
      ).not.toBeNull();
    });
    const cell = document.querySelector<HTMLElement>(
      '.ag-row[row-index="0"] .ag-cell[col-id="item"]',
    );
    if (cell === null) throw new Error('no item cell');
    await user.click(cell);
    const field = await screen.findByRole('combobox');
    await waitFor(() => {
      expect(openBox()).toHaveAttribute('data-side', 'top');
    });
    // the grid's own popup holds only the field: no border or shadow of a second box around it
    const popup = field.closest<HTMLElement>('.ag-custom-component-popup');
    expect(popup).not.toBeNull();
    expect(popup).not.toHaveClass('border');
    expect(popup).not.toHaveClass('shadow-overlay');
    expect(popup?.contains(openBox())).toBe(false);
    expect(document.querySelector('.entry-grid')?.contains(openBox())).toBe(false);
  });
});

describe('the menus keep clear of the bars of the shell', () => {
  it('gives Radix the same margins: under the top bar and above the status bar', () => {
    // jsdom has no styles: the sizes of the two bars are the ones of the tokens (60 and 32 px) plus 8 px
    expect(radixCollisionPadding()).toEqual({ top: 68, right: 8, bottom: 40, left: 8 });
  });
});
