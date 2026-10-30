// The dead amount on the party card and in the pickers (plan 40, PRD FR-10.10, FR-7.38, FR-7.40): a
// buyer who owes gets "ڈوبی رقم میں ڈالیں"; a person already in the dead amount shows "ڈوبی رقم" with
// the date and the reason and gets "دوبارہ فعال کریں"; the receipt picker marks such a person with the
// tag "ڈوبی" and still lets him be picked. Without the app frame's dialogs there are no buttons.
// In the Urdu and in the English app.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import {
  AccountPicker,
  DeadAmountContext,
  Field,
  PartyCard,
  type DeadAmountActions,
  type PickerValue,
} from '../../src/shared/ui';
import { account, balance, cardOf, fakeAccountSearch, ZERO } from '../helpers/accounts';
import { ARSHAD_DEAD } from '../helpers/dead';
import { must } from '../helpers/must';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/accounts/api');

const AKRAM = account(21, 'buyer', '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  balance: balance(2880, 'dr'),
});
const ARSHAD = account(22, 'buyer', '03040002', 'ارشد ملتانی', 'Arshad Multani', {
  balance: balance(18684, 'dr'),
  is_active: false,
});
const RAHIM = account(31, 'seller', '030110001', 'رحیم', 'Rahim', { balance: ZERO });

const named = (text: string) => (name: string) => visible(name) === text;

function actions() {
  return {
    move: vi.fn<DeadAmountActions['move']>(),
    undo: vi.fn<DeadAmountActions['undo']>(),
  };
}

function card(id: number, host: DeadAmountActions | null) {
  const body = <PartyCard accountId={id} />;
  return (
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      {host === null ? (
        body
      ) : (
        <DeadAmountContext.Provider value={host}>{body}</DeadAmountContext.Provider>
      )}
    </QueryClientProvider>
  );
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(accountApi.assistPartyCard).mockImplementation((id) => {
    if (id === 21) return Promise.resolve(cardOf(AKRAM));
    if (id === 22) {
      return Promise.resolve(
        cardOf(ARSHAD, { dead: ARSHAD_DEAD, warnings: [{ kind: 'inactive', amount_paisa: null }] }),
      );
    }
    return Promise.resolve(cardOf(RAHIM));
  });
});

describe.each(LANGUAGES)('the dead amount on the party card (%s)', (language: Language) => {
  const m = (id: string, args: Record<string, string | number> = {}) => message(language, id, args);
  const nameOf = (row: { name_ur: string; name_en: string }) =>
    language === 'ur' ? row.name_ur : row.name_en;

  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  it('a buyer who owes can be moved: the button asks the frame for the dialog with his names', async () => {
    const host = actions();
    renderIn(language, card(21, host));
    const user = userEvent.setup();
    await user.click(await screen.findByRole('button', { name: named(m('dead-move-button')) }));
    expect(host.move).toHaveBeenCalledWith({
      accountId: 21,
      personKey: null,
      names: { name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax', tag: null },
    });
    expect(screen.queryByRole('button', { name: named(m('dead-undo-button')) })).toBeNull();
  });

  it('is reached with the keyboard alone: Tab to the button, Enter presses it', async () => {
    const host = actions();
    renderIn(language, card(21, host));
    await screen.findByRole('button', { name: named(m('dead-move-button')) });
    const user = userEvent.setup();
    await user.tab();
    await user.tab();
    await user.tab();
    expect(visible(document.activeElement?.textContent)).toBe(m('dead-move-button'));
    await user.keyboard('{Enter}');
    expect(host.move).toHaveBeenCalledTimes(1);
  });

  it('a person in the dead amount says so with the date and the reason, and can be made active again', async () => {
    const host = actions();
    renderIn(language, card(22, host));
    const line = await screen.findByText(
      (_, element) =>
        element?.tagName === 'SPAN' &&
        visible(element.textContent) ===
          m('dead-info', { date: '11-10-2026', reason: m('dead-reason-died') }),
    );
    expect(line).toBeInTheDocument();
    expect(visible(document.querySelector('[data-dead-info]')?.textContent)).toContain(
      m('dead-info-note', { note: ARSHAD_DEAD.note }).replace(/<\/?note>/g, ''),
    );
    expect(screen.queryByRole('button', { name: named(m('dead-move-button')) })).toBeNull();
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: named(m('dead-undo-button')) }));
    expect(host.undo).toHaveBeenCalledWith({
      markId: ARSHAD_DEAD.mark_id,
      names: { name_ur: 'ارشد ملتانی', name_en: 'Arshad Multani', tag: null },
    });
    expect(screen.getByText(nameOf(ARSHAD))).toBeInTheDocument();
  });

  it('a seller whose balance is nothing has no move button', async () => {
    renderIn(language, card(31, actions()));
    await screen.findByText(nameOf(RAHIM));
    expect(screen.queryByRole('button', { name: named(m('dead-move-button')) })).toBeNull();
  });

  it('has no dead button where the app frame gives no dialog', async () => {
    renderIn(language, card(21, null));
    await screen.findByText(nameOf(AKRAM));
    expect(screen.queryByRole('button', { name: named(m('dead-move-button')) })).toBeNull();
    expect(screen.queryByRole('button', { name: named(m('dead-undo-button')) })).toBeNull();
  });
});

