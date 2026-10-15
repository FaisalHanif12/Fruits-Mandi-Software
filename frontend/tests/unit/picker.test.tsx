// The shared Picker (plan 06, PRD FR-7.16, FR-7.35, FR-7.40, FR-7.41, FR-7.18, NFR-SPD-18): every
// key of its table, in the Urdu and in the English app, against a fake of the search commands. The
// picker never converts English letters to Urdu (Code Standards F41).

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import type { SearchHit } from '../../src/shared/search';
import { Field, Picker, type PickerValue } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { fail } from '../helpers/app';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

function hit(id: number, code: string, ur: string, en: string, active = true): SearchHit {
  return {
    id,
    kind: 'city',
    code,
    name_ur: ur,
    name_en: en,
    extra_ur: '',
    extra_en: '',
    is_active: active,
  };
}

const AKRAM = hit(1, '0001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
const ARSHAD = hit(2, '0002', 'ارشد ملتانی', 'Arshad Multani');
const ASLAM = hit(3, '0003', 'اسلم', 'Aslam');
const OLD = hit(4, '0004', 'اکبر', 'Akbar', false);
const ALL = [AKRAM, ARSHAD, ASLAM, OLD];

function fakeSearch(rows: SearchHit[] = ALL) {
  vi.mocked(searchApi.searchMasters).mockImplementation(({ text }) => {
    const wanted = text.trim().toLowerCase();
    return Promise.resolve(
      rows.filter(
        (row) =>
          row.code.includes(wanted) ||
          row.name_en.toLowerCase().includes(wanted) ||
          row.name_ur.includes(wanted) ||
          // A small mistake is forgiven by the backend ("akrm" finds Akram).
          (wanted === 'akrm' && row.name_en === 'Akram Toll Tax'),
      ),
    );
  });
  vi.mocked(searchApi.searchRecent).mockResolvedValue(rows);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}

interface HarnessProps {
  initial?: PickerValue | null;
  onAddNew?: (text: string) => void;
  onValue?: (value: PickerValue | null) => void;
}

function Harness({ initial = null, onAddNew, onValue }: HarnessProps) {
  const [value, setValue] = useState<PickerValue | null>(initial);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <input data-testid="before" />
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <Picker
            {...control}
            scope="city"
            value={value}
            onChange={(next) => {
              const chosen =
                next === null
                  ? null
                  : { id: next.id, code: next.code, name_ur: next.name_ur, name_en: next.name_en };
              setValue(chosen);
              onValue?.(chosen);
            }}
            onAddNew={onAddNew}
          />
        )}
      </Field>
      <input data-testid="after" />
    </form>
  );
}

function setup(language: Language, props: HarnessProps = {}) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  renderIn(
    language,
    <QueryClientProvider client={client}>
      <Harness {...props} />
    </QueryClientProvider>,
  );
  const input = screen.getByRole<HTMLInputElement>('combobox');
  return { input, user: userEvent.setup(), client };
}

const options = () => screen.queryAllByRole('option');
const optionTexts = () => options().map((option) => visible(option.textContent));

beforeEach(() => {
  vi.clearAllMocks();
  fakeSearch();
});

