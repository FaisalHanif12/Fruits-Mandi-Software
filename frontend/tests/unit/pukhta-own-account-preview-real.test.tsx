// The sale screen with a پختہ party's own account, against the backend's own previews (plan 47, PRD
// FR-13.11, FR-12.9, A79): نذیر, a پختہ buyer 0032 with credit off, buys 10 at 1,200 and his line starts
// as نقدی, and he pays 12,800; کمال, a پختہ seller 0033 with an open advance of 5,000, sends a lot
// whose نقد the backend starts at 5,160 (10,160 less the 5,000 the lot adjusts). The two files were
// written by the backend's own test (tests/fixtures/bikri_preview_pukhta_party_*), copied and never
// edited. The screen only shows them: the numbers are the backend's, the نقد cell stays empty in what
// the screen sends, and the working is read in both languages.

import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { BikriPreviewInput } from '../../src/bindings/BikriPreviewInput';
import { lineView, lotView } from '../../src/features/bikri/preview-view';
import { saleInput, lotInput } from '../../src/features/bikri/draft';
import type { HeaderState } from '../../src/features/bikri/draft-store';
import { LOT, SALE } from '../../src/features/bikri/types';
import { LANGUAGES } from '../../src/i18n';
import type { EntryRow } from '../../src/shared/grid';
import { CalcInfo } from '../../src/shared/ui';
import buyerCash from '../fixtures/bikri_preview_pukhta_party_buyer_cash.json';
import buyerCashRequest from '../fixtures/bikri_preview_pukhta_party_buyer_cash_request.json';
import sellerAdvance from '../fixtures/bikri_preview_pukhta_party_seller_advance.json';
import sellerAdvanceRequest from '../fixtures/bikri_preview_pukhta_party_seller_advance_request.json';
import { message, renderIn, visible } from '../helpers/render';

// The fixture files are the backend's own JSON, read as the generated types.
const BUYER_CASH = buyerCash as unknown as BikriPreview;
const BUYER_CASH_REQUEST = buyerCashRequest as unknown as BikriPreviewInput;
const SELLER_ADVANCE = sellerAdvance as unknown as BikriPreview;
const SELLER_ADVANCE_REQUEST = sellerAdvanceRequest as unknown as BikriPreviewInput;

const HEADER: HeaderState = {
  entryDate: '10-10-2026',
  billDate: '',
  book: '',
  truckNo: 'LES-1234',
  zamindar: null,
  adda: null,
  driverName: '',
  driverPhone: '',
  totalFreight: '',
  store: null,
  note: '',
};

const say = (id: string) => id;
const person = (id: number, code: string, ur: string, en: string) => ({
  id,
  code,
  name_ur: ur,
  name_en: en,
});

/** The sale line of the backend's file, as the screen holds it: the buyer picked, qty and rate typed. */
const saleRow = (buyer: ReturnType<typeof person>): EntryRow => ({
  rowId: 's-1',
  typed: { [SALE.qty]: '10', [SALE.rate]: '1200' },
  picked: { [SALE.buyer]: buyer },
});

describe('the buyer: a پختہ گاہک with credit off (A79)', () => {
  const NAZIR = person(32, 'x', 'نذیر', 'Nazir');
  const requested = BUYER_CASH_REQUEST.draft.lots[0]?.sales[0];

  it('the screen sends his line like any buyer: his account, no one-time person, no payment type', () => {
    const sent = saleInput(saleRow({ ...NAZIR, code: '0032' }));
    expect(requested).toBeDefined();
    expect(sent.buyer_account_id).toBe(requested?.buyer_account_id);
    expect(sent.buyer_account_id).toBe(32);
    expect(sent.pukhta).toBeNull();
    expect(requested?.pukhta).toBeNull();
    expect(sent.qty).toBe(requested?.qty);
    expect(sent.seller_rate).toBe(requested?.seller_rate);
    // Nothing is chosen for the payment type: the backend takes it from his credit setting.
    expect(sent.payment_type).toBeNull();
    expect(requested?.payment_type).toBeNull();
  });

  it('the line the backend answers starts as نقدی and he pays 12,800, shown as it came', () => {
    const line = BUYER_CASH.lots[0]?.lines[0];
    const view = lineView(line, 'international', say);
    expect(view.calc[SALE.payment]).toBe('cash');
    expect(view.calc[SALE.buyerTotal]).toBe('12,800');
    expect(view.calc[SALE.sellerComm]).toBe('720');
    expect(view.calc[SALE.buyerComm]).toBe('720');
    expect(view.calc[SALE.laga]).toBe('80');
    expect(BUYER_CASH.errors).toEqual([]);
    expect(BUYER_CASH.warnings).toEqual([]);
  });
});

describe('the seller: a پختہ بیوپاری with an open advance (FR-13.11, Umar 01-11-2026)', () => {
  const requested = SELLER_ADVANCE_REQUEST.draft.lots[0];

  it('the screen sends his lot with no نقد and the advance tick on, as the backend’s test did', () => {
    const kamal = person(33, '0033', 'کمال', 'Kamal');
    const lotRow: EntryRow = {
      rowId: 'lot-1',
      typed: { [LOT.qty]: '10', [LOT.freight]: '1,000', [LOT.labour]: '5', [LOT.munshiana]: '50' },
      picked: { [LOT.seller]: kamal },
    };
    const sent = lotInput(lotRow, [saleRow(person(20, '0020', 'اکرم', 'Akram'))], HEADER);
    expect(sent.seller_account_id).toBe(requested?.seller_account_id);
    expect(sent.seller_account_id).toBe(33);
    // He is a full seller of his own: no one-time person, and نقد left empty for the backend.
    expect(sent.pukhta).toBeNull();
    expect(requested?.pukhta).toBeNull();
    expect(sent.cash_paid).toBeNull();
    expect(requested?.cash_paid).toBeNull();
    expect(sent.adjust_advance).toBe(true);
    expect(requested?.adjust_advance).toBe(true);
  });

  it('the نقد cell shows the backend’s 5,160 with the advance working; the net is the 5,000 left', () => {
    const lot = SELLER_ADVANCE.lots[0];
    const view = lotView(lot, 'international');
    expect(view.calc[LOT.cash]).toBe('5,160');
    expect(view.calc[LOT.net]).toBe('5,000');
    expect(view.workings[LOT.cash]?.message_id).toBe('working-cash-pukhta-advance');
    expect(SELLER_ADVANCE.errors).toEqual([]);
  });

  it.each(LANGUAGES)(
    'the working reads in the app language with its three numbers (%s)',
    async (language) => {
      const working = SELLER_ADVANCE.lots[0]?.totals?.workings.cash_paid;
      if (working === undefined) throw new Error('the backend sent no working');
      renderIn(language, <CalcInfo working={working} />);
      await userEvent
        .setup()
        .click(screen.getByRole('button', { name: message(language, 'calc-info-label') }));
      await screen.findByText(message(language, 'calc-info-title'));
      expect(visible(document.body.textContent)).toContain(
        message(language, 'working-cash-pukhta-advance', {
          credit: '10,160',
          advance: '5,000',
          result: '5,160',
        }),
      );
    },
  );
});
