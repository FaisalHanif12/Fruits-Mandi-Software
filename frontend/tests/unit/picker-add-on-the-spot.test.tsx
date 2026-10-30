// Plan 41, PRD FR-7.69, FR-7.42, E74: a missing record is added from the list where it is needed.
// On the shared master picker: the open list ends with one row ("Add a new city", or the typed text
// named), an empty list says so in words, the row is reached by a click, by ↓ then Enter, by Enter on
// text that matches nothing, and by F4; the short form is asked of the app with the typed text; a
// saved record is chosen in the field; closing the form without a save leaves the field as it was;
// a filter box and a field without the app's forms have no row. Both languages, against a fake of
// the search and a fake of the app's host.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import type { SearchHit, SearchScope } from '../../src/shared/search';
import {
  AddRecordContext,
  Field,
  Picker,
  type AddRecord,
  type AddRecordRequest,
  type PickerValue,
} from '../../src/shared/ui';
import { message, paragraph, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/search/api');

function hit(id: number, scope: SearchScope): SearchHit {
  return {
    id,
    kind: scope,
    code: String(id).padStart(4, '0'),
    name_ur: `نام ${String(id)}`,
    name_en: `Name ${String(id)}`,
    extra_ur: '',
    extra_en: '',
    is_active: true,
    sold_by_weight: false,
  };
}

/** The fake search: `total` records, found by code or English name; the new one is added later. */
function fakeSearch(scope: SearchScope, total: number, extra: SearchHit[] = []) {
  const all = [...Array.from({ length: total }, (_, index) => hit(index + 1, scope)), ...extra];
  vi.mocked(searchApi.searchMasters).mockImplementation(({ text, offset = 0, limit }) => {
    const wanted = text.trim().toLowerCase();
    const found = all.filter(
      (row) =>
        wanted === '' || row.code.includes(wanted) || row.name_en.toLowerCase().includes(wanted),
    );
    return Promise.resolve(found.slice(offset, offset + limit));
  });
  vi.mocked(searchApi.searchRecent).mockResolvedValue(all);
  vi.mocked(searchApi.findMaster).mockImplementation((_scope, id) =>
    Promise.resolve(all.find((row) => row.id === id) ?? null),
  );
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}

interface HarnessProps {
  scope?: SearchScope;
  host?: AddRecord | null;
  canAdd?: boolean;
  initial?: PickerValue | null;
  addOwner?: PickerValue | null;
}

function Harness({ scope = 'city', host = null, canAdd, initial = null, addOwner }: HarnessProps) {
  const [value, setValue] = useState<PickerValue | null>(initial);
  useEscapeDispatcher();
  return (
    <AddRecordContext.Provider value={host}>
      <form {...formNavigationProps}>
        <input data-testid="before" />
        <Field label={message('en', 'common-back')}>
          {(control) => (
            <Picker
              {...control}
              scope={scope}
              value={value}
              {...(canAdd === undefined ? {} : { canAdd })}
              {...(addOwner === undefined ? {} : { addOwner })}
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
        <input data-testid="after" />
      </form>
    </AddRecordContext.Provider>
  );
}

function setup(language: Language, props: HarnessProps = {}) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Harness {...props} />
    </QueryClientProvider>,
  );
  const input = screen.getByRole<HTMLInputElement>('combobox');
  return { input, user: userEvent.setup() };
}

const options = () => screen.queryAllByRole('option');

/** The add row: a button inside the open list's popup, named by the Fluent message of the kind. */
const addRow = (language: Language, noun: string, text?: string) =>
  screen.queryByRole('button', {
    name: (name) =>
      visible(name).startsWith(
        text === undefined
          ? message(language, 'search-add-row', { noun })
          : message(language, 'search-add-row-named', { noun, text }),
      ),
  });

/** The add row, which the test needs to be there. */
function theRow(language: Language, noun: string, text?: string): HTMLElement {
  const row = addRow(language, noun, text);
  if (row === null) throw new Error('no add row');
  return row;
}

function pressF4(input: HTMLInputElement) {
  // user-event does not know F4 by code, so press it the way the window gets it.
  act(() => {
    input.dispatchEvent(new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }));
  });
}

