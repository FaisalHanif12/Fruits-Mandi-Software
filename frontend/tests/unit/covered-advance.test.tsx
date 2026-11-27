// The "covered advance" (PRD FR-9.6 v1.61, plan 100 A, Umar 28-11-2026) in the Urdu and the English app:
// an advance the seller's own credit already covers is not shown as پیشگی باقی; one line says it was
// taken out of his balance. With Umar's فیصل: payable 117,040, advance 100,000, so the net is 17,040 and
// nothing of the advance is left uncovered. The party card, the پیشگی card and the payables panel of
// ادائیگی write only the backend's figures (CLAUDE.md rule 1).

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AdvanceHistory } from '../../src/features/vouchers/components/AdvanceHistory';
import { PayablesPanel } from '../../src/features/vouchers/components/PayablesPanel';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import { PartyCard } from '../../src/shared/ui';
import { account, balance, cardOf, money } from '../helpers/accounts';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/vouchers/api');

const FAISAL = account(40, 'seller', '0013', 'فیصل', 'Faisal', { balance: balance(17040, 'cr') });

beforeEach(() => {
  vi.resetAllMocks();
});

function show(language: Language, ui: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return renderIn(language, <QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}

describe.each(LANGUAGES)('an advance the balance already covers (%s)', (language) => {
  const coveredWords = message(language, 'assist-advance-covered', { amount: '100,000' }).replace(
    /<\/?num>/g,
    '',
  );
  const line = () => visible(document.querySelector('[data-advance-covered]')?.textContent);

  it('the party card shows nothing uncovered, the net to pay, and the line for the covered part', async () => {
    vi.mocked(accountApi.assistPartyCard).mockResolvedValue(
      cardOf(FAISAL, {
        advance_open: money(0),
        advance_covered: money(100000),
        net_payable: money(17040),
      }),
    );
    show(language, <PartyCard accountId={FAISAL.id} />);
    await waitFor(() => {
      expect(document.querySelector('[data-advance-covered]')).not.toBeNull();
    });
    expect(line()).toBe(coveredWords);
    const card = visible(document.body.textContent);
    expect(card).toContain(message(language, 'assist-card-net-payable'));
    expect(card).toContain('17,040');
  });

  it('the party card shows no line while nothing is covered', async () => {
    vi.mocked(accountApi.assistPartyCard).mockResolvedValue(
      cardOf(FAISAL, { advance_open: money(15000), advance_covered: null }),
    );
    show(language, <PartyCard accountId={FAISAL.id} />);
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'assist-card-advance'),
      );
    });
    expect(document.querySelector('[data-advance-covered]')).toBeNull();
  });

  it('the پیشگی card: پیشگی باقی is the uncovered part, with the line for the rest', async () => {
    vi.mocked(vouchersApi.assistAdvances).mockResolvedValue({
      open: [
        {
          source: 'voucher',
          voucher_number: 1,
          entry_date: '2026-11-20',
          given_paisa: money(100000),
          adjusted_paisa: money(0),
          open_paisa: money(100000),
        },
      ],
      total_count: 1,
      truncated: false,
      open_paisa: money(100000),
      uncovered_paisa: money(0),
      advance_covered: money(100000),
      last: null,
    });
    show(language, <AdvanceHistory sellerId={FAISAL.id} />);
    await waitFor(() => {
      expect(document.querySelector('[data-advance-uncovered]')).not.toBeNull();
    });
    const open = visible(document.querySelector('[data-advance-uncovered]')?.textContent);
    expect(open).toContain(message(language, 'vouchers-advances-open'));
    expect(open).not.toContain('100,000');
    expect(open).toContain('0');
    expect(line()).toBe(coveredWords);
  });

  it('ادائیگی: the note says nothing uncovered and the net to pay, then the covered line', async () => {
    vi.mocked(vouchersApi.assistOpenPayables).mockResolvedValue({
      account: { id: FAISAL.id, code: FAISAL.code, name_ur: 'فیصل', name_en: 'Faisal' },
      postings: [],
      total_count: 0,
      truncated: false,
      open_paisa: money(117040),
      advances_open_paisa: money(100000),
      net_payable_paisa: money(17040),
      uncovered_advance_paisa: money(0),
      advance_covered: money(100000),
      owes_firm_paisa: money(0),
      balance: balance(17040, 'cr'),
    });
    show(language, <PayablesPanel sellerId={FAISAL.id} />);
    await waitFor(() => {
      expect(document.querySelector('[data-payables-net]')).not.toBeNull();
    });
    expect(visible(document.querySelector('[data-payables-net]')?.textContent)).toBe(
      message(language, 'vouchers-payables-net-note', { advance: '0', net: '17,040' }).replace(
        /<\/?num>/g,
        '',
      ),
    );
    expect(line()).toBe(coveredWords);
  });
});