function Harness({
  onValue,
  includeDead = false,
}: {
  onValue: (value: PickerValue | null) => void;
  includeDead?: boolean;
}) {
  const [value, setValue] = useState<PickerValue | null>(null);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <AccountPicker
            {...control}
            scope="buyer"
            includeDead={includeDead}
            value={value}
            onChange={(hit) => {
              const next =
                hit === null
                  ? null
                  : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en };
              setValue(next);
              onValue(next);
            }}
          />
        )}
      </Field>
    </form>
  );
}

describe.each(LANGUAGES)('the tag ڈوبی in the picker (%s)', (language: Language) => {
  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
    // The search gives the dead person as the backend does for the receipt picker: inactive, dead.
    fakeAccountSearch([AKRAM, { ...ARSHAD, is_active: true }]);
  });

  it('marks a person in the dead amount with the tag, and he can still be picked', async () => {
    const hits = vi.mocked(accountApi.searchAccounts);
    const base = hits.getMockImplementation();
    hits.mockImplementation(async (input) => {
      const rows = (await base?.(input)) ?? [];
      return rows.map((row) => (row.id === 22 ? { ...row, is_active: false, is_dead: true } : row));
    });
    const picked = vi.fn();
    renderIn(
      language,
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <Harness onValue={picked} />
      </QueryClientProvider>,
    );
    const user = userEvent.setup();
    const input = screen.getByRole<HTMLInputElement>('combobox');
    await user.click(input);
    const rows = await screen.findAllByRole('option');
    expect(rows).toHaveLength(2);
    const tagged = rows.filter((row) =>
      visible(row.textContent).includes(message(language, 'dead-tag')),
    );
    expect(tagged).toHaveLength(1);
    expect(visible(tagged[0]?.textContent)).toContain('03040002');
    await user.click(must(tagged[0]));
    expect(picked).toHaveBeenLastCalledWith(expect.objectContaining({ id: 22, code: '03040002' }));
  });
});

describe.each(LANGUAGES)(
  'a person in the dead amount in the receipt field (%s)',
  (language: Language) => {
    const DEAD_HIT = {
      id: 22,
      code: '03040002',
      account_type: 'buyer' as const,
      control_id: 4,
      name_ur: 'ارشد ملتانی',
      name_en: 'Arshad Multani',
      extra_ur: '',
      extra_en: '',
      is_active: false,
      is_dead: true,
      balance: null,
    };

    beforeEach(() => {
      act(() => {
        useLanguageStore.setState({ language });
      });
      fakeAccountSearch([AKRAM]);
      vi.mocked(accountApi.accountsResolveCode).mockResolvedValue({
        status: 'inactive',
        account: DEAD_HIT,
        candidates: [],
        suggested_code: null,
      });
    });

    function open(includeDead: boolean, onValue = vi.fn()) {
      renderIn(
        language,
        <QueryClientProvider
          client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
        >
          <Harness onValue={onValue} includeDead={includeDead} />
        </QueryClientProvider>,
      );
      return {
        onValue,
        user: userEvent.setup(),
        input: screen.getByRole<HTMLInputElement>('combobox'),
      };
    }

    it('asks the backend for dead people only when the field takes them', async () => {
      const { user, input } = open(true);
      await user.click(input);
      await screen.findAllByRole('option');
      expect(accountApi.searchAccounts).toHaveBeenCalledWith(
        expect.objectContaining({ includeDead: true }),
      );
    });

    it('a plain field never asks for them', async () => {
      const { user, input } = open(false);
      await user.click(input);
      await screen.findAllByRole('option');
      expect(accountApi.searchAccounts).toHaveBeenCalled();
      expect(accountApi.searchAccounts).not.toHaveBeenCalledWith(
        expect.objectContaining({ includeDead: true }),
      );
    });

    it('his code typed and left picks him in the receipt field', async () => {
      const { user, input, onValue } = open(true);
      await user.click(input);
      await user.keyboard('03040002{Tab}');
      await waitFor(() => {
        expect(onValue).toHaveBeenLastCalledWith(expect.objectContaining({ id: 22 }));
      });
    });

    it('the same code in a field that does not take him is told as an inactive account, never picked', async () => {
      const { user, input, onValue } = open(false);
      await user.click(input);
      await user.keyboard('03040002{Tab}');
      await screen.findByText(
        (content) =>
          visible(content) ===
          message(language, 'accountpicker-inactive', {
            name: language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani',
            code: '03040002',
          }),
      );
      expect(onValue).not.toHaveBeenCalledWith(expect.objectContaining({ id: 22 }));
    });
  },
);
