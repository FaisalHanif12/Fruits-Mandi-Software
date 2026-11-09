// The party card of a چکتا buyer (plan 64, PRD FR-7.38, FR-13.6): his name, his phone and the words
// "چکتا: پورا نقد" (pays everything in cash), and nothing else. He has no balance, no credit line and
// no ledger, so the card has no balance and no report buttons even where the app frame gives a way
// to open reports. The backend answers for a person it knows (`chukta_id`); a person read back from
// a saved line has no record id, so his card is drawn from the names and phone the line holds and
// the backend is not asked. In the Urdu and in the English app, against the backend's real answer.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import { useLanguageStore } from '../../src/shared/i18n';
import { PartyCard } from '../../src/shared/ui';
import { PartyCardLinksProvider } from '../../src/shared/ui/party-card-links';
import { account, balance, cardOf, money } from '../helpers/accounts';
import { CARD_CHUKTA_BUYER, CARD_CHUKTA_BUYER_REQUEST } from '../helpers/bikri-real';
import { message, paragraph, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/accounts/api');

const SHAHID = { id: 1, name_ur: 'شاہد', name_en: 'Shahid', phone: '03000000000' };
const AKRAM = account(20, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  balance: balance(2880, 'dr'),
  credit_limit_paisa: money(50000),
});

beforeEach(() => {
  vi.resetAllMocks();
});

function client() {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

describe.each(LANGUAGES)('the party card of a چکتا buyer (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);
  const named = (text: string) => (name: string) => visible(name) === text;
  const shahid = language === 'ur' ? 'شاہد' : 'Shahid';

  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  it('asks the backend with his record id and shows his name, phone and that he pays in cash', async () => {
    vi.mocked(accountApi.assistPartyCard).mockResolvedValue(CARD_CHUKTA_BUYER);
    renderIn(
      language,
      <QueryClientProvider client={client()}>
        <PartyCard accountId={9} chukta={SHAHID} />
      </QueryClientProvider>,
    );
    expect(await screen.findByText(shahid)).toBeTruthy();
    // The request is the one the backend's own test wrote: the shared account 0030 and the person.
    expect(vi.mocked(accountApi.assistPartyCard).mock.calls).toEqual([
      [CARD_CHUKTA_BUYER_REQUEST['account_id'], CARD_CHUKTA_BUYER_REQUEST['chukta_id']],
    ]);
    expect(screen.getByText(m('accounts-type-chukta'))).toBeTruthy();
    expect(screen.getByText('03000000000')).toBeTruthy();
    expect(screen.getByText(paragraph(m('assist-card-chukta-cash')))).toBeTruthy();
  });

  it('has no balance, no credit line and no report buttons, even where reports can be opened', async () => {
    vi.mocked(accountApi.assistPartyCard).mockResolvedValue(CARD_CHUKTA_BUYER);
    renderIn(
      language,
      <QueryClientProvider client={client()}>
        <PartyCardLinksProvider value={vi.fn()}>
          <PartyCard accountId={9} chukta={SHAHID} />
        </PartyCardLinksProvider>
      </QueryClientProvider>,
    );
    await screen.findByText(shahid);
    expect(screen.queryByRole('button', { name: named(m('report-card-ledger')) })).toBeNull();
    expect(screen.queryByRole('button', { name: named(m('report-card-statement')) })).toBeNull();
    expect(screen.queryByText(m('assist-card-balance'))).toBeNull();
    expect(screen.queryByText(m('assist-card-credit-limit'))).toBeNull();
  });

  it('draws a person read back from a saved line from the line, without asking the backend', async () => {
    renderIn(
      language,
      <QueryClientProvider client={client()}>
        <PartyCard
          accountId={9}
          chukta={{ id: null, name_ur: 'ناصر', name_en: 'Nasir', phone: '03001111111' }}
        />
      </QueryClientProvider>,
    );
    expect(await screen.findByText(language === 'ur' ? 'ناصر' : 'Nasir')).toBeTruthy();
    expect(screen.getByText('03001111111')).toBeTruthy();
    expect(screen.getByText(paragraph(m('assist-card-chukta-cash')))).toBeTruthy();
    expect(vi.mocked(accountApi.assistPartyCard)).not.toHaveBeenCalled();
  });

  it('leaves the phone out when he gave none', async () => {
    renderIn(
      language,
      <QueryClientProvider client={client()}>
        <PartyCard
          accountId={9}
          chukta={{ id: null, name_ur: 'ناصر', name_en: 'Nasir', phone: '' }}
        />
      </QueryClientProvider>,
    );
    expect(await screen.findByText(language === 'ur' ? 'ناصر' : 'Nasir')).toBeTruthy();
    expect(document.querySelector('bdi')).toBeNull();
    expect(screen.getByText(paragraph(m('assist-card-chukta-cash')))).toBeTruthy();
  });

  it('still shows an ordinary buyer with his balance, so the new card changes nothing for him', async () => {
    vi.mocked(accountApi.assistPartyCard).mockResolvedValue(cardOf(AKRAM));
    renderIn(
      language,
      <QueryClientProvider client={client()}>
        <PartyCard accountId={20} />
      </QueryClientProvider>,
    );
    expect(
      await screen.findByText(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax'),
    ).toBeTruthy();
    expect(vi.mocked(accountApi.assistPartyCard).mock.calls).toEqual([[20]]);
    expect(screen.queryByText(paragraph(m('assist-card-chukta-cash')))).toBeNull();
  });
});
