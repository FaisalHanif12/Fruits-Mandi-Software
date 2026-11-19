// Plan 78 D2, the shared MultiPicker (PRD 12.11, Umar 17-11-2026): one field for several records of a master list,
// used for the items of a garden. The chosen ones are listed above the picker, each with a button that removes it;
// the picker itself starts empty after every choice and is the same picker as everywhere (a click lists the records,
// typing filters them). A record that is listed already is not added twice. Both languages, against a fake of the
// search that answers as the backend does.

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
import { Field, MultiPicker, type PickerValue } from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/search/api');

const MANGO = { id: 61, code: '0040', name_ur: 'آم', name_en: 'Mango' };
const ORANGE = { id: 62, code: '0041', name_ur: 'مالٹا', name_en: 'Orange' };
const ITEMS = [MANGO, ORANGE];

const hitOf = (item: (typeof ITEMS)[number]): SearchHit => ({
  id: item.id,
  kind: 'item',
  code: item.code,
  name_ur: item.name_ur,
  name_en: item.name_en,
  extra_ur: '',
  extra_en: '',
  is_active: true,
  sold_by_weight: false,
});

/** The search as the backend answers it: every item for empty text, else the ones whose name or code has the text. */
function fakeSearch() {
  vi.mocked(searchApi.searchMasters).mockImplementation(({ text }) => {
    const wanted = text.trim().toLowerCase();
    const found = ITEMS.filter(
      (item) =>
        wanted === '' ||
        item.name_en.toLowerCase().includes(wanted) ||
        item.name_ur.includes(wanted) ||
        item.code.includes(wanted),
    );
    return Promise.resolve(found.map(hitOf));
  });
  vi.mocked(searchApi.searchRecent).mockResolvedValue(ITEMS.map(hitOf));
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
}

/** What the screen keeps: the records chosen so far. It is shown under the field so the test can read it. */
function Harness({ start }: { start: PickerValue[] }) {
  const [values, setValues] = useState<PickerValue[]>(start);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <MultiPicker
            {...control}
            scope="item"
            values={values}
            valueOf={(hit) => ({
              id: hit.id,
              code: hit.code,
              name_ur: hit.name_ur,
              name_en: hit.name_en,
            })}
            onChange={setValues}
          />
        )}
      </Field>
      <output data-testid="ids">{values.map((value) => value.id).join(',')}</output>
    </form>
  );
}

function setup(language: Language, start: PickerValue[] = []) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Harness start={start} />
    </QueryClientProvider>,
  );
  return { user: userEvent.setup() };
}

const ids = () => screen.getByTestId('ids').textContent;

beforeEach(() => {
  vi.clearAllMocks();
  fakeSearch();
});

describe.each(LANGUAGES)('the multi picker (%s)', (language) => {
  const nameOf = (item: (typeof ITEMS)[number]) =>
    language === 'ur' ? item.name_ur : item.name_en;
  const removeButton = (item: (typeof ITEMS)[number]) =>
    screen.getByRole('button', {
      name: (name) =>
        visible(name) === message(language, 'ui-multi-remove', { name: nameOf(item) }),
    });

  it('R1: it starts with no chosen record, and says in words that more can be added', () => {
    setup(language);
    expect(ids()).toBe('');
    expect(screen.queryByRole('list')).toBeNull();
    expect(
      screen.getByText((text) => visible(text) === message(language, 'ui-multi-hint')),
    ).toBeInTheDocument();
  });

  it('R2: a pick lists the record above the picker, and the picker is empty again for the next one', async () => {
    const { user } = setup(language);
    const input = screen.getByRole<HTMLInputElement>('combobox');
    await user.click(input);
    await user.click(
      await screen.findByRole('option', { name: (name) => visible(name).includes(nameOf(MANGO)) }),
    );
    await waitFor(() => {
      expect(ids()).toBe('61');
    });
    expect(visible(screen.getByRole('list').textContent)).toContain(nameOf(MANGO));
    expect(input.value).toBe('');
    // The next record is picked from the same field, and both are listed, in the order they were chosen.
    await user.click(input);
    await user.click(
      await screen.findByRole('option', { name: (name) => visible(name).includes(nameOf(ORANGE)) }),
    );
    await waitFor(() => {
      expect(ids()).toBe('61,62');
    });
    const listed = screen.getAllByRole('listitem').map((item) => visible(item.textContent));
    expect(listed).toHaveLength(2);
    expect(listed[0]).toContain(nameOf(MANGO));
    expect(listed[1]).toContain(nameOf(ORANGE));
  });

  it('R3: a record that is listed already is not added twice', async () => {
    const { user } = setup(language, [MANGO]);
    const input = screen.getByRole<HTMLInputElement>('combobox');
    await user.click(input);
    await user.click(
      await screen.findByRole('option', { name: (name) => visible(name).includes(nameOf(MANGO)) }),
    );
    await waitFor(() => {
      expect(input.value).toBe('');
    });
    expect(ids()).toBe('61');
    expect(screen.getAllByRole('listitem')).toHaveLength(1);
  });

  it('R4: the button of a record removes it, and only it', async () => {
    const { user } = setup(language, [MANGO, ORANGE]);
    await user.click(removeButton(MANGO));
    await waitFor(() => {
      expect(ids()).toBe('62');
    });
    expect(screen.getAllByRole('listitem')).toHaveLength(1);
    await user.click(removeButton(ORANGE));
    await waitFor(() => {
      expect(ids()).toBe('');
    });
    expect(screen.queryByRole('list')).toBeNull();
  });

  it('R5: the keyboard picks too: type, Enter on the match, and the picker is empty for the next', async () => {
    const { user } = setup(language);
    const input = screen.getByRole<HTMLInputElement>('combobox');
    await user.click(input);
    await user.keyboard(language === 'ur' ? 'مالٹا' : 'Oran');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(ids()).toBe('62');
    });
    expect(input.value).toBe('');
  });

  it('R6: a language switch keeps the chosen records and shows their names in the other language', async () => {
    setup(language, [MANGO, ORANGE]);
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    await waitFor(() => {
      const listed = screen.getAllByRole('listitem').map((item) => visible(item.textContent));
      expect(listed[0]).toContain(other === 'ur' ? MANGO.name_ur : MANGO.name_en);
      expect(listed[1]).toContain(other === 'ur' ? ORANGE.name_ur : ORANGE.name_en);
    });
    expect(ids()).toBe('61,62');
  });
});
