// Plan 64, PRD FR-7.29, FR-7.38, FR-7.69, FR-13.3, FR-13.4: the buyer box of a بکری line
// (`withChukta`) lists چکتا buyers beside ordinary buyers, each marked چکتا with his phone, no code
// and no balance; its list ends with TWO add rows, a new buyer and a new چکتا buyer; the party card
// of a highlighted or chosen چکتا buyer is his own. Every other buyer box lists none of them and has
// one add row. All of them share the id of the account چکتا کھاتہ, so a row is told by his own
// record. In the Urdu and in the English app, by keyboard, against the fakes of the commands.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { ChuktaBuyer } from '../../src/bindings/ChuktaBuyer';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import { pickedOf, type AccountHit } from '../../src/shared/accounts';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import { AccountPicker, Field, type PickerValue } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { account, balance, fakeAccountSearch } from '../helpers/accounts';
import { press } from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, renderIn, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  balance: balance(2880, 'dr'),
});
const KHATA = account(9, 'buyer', '0030', 'چکتا کھاتہ', 'Chukta Khata', { is_chukta_khata: true });
const SHAHID: ChuktaBuyer = {
  id: 1,
  account_id: 9,
  name_ur: 'شاہد',
  name_en: 'Shahid',
  phone: '03000000000',
  last_date: null,
};
const NASIR: ChuktaBuyer = {
  id: 2,
  account_id: 9,
  name_ur: 'ناصر',
  name_en: 'Nasir',
  phone: '03111111111',
  last_date: null,
};
const TARIQ: ChuktaBuyer = {
  id: 3,
  account_id: 9,
  name_ur: 'طارق',
  name_en: 'Tariq',
  phone: '03002222222',
  last_date: null,
};

const named = (text: string) => (name: string) => visible(name) === text;
const options = () => screen.queryAllByRole('option');
const nameIn = (language: Language, row: { name_ur: string; name_en: string }) =>
  language === 'ur' ? row.name_ur : row.name_en;

function Harness({
  withChukta,
  initial = null,
  onHit,
}: {
  withChukta: boolean;
  initial?: PickerValue | null;
  onHit?: (hit: AccountHit | null) => void;
}) {
  const [value, setValue] = useState<PickerValue | null>(initial);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <AccountPicker
            {...control}
            scope="buyer"
            withChukta={withChukta}
            value={value}
            onChange={(hit) => {
              setValue(hit === null ? null : pickedOf(hit));
              onHit?.(hit);
            }}
          />
        )}
      </Field>
      <input data-testid="after" />
    </form>
  );
}

function setup(
  language: Language,
  props: {
    withChukta?: boolean;
    initial?: PickerValue | null;
    onHit?: (hit: AccountHit | null) => void;
  } = {},
) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Harness withChukta={props.withChukta ?? true} {...props} />
    </QueryClientProvider>,
  );
  return { input: screen.getByRole<HTMLInputElement>('combobox'), user: userEvent.setup() };
}

/** The add rows of the open list, by the id Downshift and the picker give them. */
const addRow = () => document.querySelector<HTMLElement>('[id$="-add"]');
const extraRow = () => document.querySelector<HTMLElement>('[id$="-add-extra"]');
const activeRow = (input: HTMLElement) => input.getAttribute('aria-activedescendant') ?? '';

beforeEach(() => {
  vi.resetAllMocks();
  fakeAccountSearch([AKRAM, KHATA], {}, [SHAHID, NASIR]);
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        { toUrdu: { tariq: ['طارق'] }, toEnglish: { طارق: 'Tariq' } },
        text,
        direction,
      ),
    ),
  );
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
});

