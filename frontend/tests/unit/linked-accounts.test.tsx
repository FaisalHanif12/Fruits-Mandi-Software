// One person who is both a seller and a buyer has two linked accounts (PRD E10, FR-10.2, plan 80 T4-2). The
// party card shows the other account with its balance and both balances added together, and the lines above
// R13 and R14 carry the same combined balance. Both come from the backend: the card as `linked`, the ledger as
// a summary line named by the message ID `report-linked-combined`. The screen adds nothing (CLAUDE.md rule 1).
// In the Urdu and in the English app.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportSummaryLine } from '../../src/bindings/ReportSummaryLine';
import { ReportSummary } from '../../src/features/reports/components/ReportSummary';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import { useLanguageStore } from '../../src/shared/i18n';
import { PartyCard } from '../../src/shared/ui';
import { account, balance, cardOf, ZERO } from '../helpers/accounts';
import { paisaForTest } from '../helpers/paisa';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/accounts/api');

const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  balance: balance(2_880, 'dr'),
});
const RAHIM = account(31, 'seller', '0010', 'رحیم', 'Rahim', { balance: balance(10_160, 'cr') });

/** Akram is a buyer and also Rahim's seller account: 2,880 بنام and 10,160 جمع give 7,280 جمع. */
const LINKED = {
  account_id: 31,
  code: '0010',
  account_type: 'seller',
  name_ur: 'رحیم',
  name_en: 'Rahim',
  is_active: true,
  balance: balance(10_160, 'cr'),
  combined: balance(7_280, 'cr'),
} as const;

function card(id: number) {
  return (
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <PartyCard accountId={id} />
    </QueryClientProvider>
  );
}

beforeEach(() => {
  vi.resetAllMocks();
});

describe.each(LANGUAGES)('linked seller and buyer accounts (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);
  const nameOf = (row: { name_ur: string; name_en: string }) =>
    language === 'ur' ? row.name_ur : row.name_en;

  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  describe('the party card', () => {
    it('shows the other account, its balance and both balances together, each with its side', async () => {
      vi.mocked(accountApi.assistPartyCard).mockResolvedValue(cardOf(AKRAM, { linked: LINKED }));
      renderIn(language, card(21));
      const heading = await screen.findByText(visible(m('party-card-linked-heading')));
      const block = heading.closest('[data-linked-account]');
      if (!(block instanceof HTMLElement)) throw new Error('no linked block on the card');
      const text = visible(block.textContent);
      // The other account: its name, code and kind.
      expect(text).toContain(nameOf(LINKED));
      expect(text).toContain(LINKED.code);
      expect(text).toContain(m('accounts-type-seller'));
      // Its own balance and both together, the backend's figures, each with the word of its side.
      const factOf = (labelId: string) =>
        visible(within(block).getByText(visible(m(labelId))).parentElement?.textContent);
      expect(factOf('assist-card-balance')).toContain('10,160');
      expect(factOf('assist-card-balance')).toContain(m('common-side-cr'));
      expect(factOf('party-card-linked-both')).toContain('7,280');
      expect(factOf('party-card-linked-both')).toContain(m('common-side-cr'));
      // The card's own balance is still its own: 2,880 بنام.
      expect(visible(document.body.textContent)).toContain('2,880');
    });

    it('a card with no linked account shows no such block', async () => {
      vi.mocked(accountApi.assistPartyCard).mockResolvedValue(cardOf(RAHIM, { balance: ZERO }));
      renderIn(language, card(31));
      await waitFor(() => {
        expect(visible(document.body.textContent)).toContain(nameOf(RAHIM));
      });
      expect(screen.queryByText(visible(m('party-card-linked-heading')))).toBeNull();
      expect(document.querySelector('[data-linked-account]')).toBeNull();
    });

    it('an inactive other account is still shown, marked inactive: its balance is real', async () => {
      vi.mocked(accountApi.assistPartyCard).mockResolvedValue(
        cardOf(AKRAM, { linked: { ...LINKED, is_active: false } }),
      );
      renderIn(language, card(21));
      const heading = await screen.findByText(visible(m('party-card-linked-heading')));
      const block = heading.closest('[data-linked-account]');
      if (!(block instanceof HTMLElement)) throw new Error('no linked block on the card');
      expect(visible(block.textContent)).toContain(m('assist-warning-inactive'));
      expect(within(block).getByText('7,280')).toBeInTheDocument();
    });
  });

  describe('the lines above R13 and R14', () => {
    const LINE: ReportSummaryLine = {
      label_id: 'report-linked-combined',
      cell: { type: 'balance', paisa: paisaForTest(728_000), side: 'cr' },
    };

    it('say "both accounts together" with the combined balance and its side, in words', () => {
      renderIn(language, <ReportSummary lines={[LINE]} />);
      const label = screen.getByText(visible(m('report-linked-combined')));
      const row = label.parentElement;
      expect(visible(row?.textContent)).toContain(m('report-linked-combined'));
      expect(visible(row?.textContent)).toContain('7,280');
      expect(visible(row?.textContent)).toContain(m('common-side-cr'));
    });

    it('the message exists in both languages and says the same as the card', () => {
      expect(m('report-linked-combined')).toBe(m('party-card-linked-both'));
    });
  });
});
