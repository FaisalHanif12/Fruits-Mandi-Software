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
    sold_by_weight: false,
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
  onValue?: (value: PickerValue | null) => void;
}

function Harness({ initial = null, onValue }: HarnessProps) {
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
      offset: 0,
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
    // The list kept from the click has the same three rows: wait for the answer to the text.
    await waitFor(() => {
      expect(searchApi.searchMasters).toHaveBeenLastCalledWith(
        expect.objectContaining({ text: 'a' }),
      );
    });
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    await act(async () => {
      await Promise.resolve();
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
    // Tab into the field opens its list at once (PRD FR-7.16, plan 46); Esc closes it, and then ↑ ↓
    // move between fields again. jsdom has no layout: give the three fields the boxes of a column,
    // one under the other.
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
    expect(input).toHaveAttribute('aria-expanded', 'true');
    await user.keyboard('{Escape}');
    expect(input).toHaveAttribute('aria-expanded', 'false');
    await user.keyboard('{ArrowDown}');
    expect(screen.getByTestId('after')).toHaveFocus();
    await user.keyboard('{ArrowUp}');
    expect(input).toHaveFocus();
    // The focus came back to the field, so its list is open again: Esc closes it, then ↑ moves.
    expect(input).toHaveAttribute('aria-expanded', 'true');
    await user.keyboard('{Escape}');
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
    // Empty text is the whole list, the last chosen first, then by code (plan 32, R7).
    expect(searchApi.searchMasters).toHaveBeenCalledWith({
      scope: 'city',
      text: '',
      offset: 0,
      limit: 50,
    });
    // The inactive one is still not offered.
    expect(options()).toHaveLength(3);
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

/** jsdom has no layout: give fields the boxes of a row, one beside the other. */
function layInRow(fields: HTMLElement[]) {
  fields.forEach((field, column) => {
    field.getBoundingClientRect = () =>
      ({
        left: column * 120,
        right: column * 120 + 100,
        top: 0,
        bottom: 40,
        x: column * 120,
        y: 0,
        width: 100,
        height: 40,
        toJSON: () => ({}),
      }) satisfies DOMRect;
  });
}

// Plan 46, PRD FR-7.16 of v1.14: the list opens as soon as the field has the focus.
describe.each(LANGUAGES)('Picker, opened by the focus (%s)', (language) => {
  it('opens on a Tab into the field and again each time the focus comes back; Tab leaves and closes it', async () => {
    const { input, user } = setup(language);
    await user.tab();
    await user.tab();
    expect(input).toHaveFocus();
    expect(input).toHaveAttribute('aria-expanded', 'true');
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(0);
    });
    await user.tab();
    expect(screen.getByTestId('after')).toHaveFocus();
    expect(input).toHaveAttribute('aria-expanded', 'false');
    expect(options()).toHaveLength(0);
    await user.tab({ shift: true });
    expect(input).toHaveFocus();
    expect(input).toHaveAttribute('aria-expanded', 'true');
  });

  it('highlights the record the field has; Enter keeps it and moves on to the next field', async () => {
    const onValue = vi.fn();
    const { input, user } = setup(language, { initial: ARSHAD, onValue });
    await user.tab();
    await user.tab();
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    await waitFor(() => {
      expect(
        options().filter((option) => option.getAttribute('aria-selected') === 'true'),
      ).toHaveLength(1);
    });
    expect(
      visible(options().find((o) => o.getAttribute('aria-selected') === 'true')?.textContent),
    ).toContain(language === 'ur' ? ARSHAD.name_ur : ARSHAD.name_en);
    await user.keyboard('{Enter}');
    expect(screen.getByTestId('after')).toHaveFocus();
    // It was not swapped for the first row, and nothing was asked to change.
    expect(onValue).not.toHaveBeenCalled();
    expect(input.value).toBe(language === 'ur' ? ARSHAD.name_ur : ARSHAD.name_en);
  });

  it('a record that is not among the rows read is kept by Enter too: nothing is highlighted', async () => {
    const onValue = vi.fn();
    const missing: PickerValue = { id: 99, code: '0099', name_ur: 'گمشدہ', name_en: 'Missing' };
    const { user } = setup(language, { initial: missing, onValue });
    await user.tab();
    await user.tab();
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    expect(
      options().filter((option) => option.getAttribute('aria-selected') === 'true'),
    ).toHaveLength(0);
    await user.keyboard('{Enter}');
    expect(screen.getByTestId('after')).toHaveFocus();
    expect(onValue).not.toHaveBeenCalled();
  });

  it('Enter before the list has answered keeps the record and moves on; it never picks a row unseen', async () => {
    vi.mocked(searchApi.searchMasters).mockReturnValue(new Promise(() => undefined));
    vi.mocked(searchApi.searchRecent).mockReturnValue(new Promise(() => undefined));
    const onValue = vi.fn();
    const { user } = setup(language, { initial: ARSHAD, onValue });
    await user.tab();
    await user.tab();
    await user.keyboard('{Enter}');
    expect(screen.getByTestId('after')).toHaveFocus();
    expect(onValue).not.toHaveBeenCalled();
  });

  it('an empty field highlights nothing: Enter leaves it empty and moves on; ↓ then Enter picks', async () => {
    const onValue = vi.fn();
    const { user } = setup(language, { onValue });
    await user.tab();
    await user.tab();
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    expect(
      options().filter((option) => option.getAttribute('aria-selected') === 'true'),
    ).toHaveLength(0);
    // An optional field is skipped with Enter and stays empty (the اڈا of a بکری).
    await user.keyboard('{Enter}');
    expect(screen.getByTestId('after')).toHaveFocus();
    expect(onValue).not.toHaveBeenCalled();
    // Back in the field, ↓ points at the first row and the next ↓ at the second; Enter picks it.
    await user.tab({ shift: true });
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    await user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    expect(onValue).toHaveBeenCalledWith(expect.objectContaining({ id: ARSHAD.id }));
    expect(screen.getByTestId('after')).toHaveFocus();
  });

  it('← and → leave the open list for the field on that side, like in any text field', async () => {
    const { input, user } = setup(language);
    layInRow([screen.getByTestId('before'), input, screen.getByTestId('after')]);
    await user.tab();
    await user.tab();
    expect(input).toHaveAttribute('aria-expanded', 'true');
    await user.keyboard('{ArrowRight}');
    expect(screen.getByTestId('after')).toHaveFocus();
    expect(options()).toHaveLength(0);
    await user.keyboard('{ArrowLeft}');
    expect(input).toHaveFocus();
    expect(input).toHaveAttribute('aria-expanded', 'true');
    await user.keyboard('{ArrowLeft}');
    expect(screen.getByTestId('before')).toHaveFocus();
  });

  it('once he has typed, ← and → move the cursor inside the text first', async () => {
    const { input, user } = setup(language);
    layInRow([screen.getByTestId('before'), input, screen.getByTestId('after')]);
    await user.tab();
    await user.tab();
    await user.keyboard('ar');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    // The arrow toward the start of the text: ← in English, → in Urdu (the text runs the other way).
    await user.keyboard(language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}');
    // The cursor moved inside "ar"; the focus stayed and the list is still open.
    expect(input).toHaveFocus();
    expect(input).toHaveAttribute('aria-expanded', 'true');
  });

  it('Esc closes the list and then ↓ is a key of the form again; the chosen name stays', async () => {
    const { input, user } = setup(language, { initial: AKRAM });
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
    expect(input).toHaveAttribute('aria-expanded', 'true');
    await user.keyboard('{ArrowDown}');
    // Open: ↓ chose a row, the focus did not move.
    expect(input).toHaveFocus();
    await user.keyboard('{Escape}');
    expect(input).toHaveAttribute('aria-expanded', 'false');
    await user.keyboard('{ArrowDown}');
    expect(screen.getByTestId('after')).toHaveFocus();
    expect(input.value).toBe(language === 'ur' ? AKRAM.name_ur : AKRAM.name_en);
  });

  it('a click opens it at once, and the language can change with the list open', async () => {
    const { input, user } = setup(language, { initial: AKRAM });
    await user.click(input);
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    // The list stays open and the same rows read in the other language; the value is kept.
    expect(input).toHaveAttribute('aria-expanded', 'true');
    await waitFor(() => {
      expect(
        optionTexts().some((text) => text.includes(other === 'ur' ? AKRAM.name_ur : AKRAM.name_en)),
      ).toBe(true);
    });
    expect(input.value).toBe(other === 'ur' ? AKRAM.name_ur : AKRAM.name_en);
  });
});