describe.each(LANGUAGES)('the buyer box of a sale line (%s)', (language) => {
  it('lists a چکتا buyer marked چکتا with his phone, and no code and no balance', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('shahid');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const row = options()[0];
    if (row === undefined) throw new Error('no row');
    const text = visible(row.textContent);
    expect(text).toContain(nameIn(language, SHAHID));
    expect(text).toContain(message(language, 'accounts-type-chukta'));
    expect(text).toContain('03000000000');
    // The shared account's code and a balance are not his.
    expect(text).not.toContain('0030');
    expect(accountApi.searchAccounts).toHaveBeenLastCalledWith(
      expect.objectContaining({ scope: 'buyer', text: 'shahid', withChukta: true }),
    );
  });

  it('keeps ordinary buyers as they were, with their code and balance', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('akram');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const text = visible(options()[0]?.textContent);
    expect(text).toContain('0020');
    expect(text).toContain('2,880');
    expect(text).not.toContain(message(language, 'accounts-type-chukta'));
  });

  it('Enter takes the person: the shared account with his own record, names and phone', async () => {
    const onHit = vi.fn();
    const { input, user } = setup(language, { onHit });
    await user.click(input);
    await user.keyboard('nasir');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    expect(onHit).toHaveBeenCalledWith(
      expect.objectContaining({
        id: 9,
        kind: 'chukta',
        chukta_id: 2,
        name_ur: 'ناصر',
        name_en: 'Nasir',
        phone: '03111111111',
      }),
    );
    expect(input.value).toBe(nameIn(language, NASIR));
    expect(screen.getByTestId('after')).toHaveFocus();
    // A چکتا buyer is not a "recent account": the shared account is not remembered for him.
    expect(accountApi.searchAccountsTouch).not.toHaveBeenCalled();
  });

  it('shows the card of the person the list highlights: his name, phone and that he pays in cash', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('shahid');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    const cash = message(language, 'assist-card-chukta-cash');
    await waitFor(() => {
      expect(
        screen.getAllByText((_text, element) => visible(element?.textContent) === cash).length,
      ).toBeGreaterThan(0);
    });
    expect(accountApi.assistPartyCard).toHaveBeenLastCalledWith(9, 1);
  });

  it('a chosen person is told from another by his own record, not by the shared id', async () => {
    const { input, user } = setup(language, {
      initial: pickedOf({
        id: 9,
        code: '0030',
        name_ur: 'ناصر',
        name_en: 'Nasir',
        kind: 'chukta',
        chukta_id: 2,
        phone: '03111111111',
      }),
    });
    await user.click(input);
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    const marked = options().filter((row) => row.getAttribute('aria-selected') === 'true');
    expect(marked).toHaveLength(1);
    expect(visible(marked[0]?.textContent)).toContain(nameIn(language, NASIR));
  });

  it('every row has its own key: two people of the shared account are two rows', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    await waitFor(() => {
      expect(options().length).toBeGreaterThanOrEqual(3);
    });
    const names = options().map((row) => visible(row.textContent));
    expect(
      names.filter((text) => text.includes(message(language, 'accounts-type-chukta'))),
    ).toHaveLength(2);
  });

  it('ends its list with two add rows: a new buyer, then a new چکتا buyer; ↓ walks to them and ↑ back', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(0);
    });
    await waitFor(() => {
      expect(addRow()).not.toBeNull();
    });
    // "tariq" is the name in both rows, and the second row says چکتا (PRD FR-7.69, FR-13.3).
    expect(visible(addRow()?.textContent)).toContain('tariq');
    expect(visible(extraRow()?.textContent)).toContain('tariq');
    expect(visible(extraRow()?.textContent)).toContain(message(language, 'accounts-type-chukta'));
    expect(visible(addRow()?.textContent)).not.toContain(message(language, 'accounts-type-chukta'));
    await user.keyboard('{ArrowDown}');
    expect(activeRow(input)).toBe(addRow()?.id);
    await user.keyboard('{ArrowDown}');
    expect(activeRow(input)).toBe(extraRow()?.id);
    // ↓ on the last stop stays there; ↑ goes back to the first add row.
    await user.keyboard('{ArrowDown}');
    expect(activeRow(input)).toBe(extraRow()?.id);
    await user.keyboard('{ArrowUp}');
    expect(activeRow(input)).toBe(addRow()?.id);
  });

  it('Enter on the second add row opens the short form of a چکتا buyer, with the typed name in it', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(0);
    });
    await user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    const dialog = await screen.findByRole('dialog', {
      name: named(message(language, 'chukta-add-title')),
    });
    expect(
      within(dialog).getByLabelText<HTMLInputElement>(
        named(message(language, 'accounts-field-name-en')),
      ).value,
    ).toBe('tariq');
  });

  it('Enter on the first add row, and F4, open the short form of an ordinary buyer, as before', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(0);
    });
    await user.keyboard('{ArrowDown}{Enter}');
    expect(
      await screen.findByRole('dialog', { name: named(message(language, 'accounts-new-title')) }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('dialog', { name: named(message(language, 'chukta-add-title')) }),
    ).toBeNull();
  });

  it('a click on the second add row opens the same form', async () => {
    const { input, user } = setup(language);
    await user.click(input);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(extraRow()).not.toBeNull();
    });
    const second = extraRow();
    if (second === null) throw new Error('no second add row');
    await user.click(second);
    expect(
      await screen.findByRole('dialog', { name: named(message(language, 'chukta-add-title')) }),
    ).toBeInTheDocument();
  });

  it('the saved person is chosen at once, as the shared account with his own record', async () => {
    vi.mocked(accountApi.chuktaAdd).mockResolvedValue(TARIQ);
    const onHit = vi.fn();
    const { input, user } = setup(language, { onHit });
    await user.click(input);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(0);
    });
    await user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    const dialog = await screen.findByRole('dialog', {
      name: named(message(language, 'chukta-add-title')),
    });
    await waitFor(() => {
      expect(
        within(dialog).getByLabelText<HTMLInputElement>(
          named(message(language, 'accounts-field-name-ur')),
        ).value,
      ).toBe('طارق');
    });
    // F10 saves (the same key as every short form).
    press('F10');
    await waitFor(() => {
      expect(accountApi.chuktaAdd).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(onHit).toHaveBeenCalledWith(
        expect.objectContaining({ id: 9, kind: 'chukta', chukta_id: 3, name_en: 'Tariq' }),
      );
    });
    await waitFor(() => {
      expect(input.value).toBe(nameIn(language, TARIQ));
    });
  });

  it('closing the short form without saving leaves the box as it was, with the typed text', async () => {
    const onHit = vi.fn();
    const { input, user } = setup(language, { onHit });
    await user.click(input);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(0);
    });
    await user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    await screen.findByRole('dialog', { name: named(message(language, 'chukta-add-title')) });
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(
        screen.queryByRole('dialog', { name: named(message(language, 'chukta-add-title')) }),
      ).toBeNull();
    });
    expect(onHit).not.toHaveBeenCalled();
    await waitFor(() => {
      expect(input.value).toBe('tariq');
    });
    expect(accountApi.chuktaAdd).not.toHaveBeenCalled();
  });
});

describe.each(LANGUAGES)('every other buyer box (%s)', (language) => {
  it('lists no چکتا buyer and has one add row only', async () => {
    const { input, user } = setup(language, { withChukta: false });
    await user.click(input);
    await user.keyboard('shahid');
    await waitFor(() => {
      expect(accountApi.searchAccounts).toHaveBeenCalled();
    });
    await waitFor(() => {
      expect(options()).toHaveLength(0);
    });
    // The search is asked without the flag: a missing key is "no چکتا buyers".
    for (const [input] of vi.mocked(accountApi.searchAccounts).mock.calls) {
      expect(input.withChukta).toBeUndefined();
    }
    await waitFor(() => {
      expect(addRow()).not.toBeNull();
    });
    expect(extraRow()).toBeNull();
  });
});