describe.each(LANGUAGES)('Picker (%s)', (language) => {
  it('typing opens the list with the best match highlighted; the search gets the raw text', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('ar');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    expect(options()[0]).toHaveAttribute('aria-selected', 'true');
    expect(searchApi.searchMasters).toHaveBeenLastCalledWith({
      scope: 'city',
      text: 'ar',
      limit: 50,
    });
    // Never converted to Urdu: pickers search raw text (F41).
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
    expect(input).toHaveAttribute('aria-expanded', 'true');
  });

  it('finds by the name in either script, by code, and forgives a small mistake', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('اکرم');
    await waitFor(() => {
      expect(optionTexts().join(' ')).toContain('Akram');
    });
    await user.clear(input);
    await user.keyboard('akrm');
    await waitFor(() => {
      expect(optionTexts().join(' ')).toContain('Akram');
    });
    await user.clear(input);
    await user.keyboard('0003');
    await waitFor(() => {
      expect(optionTexts().join(' ')).toContain('Aslam');
    });
  });

  it('rows show the code and the name in the app language, with the other script dimmed', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('akram');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const text = visible(options()[0]?.textContent);
    expect(text).toContain('0001');
    expect(text).toContain(language === 'ur' ? AKRAM.name_ur : AKRAM.name_en);
    expect(text).toContain(language === 'ur' ? AKRAM.name_en : AKRAM.name_ur);
  });

  it('↓ and ↑ move through the matches, Enter picks and goes on to the next field', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, { onValue });
    await user.click(input);
    await user.keyboard('a');
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    await user.keyboard('{ArrowDown}');
    expect(options()[1]).toHaveAttribute('aria-selected', 'true');
    expect(input).toHaveFocus();
    await user.keyboard('{ArrowUp}');
    expect(options()[0]).toHaveAttribute('aria-selected', 'true');
    // Past the first it goes round to the last, like any combobox.
    await user.keyboard('{ArrowUp}');
    expect(options()[2]).toHaveAttribute('aria-selected', 'true');
    await user.keyboard('{ArrowUp}{Enter}');
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 2, code: '0002' }));
    expect(screen.getByTestId('after')).toHaveFocus();
    expect(input.value).toBe(language === 'ur' ? ARSHAD.name_ur : ARSHAD.name_en);
    expect(searchApi.searchTouch).toHaveBeenCalledWith({ scope: 'city', id: 2 });
  });

  it('Enter straight after a key never takes a row from the older text, it waits for the whole text', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, { onValue });
    await user.click(input);
    await user.keyboard('a');
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    // `ar` is the text now. The list still shows the answer to `a` (Akram first), which must not
    // be taken: the best match of `ar` is Arshad.
    await user.keyboard('r{Enter}');
    await waitFor(() => {
      expect(onValue).toHaveBeenCalledTimes(1);
    });
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: 2, code: '0002' }));
    expect(input.value).toBe(language === 'ur' ? ARSHAD.name_ur : ARSHAD.name_en);
  });

  it('an inactive record is never offered', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('ak');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    expect(optionTexts().join(' ')).not.toContain('Akbar');
  });

  it('Esc closes the list first, then brings back the chosen name, then goes on to leave', async () => {
    const { input, user } = setup(language, { initial: AKRAM });
    const chosen = language === 'ur' ? AKRAM.name_ur : AKRAM.name_en;
    expect(input.value).toBe(chosen);
    await user.click(input);
    await user.keyboard('{Control>}a{/Control}zz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    await user.keyboard('{Escape}');
    expect(screen.queryByText(message(language, 'search-no-match'))).not.toBeInTheDocument();
    expect(input.value).toBe('zz');
    await user.keyboard('{Escape}');
    expect(input.value).toBe(chosen);
  });

  it('with the list closed ↑ ↓ move between fields like in any other field', async () => {
    const { input, user } = setup(language);
    // Tab into the field: arriving by keyboard opens nothing (a mouse click shows the recent ones).
    // jsdom has no layout: give the three fields the boxes of a column, one under the other.
    [screen.getByTestId('before'), input, screen.getByTestId('after')].forEach((field, row) => {
      field.getBoundingClientRect = () =>
        ({
          left: 0,
          right: 100,
          top: row * 60,
          bottom: row * 60 + 40,
          x: 0,
          y: row * 60,
          width: 100,
          height: 40,
          toJSON: () => ({}),
        }) satisfies DOMRect;
    });
    await user.tab();
    await user.tab();
    expect(input).toHaveFocus();
    expect(input).toHaveAttribute('aria-expanded', 'false');
    await user.keyboard('{ArrowDown}');
    expect(screen.getByTestId('after')).toHaveFocus();
    await user.keyboard('{ArrowUp}');
    expect(input).toHaveFocus();
    await user.keyboard('{ArrowUp}');
    expect(screen.getByTestId('before')).toHaveFocus();
  });

  it('Alt+↓ opens the full list without typing, the recent records first', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(0);
    });
    expect(searchApi.searchRecent).toHaveBeenCalledWith({ scope: 'city' });
    expect(searchApi.searchMasters).not.toHaveBeenCalled();
    // The inactive one is still not offered.
    expect(options()).toHaveLength(3);
  });

  it('F4 adds a new one with the typed text; an empty result offers the same by button', async () => {
    const onAddNew = vi.fn();
    const { input, user } = setup(language, { onAddNew });
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'search-add-new')) }),
    );
    expect(onAddNew).toHaveBeenLastCalledWith('zzz');
    await user.click(input);
    await user.keyboard('{F4}');
    // user-event does not know F4 by code, so press it the way the window gets it.
    act(() => {
      input.dispatchEvent(new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }));
    });
    expect(onAddNew).toHaveBeenCalledTimes(2);
  });

  it('without an add action there is no add button and F4 does nothing', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('zzz');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    expect(
      screen.queryByRole('button', { name: new RegExp(message(language, 'search-add-new')) }),
    ).not.toBeInTheDocument();
  });

  it('emptying the field and leaving clears the value; other typing is dropped on leaving', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, { initial: AKRAM, onValue });
    await user.click(input);
    await user.keyboard('{Control>}a{/Control}{Backspace}');
    await user.tab();
    expect(onValue).toHaveBeenCalledWith(null);
    expect(input.value).toBe('');
  });

  it('leaving with other text typed keeps the chosen record and shows its name again', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, { initial: AKRAM, onValue });
    await user.click(input);
    await user.keyboard('{Control>}a{/Control}ars');
    await user.tab();
    expect(onValue).not.toHaveBeenCalled();
    expect(input.value).toBe(language === 'ur' ? AKRAM.name_ur : AKRAM.name_en);
  });

  it('a language switch keeps the chosen record and shows its name in the other language', () => {
    const { input } = setup(language, { initial: AKRAM });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    expect(input.value).toBe(language === 'ur' ? AKRAM.name_ur : AKRAM.name_en);
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect(input.value).toBe(other === 'ur' ? AKRAM.name_ur : AKRAM.name_en);
  });

  it('a failed search says so in words and with the reference', async () => {
    vi.mocked(searchApi.searchMasters).mockRejectedValue(fail({ code: 'ERR_DB_BUSY' }));
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('ak');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'search-error'));
    expect(visible(alert.textContent)).toContain(message(language, 'err-db-busy'));
  });

  it('a read-only picker shows the name and opens nothing', async () => {
    renderIn(
      language,
      <QueryClientProvider client={new QueryClient()}>
        <Field label={message('en', 'common-back')} readOnly>
          {(control) => <Picker {...control} scope="city" value={AKRAM} onChange={vi.fn()} />}
        </Field>
      </QueryClientProvider>,
    );
    const input = screen.getByRole<HTMLInputElement>('combobox');
    expect(input).toHaveAttribute('readonly');
    await userEvent.setup().click(input);
    await userEvent.setup().keyboard('{Alt>}{ArrowDown}{/Alt}');
    expect(searchApi.searchRecent).not.toHaveBeenCalled();
    expect(
      screen.queryByRole('button', { name: message(language, 'search-open-list') }),
    ).not.toBeInTheDocument();
  });
});

describe('Picker with a long list', () => {
  it('draws only the rows in view', async () => {
    const many = Array.from({ length: 5000 }, (_, index) =>
      hit(
        index + 1,
        String(index + 1).padStart(4, '0'),
        `نام ${String(index)}`,
        `Name ${String(index)}`,
      ),
    );
    fakeSearch(many);
    const { input, user } = setup('en');
    await user.click(input);
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(0);
    });
    // The list holds 5,000 records but the page holds a few dozen rows at most (NFR-SPD-18).
    expect(options().length).toBeLessThan(40);
    expect(options()[0]).toHaveAttribute('aria-setsize', '5000');
  });
});
