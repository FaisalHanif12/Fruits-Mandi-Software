// A پختہ party's own account on the sale screen (plan 47, PRD FR-13.11, FR-12.5): he is a full buyer
// or seller of his side, so his lines and lots are sent like any other party's: no name, phone or
// person key is asked or sent. Only the two shared accounts, 0030 and 0031, ask for a name and a
// phone on the line (PRD FR-13.2). The cash first start of his lines (نقدی for a buyer with credit
// off) and of his lot (نقد as the lot's net) come from the backend's preview, which the sale screen
// shows without a rule of its own; the preview side is tested with the backend's answers.

import { describe, expect, it } from 'vitest';
import { isPukhtaBuyer, isPukhtaSeller, saleInput } from '../../src/features/bikri/draft';
import { LOT, SALE } from '../../src/features/bikri/types';
import type { EntryRow } from '../../src/shared/grid';

const picked = (code: string) => ({ id: 1, code, name_ur: 'نذیر', name_en: 'Nazir' });

describe('which accounts ask for a name on the line', () => {
  it('the shared 0030 and 0031 do, his own 0032 and 0033 do not', () => {
    const buyer = (code: string): EntryRow => ({
      rowId: 'b',
      typed: {},
      picked: { [SALE.buyer]: picked(code) },
    });
    const seller = (code: string): EntryRow => ({
      rowId: 's',
      typed: {},
      picked: { [LOT.seller]: picked(code) },
    });
    expect(isPukhtaBuyer(buyer('0030'))).toBe(true);
    expect(isPukhtaBuyer(buyer('0032'))).toBe(false);
    expect(isPukhtaBuyer(buyer('0020'))).toBe(false);
    expect(isPukhtaSeller(seller('0031'))).toBe(true);
    expect(isPukhtaSeller(seller('0033'))).toBe(false);
    expect(isPukhtaSeller(seller('0010'))).toBe(false);
  });

  it('a line of his is sent with no one-time person, even if a name was left in a hidden cell', () => {
    const row: EntryRow = {
      rowId: 'line-1',
      typed: {
        [SALE.qty]: '10',
        [SALE.rate]: '1,200',
        [SALE.pukhtaUr]: 'شاہد',
        [SALE.pukhtaEn]: 'Shahid',
        [SALE.pukhtaPhone]: '0300-0000000',
      },
      picked: { [SALE.buyer]: picked('0032') },
    };
    const sent = saleInput(row);
    expect(sent.buyer_account_id).toBe(1);
    expect(sent.pukhta).toBeNull();
    // The payment type is not decided here: left empty, the backend takes it from his credit setting.
    expect(sent.payment_type).toBeNull();
  });

  it('a line of the shared buyer still carries the person typed on it', () => {
    const row: EntryRow = {
      rowId: 'line-2',
      typed: {
        [SALE.qty]: '10',
        [SALE.rate]: '1,200',
        [SALE.pukhtaUr]: 'شاہد',
        [SALE.pukhtaEn]: 'Shahid',
        [SALE.pukhtaPhone]: '0300-0000000',
      },
      picked: { [SALE.buyer]: picked('0030') },
    };
    expect(saleInput(row).pukhta).toEqual({
      name_ur: 'شاہد',
      name_en: 'Shahid',
      phone: '0300-0000000',
    });
  });
});
