// Plan 32, R7 and R8 on the shared master picker: a mouse click opens the list of every record of
// the kind (recent first, then by code), a long list is read page by page, arriving by keyboard
// does not open it (PRD 7.2 stays), and an empty list offers a button that asks the app for the
// Index form of that kind with the typed text. Both languages, against a fake of the search.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import type { SearchHit } from '../../src/shared/search';
import {
  AddRecordContext,
  Field,
  Picker,
  type AddRecord,
  type PickerValue,
} from '../../src/shared/ui';
import { message, renderIn } from '../helpers/render';

vi.mock('../../src/shared/search/api');

function hit(id: number): SearchHit {
  return {
    id,
    kind: 'city',
    code: String(id).padStart(4, '0'),
    name_ur: `شہر ${String(id)}`,
    name_en: `City ${String(id)}`,
    extra_ur: '',
    extra_en: '',
    is_active: true,
    sold_by_weight: false,
  };
}

/** The fake search: every record by code for empty text, a match on the text otherwise, by page. */
function fakeSearch(total: number) {
  const all = Array.from({ length: total }, (_, index) => hit(index + 1));
  vi.mocked(searchApi.searchMasters).mockImplementation(({ text, offset = 0, limit }) => {
    const wanted = text.trim().toLowerCase();
    const found = all.filter(
      (row) =>
        wanted === '' || row.code.includes(wanted) || row.name_en.toLowerCase().includes(wanted),
    );
    return Promise.resolve(found.slice(offset, offset + limit));
  });
  vi.mocked(searchApi.searchRecent).mockResolvedValue(all);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}

interface HarnessProps {
  host?: AddRecord | null;
  showAdd?: boolean;
  readOnly?: boolean;
  initial?: PickerValue | null;
}

function Harness({ host = null, showAdd, readOnly, initial = null }: HarnessProps) {
  const [value, setValue] = useState<PickerValue | null>(initial);
  useEscapeDispatcher();
  return (
    <AddRecordContext.Provider value={host}>
      <form {...formNavigationProps}>
        <input data-testid="before" />
        <Field
          label={message('en', 'common-back')}
          {...(readOnly === undefined ? {} : { readOnly })}
        >
          {(control) => (
            <Picker
              {...control}
              scope="city"
              value={value}
              canAdd={showAdd}
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

beforeEach(() => {
  vi.clearAllMocks();
  fakeSearch(3);
});

describe.each(LANGUAGES)('Picker opened by a click (%s)', (language) => {
  it('a mouse click lists every record of the kind, by code, without typing a letter', async () => {
    const { input, user } = setup(language);
    expect(input).toHaveAttribute('aria-expanded', 'false');
    await user.click(input);
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    expect(input).toHaveAttribute('aria-expanded', 'true');
    expect(searchApi.searchMasters).toHaveBeenCalledWith({
      scope: 'city',
      text: '',
      offset: 0,
      limit: 50,
    });
  });

  it('a second click on the open field keeps the list open', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    await user.click(input);
    expect(input).toHaveAttribute('aria-expanded', 'true');
    expect(options()).toHaveLength(3);
  });

  it('typing narrows the list by code or by name', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('0002');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.clear(input);
    await user.keyboard(language === 'ur' ? 'City 3' : 'city 3');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
  });

  it('arriving by keyboard does not open the list; typing does (PRD 7.2)', async () => {
    const { input, user } = setup(language);
    screen.getByTestId('before').focus();
    await user.tab();
    expect(input).toHaveFocus();
    expect(input).toHaveAttribute('aria-expanded', 'false');
    expect(options()).toHaveLength(0);
    expect(searchApi.searchMasters).not.toHaveBeenCalled();
    await user.keyboard('0');
    await waitFor(() => {
      expect(input).toHaveAttribute('aria-expanded', 'true');
    });
  });

  it('a field that cannot be changed opens no list when it is clicked', async () => {
    const { input, user } = setup(language, { readOnly: true });
    await user.click(input);
    expect(input).toHaveAttribute('aria-expanded', 'false');
    expect(searchApi.searchMasters).not.toHaveBeenCalled();
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });

  it('a click on a field that shows a chosen name selects the whole name, so typing replaces it', async () => {
    const { input, user } = setup(language, {
      initial: { id: 2, code: '0002', name_ur: 'شہر 2', name_en: 'City 2' },
    });
    await user.click(input);
    expect(input.selectionStart).toBe(0);
    expect(input.selectionEnd).toBe(input.value.length);
    expect(input.value).not.toBe('');
  });

  it('Alt+ArrowDown still opens the full list from the keyboard', async () => {
    const { input, user } = setup(language);
    input.focus();
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
  });
});

describe('Picker, a long list is read page by page', () => {
  it('asks for the next page near the end of what is read, and stops at a short page', async () => {
    fakeSearch(120);
    const { input, user } = setup('en');
    await user.click(input);
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(0);
    });
    expect(searchApi.searchMasters).toHaveBeenCalledTimes(1);
    const list = screen.getByRole('listbox');
    fireEvent.scroll(list, { target: { scrollTop: 2600 } });
    await waitFor(() => {
      expect(searchApi.searchMasters).toHaveBeenCalledWith(
        expect.objectContaining({ text: '', offset: 50, limit: 50 }),
      );
    });
    fireEvent.scroll(list, { target: { scrollTop: 6200 } });
    await waitFor(() => {
      expect(searchApi.searchMasters).toHaveBeenCalledWith(
        expect.objectContaining({ text: '', offset: 100, limit: 50 }),
      );
    });
    // The last page held 20 records: that is the end, so nothing more is asked.
    fireEvent.scroll(list, { target: { scrollTop: 8600 } });
    await act(async () => {
      await Promise.resolve();
    });
    expect(searchApi.searchMasters).toHaveBeenCalledTimes(3);
  });
});

describe.each(LANGUAGES)('Picker, nothing found (%s)', (language) => {
  it('says nothing has been added yet and offers the button when the list is empty', async () => {
    fakeSearch(0);
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-empty'))).toBeInTheDocument();
    });
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'search-add-city')) }),
    );
    expect(host).toHaveBeenCalledWith(expect.objectContaining({ kind: 'city', text: '' }));
  });

  it('asks the app for the Index form with the typed text, and takes the field back when it closes', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'search-add-city')) }),
    );
    expect(host).toHaveBeenCalledTimes(1);
    const request = host.mock.calls[0]?.[0] as { kind: string; text: string; onClosed: () => void };
    expect(request.kind).toBe('city');
    expect(request.text).toBe('zzz');
    // The form was opened above the screen: the field loses focus to it. Nothing typed is dropped.
    act(() => {
      input.blur();
    });
    expect(input.value).toBe('zzz');
    // The form closed: the field has the same text, the focus, and its list is open again.
    act(() => {
      request.onClosed();
    });
    await waitFor(() => {
      expect(input).toHaveFocus();
    });
    expect(input.value).toBe('zzz');
    expect(input).toHaveAttribute('aria-expanded', 'true');
  });

  it('without the app Index form the empty list has no button; a filter can turn it off', async () => {
    const { input, user } = setup(language, { host: null });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    expect(
      screen.queryByRole('button', { name: new RegExp(message(language, 'search-add-city')) }),
    ).not.toBeInTheDocument();
  });

  it('showAdd false hides the button even when the app has the Index form', async () => {
    const host = vi.fn();
    const { input, user } = setup(language, { host, showAdd: false });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    expect(
      screen.queryByRole('button', { name: new RegExp(message(language, 'search-add-city')) }),
    ).not.toBeInTheDocument();
  });
});
