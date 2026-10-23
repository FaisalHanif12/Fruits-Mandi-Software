// The two report buttons of the party card (plan 20, step F3, PRD FR-7.65): کھاتہ opens the ledger
// (R13) and فرد حساب the statement (R14) of the account on the card, with nothing to type. The card
// is shared and knows no router: without the app frame's provider it has no buttons; with it, a press
// opens the report of that account in the real router. In the Urdu and in the English app.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useParams, useSearch } from '@tanstack/react-router';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { PartyReportLinks } from '../../src/features/reports';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import { useLanguageStore } from '../../src/shared/i18n';
import { PartyCard } from '../../src/shared/ui';
import { PartyCardLinksProvider } from '../../src/shared/ui/party-card-links';
import { account, balance, cardOf, money } from '../helpers/accounts';
import { message, renderIn, visible } from '../helpers/render';
import { renderRouted } from '../helpers/router';

vi.mock('../../src/shared/accounts/api');

const AKRAM = account(20, 'buyer', '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  balance: balance(2880, 'dr'),
  credit_limit_paisa: money(50000),
});

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(accountApi.assistPartyCard).mockResolvedValue(cardOf(AKRAM));
});

describe.each(LANGUAGES)('the party card report buttons (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);
  const named = (text: string) => (name: string) => visible(name) === text;

  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  function card(open: ((report: 'R13' | 'R14', accountId: number) => void) | null) {
    return (
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <PartyCardLinksProvider value={open}>
          <PartyCard accountId={20} />
        </PartyCardLinksProvider>
      </QueryClientProvider>
    );
  }

  it('has no buttons where the app frame gives no way to open a report', async () => {
    renderIn(language, card(null));
    await screen.findByText(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(screen.queryByRole('button', { name: named(m('report-card-ledger')) })).toBeNull();
    expect(screen.queryByRole('button', { name: named(m('report-card-statement')) })).toBeNull();
  });

  it('opens the ledger and the statement of the account on the card', async () => {
    const open = vi.fn();
    renderIn(language, card(open));
    const user = userEvent.setup();
    await user.click(await screen.findByRole('button', { name: named(m('report-card-ledger')) }));
    await user.click(screen.getByRole('button', { name: named(m('report-card-statement')) }));
    expect(open.mock.calls).toEqual([
      ['R13', 20],
      ['R14', 20],
    ]);
  });

  it('is one group of two buttons, named for a screen reader, that the keyboard reaches', async () => {
    renderIn(language, card(vi.fn()));
    const group = await screen.findByRole('group', { name: m('report-card-reports-label') });
    expect(group.querySelectorAll('button')).toHaveLength(2);
    const user = userEvent.setup();
    await user.tab();
    expect(document.activeElement?.textContent).toBe(m('report-card-ledger'));
    await user.keyboard('{Tab}');
    expect(document.activeElement?.textContent).toBe(m('report-card-statement'));
  });

  it('goes to the report of the account, with the account in the address, through the real router', async () => {
    // The app frame wraps the screens in PartyReportLinks; here a page does the same inside a router.
    const Page = () => (
      <PartyReportLinks>
        <PartyCard accountId={20} />
      </PartyReportLinks>
    );
    // The report page shows what it was opened with: which report, and the filters in the address.
    const Report = () => (
      <p data-testid="opened">
        {useParams({ strict: false }).id} {JSON.stringify(useSearch({ strict: false }))}
      </p>
    );
    await renderRouted({ '/': Page, '/reports/$id': Report }, '/');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: named(m('report-card-statement')) }),
    );
    await waitFor(() => {
      expect(screen.getByTestId('opened').textContent).toBe('R14 {"party":20}');
    });
  });
});