/** The kinds the app can add, with the word each has in the add row's messages. */
const KINDS = [
  ['city', 'city'],
  ['item', 'item'],
  ['packing', 'packing'],
  ['adda', 'adda'],
  ['marka', 'marka'],
  ['store', 'store'],
  ['narration', 'narration'],
  ['expense_head', 'expense-head'],
  ['staff', 'staff'],
] as const;

beforeEach(() => {
  vi.clearAllMocks();
  fakeSearch('city', 3);
});

describe.each(LANGUAGES)('the add row of a picker (%s)', (language) => {
  it('ends the open list: the matches first, then one row, which is not a match', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    const row = await screen.findByRole('button', {
      name: (name) =>
        visible(name).startsWith(message(language, 'search-add-row', { noun: 'city' })),
    });
    // It comes after the last match in the page, and the matches are still three.
    const last = options()[2];
    if (last === undefined) throw new Error('no last match');
    expect(last.compareDocumentPosition(row) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(options()).toHaveLength(3);
    // It shows its key, F4, as every button of the app does (PRD FR-7.4).
    expect(visible(row.textContent)).toContain('F4');
  });

  it.each(KINDS)(
    'an empty %s list says so in words and offers the row, never an empty box',
    async (scope, noun) => {
      fakeSearch(scope, 0);
      const host = vi.fn();
      const { input, user } = setup(language, { scope, host });
      await user.click(input);
      expect(
        await screen.findByText(paragraph(message(language, 'search-empty-kind', { noun }))),
      ).toBeInTheDocument();
      expect(addRow(language, noun)).toBeInTheDocument();
      await user.click(theRow(language, noun));
      expect(host).toHaveBeenCalledWith(expect.objectContaining({ text: '' }));
    },
  );

  it('names the typed text in the row, and says nothing matched', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    expect(addRow(language, 'city', 'zzz')).toBeInTheDocument();
  });

  it('a click on the row asks the app for the short form of the kind, with the typed text', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(addRow(language, 'city', 'zzz')).toBeInTheDocument();
    });
    await user.click(theRow(language, 'city', 'zzz'));
    expect(host).toHaveBeenCalledTimes(1);
    const request = host.mock.calls[0]?.[0] as AddRecordRequest;
    expect(request.kind).toBe('city');
    expect(request.text).toBe('zzz');
    expect(request.form).toBeUndefined();
    // The form covers the screen and takes the focus: nothing typed is dropped.
    act(() => {
      input.blur();
    });
    expect(input.value).toBe('zzz');
  });

  it('a record saved in the form is chosen in the field, and the cursor stays there', async () => {
    const created = { ...hit(9, 'city'), name_ur: 'نیا شہر', name_en: 'New City' };
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await user.keyboard('New');
    await waitFor(() => {
      expect(addRow(language, 'city', 'New')).toBeInTheDocument();
    });
    await user.click(theRow(language, 'city', 'New'));
    const request = host.mock.calls[0]?.[0] as AddRecordRequest;
    // The record is in the master list now; the picker asks its own search for the row.
    fakeSearch('city', 3, [created]);
    act(() => {
      input.blur();
      request.onClosed({ id: 9, code: '0009' });
    });
    await waitFor(() => {
      expect(input.value).toBe(language === 'ur' ? created.name_ur : created.name_en);
    });
    expect(input).toHaveFocus();
    expect(input).toHaveAttribute('aria-expanded', 'false');
    // It is remembered as the last chosen, like any pick (PRD FR-7.41).
    expect(searchApi.searchTouch).toHaveBeenCalledWith({ scope: 'city', id: 9 });
  });

  it('Esc in the form leaves the field as it was: the typed text, with its list open again', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(addRow(language, 'city', 'zzz')).toBeInTheDocument();
    });
    await user.click(theRow(language, 'city', 'zzz'));
    const request = host.mock.calls[0]?.[0] as AddRecordRequest;
    act(() => {
      input.blur();
      request.onClosed(null);
    });
    await waitFor(() => {
      expect(input).toHaveFocus();
    });
    expect(input.value).toBe('zzz');
    expect(input).toHaveAttribute('aria-expanded', 'true');
    expect(await screen.findByText(message(language, 'search-no-match'))).toBeInTheDocument();
  });

  it('↓ on the last match moves to the row, Enter adds, ↑ goes back to the match', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    // A click on the field highlights none; ↓ walks down the matches to the last one.
    await user.keyboard('{ArrowDown}{ArrowDown}{ArrowDown}');
    expect(options()[2]).toHaveAttribute('aria-selected', 'true');
    await user.keyboard('{ArrowDown}');
    const row = theRow(language, 'city');
    expect(row).toHaveAttribute('aria-current', 'true');
    expect(input).toHaveAttribute('aria-activedescendant', row.id);
    expect(options().some((option) => option.getAttribute('aria-selected') === 'true')).toBe(false);
    await user.keyboard('{ArrowUp}');
    expect(options()[2]).toHaveAttribute('aria-selected', 'true');
    expect(row).toHaveAttribute('aria-current', 'false');
    await user.keyboard('{ArrowDown}{Enter}');
    expect(host).toHaveBeenCalledWith(expect.objectContaining({ kind: 'city', text: '' }));
  });

  it('on an empty list ↓ then Enter adds with the typed text', async () => {
    fakeSearch('city', 0);
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await user.keyboard('1x');
    await waitFor(() => {
      expect(addRow(language, 'city', '1x')).toBeInTheDocument();
    });
    await user.keyboard('{ArrowDown}{Enter}');
    expect(host).toHaveBeenCalledWith(expect.objectContaining({ kind: 'city', text: '1x' }));
  });

  it('Enter on text that matches nothing adds it; Enter on a number does not (a code)', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    await user.keyboard('{Enter}');
    expect(host).toHaveBeenCalledTimes(1);
    expect(host).toHaveBeenLastCalledWith(expect.objectContaining({ text: 'zzz' }));
    // A number that matches nothing is a code, not a name to add.
    act(() => {
      (host.mock.calls[0]?.[0] as AddRecordRequest).onClosed(null);
    });
    await waitFor(() => {
      expect(input).toHaveFocus();
    });
    await user.keyboard('{Control>}a{/Control}77');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    await user.keyboard('{Enter}');
    expect(host).toHaveBeenCalledTimes(1);
  });

  it('F4 adds with the typed text', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await user.keyboard('abc');
    pressF4(input);
    expect(host).toHaveBeenCalledWith(expect.objectContaining({ kind: 'city', text: 'abc' }));
  });

  it('a filter box has no row, and F4 does nothing there', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host, canAdd: false });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    expect(addRow(language, 'city', 'zzz')).not.toBeInTheDocument();
    pressF4(input);
    expect(host).not.toHaveBeenCalled();
  });

  it('a filter box with an empty list says the plain words, not those of a kind', async () => {
    fakeSearch('city', 0);
    const { input, user } = setup(language, { host: vi.fn(), canAdd: false });
    await user.click(input);
    expect(await screen.findByText(message(language, 'search-empty'))).toBeInTheDocument();
    expect(addRow(language, 'city')).not.toBeInTheDocument();
  });

  it('without the app’s forms there is no row either', async () => {
    const { input, user } = setup(language, { host: null });
    await user.click(input);
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    expect(addRow(language, 'city')).not.toBeInTheDocument();
  });

  it('a مارکہ added from a lot carries the lot’s seller as its owner', async () => {
    fakeSearch('marka', 0);
    const host = vi.fn();
    const owner = { id: 7, code: '030110001', name_ur: 'رحیم', name_en: 'Rahim' };
    const { input, user } = setup(language, { scope: 'marka', host, addOwner: owner });
    await user.click(input);
    await user.keyboard('AR');
    await waitFor(() => {
      expect(addRow(language, 'marka', 'AR')).toBeInTheDocument();
    });
    await user.click(theRow(language, 'marka', 'AR'));
    expect(host).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'marka', text: 'AR', owner }),
    );
  });

  it('a language switch with the form open keeps what was typed', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(addRow(language, 'city', 'zzz')).toBeInTheDocument();
    });
    await user.click(theRow(language, 'city', 'zzz'));
    const request = host.mock.calls[0]?.[0] as AddRecordRequest;
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      input.blur();
      useLanguageStore.setState({ language: other });
    });
    expect(input.value).toBe('zzz');
    act(() => {
      request.onClosed(null);
    });
    await waitFor(() => {
      expect(input.value).toBe('zzz');
    });
    expect(input).toHaveAttribute('aria-expanded', 'true');
  });
});
