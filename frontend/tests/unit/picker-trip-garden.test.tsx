// Plan 34, R1 to R3: a trip is found by its code of type 007 (0070, 0071 ...) and a garden by its
// code of type 008 (0080 ...), in the shared picker with the search scopes `trip` and `garden`.
// The list is every open one, the code shows in each row with the market as its second line, a
// short code is the running number (`1` finds 0071), and there is no "add" button: a trip and a
// garden are made on their own screens. Both languages, against a fake of the search that answers
// as the backend does.

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
  type PickerValue,
} from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/search/api');

type VentureScope = Extract<SearchScope, 'trip' | 'garden'>;

/** The open ones of each kind: two trips (0070, 0071) and two gardens (0080, 0081). */
function ventures(scope: VentureScope): SearchHit[] {
  const type = scope === 'trip' ? '007' : '008';
  return [0, 1].map((running) => ({
    id: running + 1,
    kind: scope,
    code: `${type}${String(running)}`,
    // A trip is named by its code and its day; a garden by the name that was typed.
    name_ur: scope === 'trip' ? `${type}${String(running)} 10-10-2026` : `باغ ${String(running)}`,
    name_en:
      scope === 'trip' ? `${type}${String(running)} 10-10-2026` : `Garden ${String(running)}`,
    extra_ur: 'لاہور منڈی',
    extra_en: 'Lahore Mandi',
    is_active: true,
    sold_by_weight: false,
  }));
}

/** The search as the backend answers it: all for empty text, the running number first, else a match. */
function fakeSearch(scope: VentureScope) {
  const all = ventures(scope);
  vi.mocked(searchApi.searchMasters).mockImplementation(({ text }) => {
    const wanted = text.trim();
    if (wanted === '') return Promise.resolve(all);
    const byRunning = all.filter((row) => row.code.slice(3) === wanted);
    if (/^(0|[1-9]\d*)$/.test(wanted) && byRunning.length > 0) return Promise.resolve(byRunning);
    return Promise.resolve(all.filter((row) => row.code.includes(wanted)));
  });
  vi.mocked(searchApi.searchRecent).mockResolvedValue(all);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}

function Harness({ scope, host }: { scope: VentureScope; host: AddRecord | null }) {
  const [value, setValue] = useState<PickerValue | null>(null);
  useEscapeDispatcher();
  return (
    <AddRecordContext.Provider value={host}>
      <form {...formNavigationProps}>
        <Field label={message('en', 'common-back')}>
          {(control) => (
            <Picker
              {...control}
              scope={scope}
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
      </form>
    </AddRecordContext.Provider>
  );
}

function setup(language: Language, scope: VentureScope, host: AddRecord | null = null) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Harness scope={scope} host={host} />
    </QueryClientProvider>,
  );
  return { input: screen.getByRole<HTMLInputElement>('combobox'), user: userEvent.setup() };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe.each(LANGUAGES)('the picker of trips and gardens (%s)', (language) => {
  it.each([
    ['trip', ['0070', '0071']],
    ['garden', ['0080', '0081']],
  ] as const)(
    'a click on the %s field lists the open ones by code, with the market as the second line',
    async (scope, codes) => {
      fakeSearch(scope);
      const { input, user } = setup(language, scope);
      await user.click(input);
      await waitFor(() => {
        expect(screen.getAllByRole('option')).toHaveLength(2);
      });
      const rows = screen.getAllByRole('option').map((row) => visible(row.textContent));
      codes.forEach((code, index) => {
        expect(rows[index]).toContain(code);
        expect(rows[index]).toContain(language === 'ur' ? 'لاہور منڈی' : 'Lahore Mandi');
      });
      expect(searchApi.searchMasters).toHaveBeenCalledWith(expect.objectContaining({ scope }));
    },
  );

  it.each([
    ['trip', '1', '0071', '0071'],
    ['garden', '0', '0080', language === 'ur' ? 'باغ 0' : 'Garden 0'],
  ] as const)(
    'a short code in the %s field is the running number: %s picks %s with Enter',
    async (scope, typed, code, shown) => {
      fakeSearch(scope);
      const { input, user } = setup(language, scope);
      await user.click(input);
      await user.keyboard(typed);
      await waitFor(() => {
        expect(screen.getAllByRole('option')).toHaveLength(1);
      });
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(input.value).toContain(shown);
      });
      expect(searchApi.searchTouch).toHaveBeenCalledWith(
        expect.objectContaining({ scope, id: code === '0071' ? 2 : 1 }),
      );
    },
  );

  it.each(['trip', 'garden'] as const)(
    'a %s that is not there offers no "add" button, only the words that nothing matches',
    async (scope) => {
      fakeSearch(scope);
      const host = vi.fn();
      const { input, user } = setup(language, scope, host);
      await user.click(input);
      await user.keyboard('zzz');
      expect(await screen.findByText(message(language, 'search-no-match'))).toBeInTheDocument();
      // The list ends with no add row (the id of the row ends with -add).
      expect(document.querySelector('[id$="-add"]')).toBeNull();
      expect(host).not.toHaveBeenCalled();
    },
  );
});
