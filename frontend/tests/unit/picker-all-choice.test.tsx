// The "All" choice of a filter picker (PRD FR-19.1, plan 79 item 1): a filter starts at سب / All, shows it
// as its value, the open list starts with the "All" row, and picking a record then "All" again empties the
// filter, so the page shows every record. A picker without `allChoice` has no such row.

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
import { message, renderIn } from '../helpers/render';

vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

function hit(id: number, code: string, ur: string, en: string): SearchHit {
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

const AKRAM = hit(1, '0001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
const ARSHAD = hit(2, '0002', 'ارشد ملتانی', 'Arshad Multani');

function Harness({
  allChoice,
  onValue,
}: {
  allChoice: boolean;
  onValue: (v: PickerValue | null) => void;
}) {
  const [value, setValue] = useState<PickerValue | null>(null);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <Picker
            {...control}
            scope="city"
            canAdd={false}
            {...(allChoice ? { allChoice: true } : {})}
            value={value}
            onChange={(next) => {
              const chosen =
                next === null
                  ? null
                  : { id: next.id, code: next.code, name_ur: next.name_ur, name_en: next.name_en };
              setValue(chosen);
              onValue(chosen);
            }}
          />
        )}
      </Field>
    </form>
  );
}

function setup(language: Language, allChoice: boolean) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  const onValue = vi.fn();
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  renderIn(
    language,
    <QueryClientProvider client={client}>
      <Harness allChoice={allChoice} onValue={onValue} />
    </QueryClientProvider>,
  );
  return {
    input: screen.getByRole<HTMLInputElement>('combobox'),
    user: userEvent.setup(),
    onValue,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(searchApi.searchMasters).mockResolvedValue([AKRAM, ARSHAD]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([AKRAM, ARSHAD]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

describe.each(LANGUAGES)('a filter picker with the All choice (%s)', (language) => {
  it('shows All as its value and starts its list with the All row', async () => {
    const { input, user } = setup(language, true);
    const all = message(language, 'search-all-choice');
    expect(input).toHaveAttribute('placeholder', all);
    await user.click(input);
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(2);
    });
    expect(screen.getByRole('button', { name: all })).toBeInTheDocument();
  });

  it('a record narrows the filter, and the All row brings every record back', async () => {
    const { input, user, onValue } = setup(language, true);
    const all = message(language, 'search-all-choice');
    await user.click(input);
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(2);
    });
    await user.click(await screen.findByRole('option', { name: /Akram|اکرم/ }));
    expect(onValue).toHaveBeenLastCalledWith(expect.objectContaining({ id: 1 }));
    // Picking the All row empties the filter again.
    await user.click(input);
    await user.click(await screen.findByRole('button', { name: all }));
    expect(onValue).toHaveBeenLastCalledWith(null);
    expect(input).toHaveAttribute('placeholder', all);
  });

  it('↑ from the first match reaches the All row and Enter takes it', async () => {
    const { input, user, onValue } = setup(language, true);
    await user.click(input);
    await user.keyboard('{ArrowDown}');
    await user.keyboard('{Enter}');
    expect(onValue).toHaveBeenLastCalledWith(expect.objectContaining({ id: 1 }));
    await user.click(input);
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(2);
    });
    await user.keyboard('{ArrowUp}{Enter}');
    expect(onValue).toHaveBeenLastCalledWith(null);
  });

  it('a picker without the All choice has no such row', async () => {
    const { input, user } = setup(language, false);
    await user.click(input);
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(2);
    });
    expect(
      screen.queryByRole('button', { name: message(language, 'search-all-choice') }),
    ).not.toBeInTheDocument();
  });
});
