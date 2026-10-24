// The voucher draft helpers: a new draft, changing the type, the input sent to the backend, which
// helper panel shows, and a saved voucher loaded back for an edit (PRD FR-7.67, FR-15.3, FR-15.11).

import { describe, expect, it } from 'vitest';
import {
  accountScope,
  build,
  changeType,
  draftOfRow,
  entryOf,
  fromSideOf,
  helperTarget,
  isCashLent,
  isFilled,
  needsPerson,
  newDraft,
  nextDraft,
  personSide,
} from '../../src/features/vouchers/draft';
import type { VoucherDraft } from '../../src/features/vouchers/types';
import { asScreenType, parseVoucherSearch } from '../../src/features/vouchers/screen-type';
import { moduleMenuOf } from '../../src/app/nav-routes';
import { voucherRow } from '../helpers/vouchers';

const AKRAM = { id: 21, code: '03040001', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax' };
const RAHIM = { id: 31, code: '030110001', name_ur: 'رحیم', name_en: 'Rahim' };
const PUKHTA_BUYER = {
  id: 23,
  code: '03049999',
  name_ur: 'پختہ کھاتہ (خریدار)',
  name_en: 'Pukhta Khata (Buyer)',
};
const PUKHTA_SELLER = {
  id: 24,
  code: '030119999',
  name_ur: 'پختہ کھاتہ (بیوپاری)',
  name_en: 'Pukhta Khata (Seller)',
};
const SHAHID = { name_ur: 'شاہد', name_en: 'Shahid', phone: '03000000000', person_key: 'k-shahid' };
const COLLECTOR = { id: 61, code: '', name_ur: 'اسلم', name_en: 'Aslam' };

const TODAY = '2026-10-10';

function draft(type: VoucherDraft['type'], changes: Partial<VoucherDraft> = {}): VoucherDraft {
  return { ...newDraft(type, TODAY), ...changes };
}

describe('newDraft', () => {
  it('starts with today as DD-MM-YYYY and everything else empty', () => {
    const created = newDraft('receipt', TODAY);
    expect(created).toMatchObject({
      type: 'receipt',
      date: '10-10-2026',
      account: null,
      amount: '',
      from: 'buyer',
    });
    // No mode, bank or cheque: all money is روکڑ (PRD 15.1).
    for (const gone of ['mode', 'bank', 'cheque_no', 'cheque_date']) {
      expect(created).not.toHaveProperty(gone);
    }
    expect(isFilled(created)).toBe(false);
  });

  it('is filled once an account or an amount is typed', () => {
    expect(isFilled(draft('payment', { amount: '5' }))).toBe(true);
    expect(isFilled(draft('payment', { amount: '  ' }))).toBe(false);
    expect(isFilled(draft('payment', { account: RAHIM }))).toBe(true);
  });
});

describe('changeType (PRD FR-7.67)', () => {
  it('keeps the date, amount and narration, and clears the rest', () => {
    const before = draft('payment', {
      account: RAHIM,
      accountType: 'seller',
      amount: '3000',
      narration: 'season',
      reason: 'cash for rent',
      collector: COLLECTOR,
      person: SHAHID,
      personText: 'Shahid',
    });
    const after = changeType(before, 'receipt');
    expect(after).toMatchObject({
      type: 'receipt',
      date: '10-10-2026',
      amount: '3000',
      narration: 'season',
      account: null,
      accountType: null,
      person: null,
      personText: '',
      reason: '',
      collector: null,
      from: 'buyer',
    });
  });

  it('returns the same draft when the type does not change', () => {
    const before = draft('advance', { amount: '1' });
    expect(changeType(before, 'advance')).toBe(before);
  });
});

describe('nextDraft', () => {
  it('keeps the type and the date, and starts the rest again', () => {
    const next = nextDraft(
      draft('receipt', {
        account: AKRAM,
        amount: '10000',
        narration: 'x',
      }),
    );
    expect(next).toMatchObject({
      type: 'receipt',
      date: '10-10-2026',
      account: null,
      amount: '',
      narration: '',
    });
  });
});

describe('what the account field takes', () => {
  it('is the chosen side for a receipt, sellers for an advance and any account for a payment', () => {
    expect(accountScope(draft('receipt'))).toBe('buyer');
    expect(accountScope(draft('receipt', { from: 'seller' }))).toBe('seller');
    expect(accountScope(draft('advance'))).toBe('seller');
    expect(accountScope(draft('payment'))).toBe('any');
  });

  it('needs a person only for the two one-time accounts, on their own side', () => {
    expect(needsPerson(draft('receipt', { account: AKRAM }))).toBe(false);
    expect(needsPerson(draft('receipt', { account: PUKHTA_BUYER }))).toBe(true);
    expect(personSide(draft('receipt', { account: PUKHTA_BUYER }))).toBe('buyer');
    expect(needsPerson(draft('payment', { account: PUKHTA_SELLER }))).toBe(true);
    expect(personSide(draft('payment', { account: PUKHTA_SELLER }))).toBe('seller');
  });

  it('calls a payment to a buyer cash lent (PRD FR-15.11)', () => {
    expect(isCashLent(draft('payment', { accountType: 'buyer' }))).toBe(true);
    expect(isCashLent(draft('payment', { accountType: 'seller' }))).toBe(false);
    expect(isCashLent(draft('receipt', { accountType: 'buyer' }))).toBe(false);
  });

  it('knows a receipt from a seller', () => {
    expect(fromSideOf('seller')).toBe('seller');
    expect(fromSideOf('buyer')).toBe('buyer');
    expect(fromSideOf(null)).toBe('buyer');
  });
});

describe('build', () => {
  it('is null until there is an account', () => {
    expect(build(draft('receipt'), false)).toBeNull();
  });

  it('sends a receipt as the text typed, with the collector and nothing about how it was paid', () => {
    const built = build(
      draft('receipt', {
        account: AKRAM,
        accountType: 'buyer',
        amount: '10,000',
        collector: COLLECTOR,
        narration: 'part',
      }),
      false,
    );
    expect(built).toEqual({
      type: 'receipt',
      input: {
        date: '10-10-2026',
        amount: '10,000',
        narration: 'part',
        account_id: 21,
        person_key: null,
        collector_staff_id: 61,
      },
    });
  });

  it('never sends a mode, a bank or a cheque, for any of the three types', () => {
    for (const type of ['receipt', 'payment', 'advance'] as const) {
      const built = build(
        draft(type, { account: AKRAM, accountType: 'buyer', amount: '5' }),
        false,
      );
      for (const gone of ['mode', 'bank_account_id', 'cheque_no', 'cheque_date']) {
        expect(built?.input, `${type} ${gone}`).not.toHaveProperty(gone);
      }
    }
  });

  it('sends the person key only for a one-time account', () => {
    const one = build(
      draft('receipt', { account: PUKHTA_BUYER, accountType: 'buyer', person: SHAHID }),
      false,
    );
    expect(one?.input).toMatchObject({ person_key: 'k-shahid' });
    const regular = build(
      draft('receipt', { account: AKRAM, accountType: 'buyer', person: SHAHID }),
      false,
    );
    expect(regular?.input).toMatchObject({ person_key: null });
  });

  it('sends the reason only for cash lent, and always sends the cash answer', () => {
    const lent = build(
      draft('payment', { account: AKRAM, accountType: 'buyer', amount: '3000', reason: 'urgent' }),
      true,
    );
    expect(lent).toMatchObject({
      type: 'payment',
      input: { reason: 'urgent', confirm_cash_below_zero: true, account_id: 21 },
    });
    const toSeller = build(
      draft('payment', {
        account: RAHIM,
        accountType: 'seller',
        amount: '4316',
        reason: 'left over',
      }),
      false,
    );
    expect(toSeller).toMatchObject({
      type: 'payment',
      input: { reason: '', confirm_cash_below_zero: false },
    });
  });

  it('sends an advance with the seller id and the expected goods', () => {
    const built = build(
      draft('advance', {
        account: RAHIM,
        accountType: 'seller',
        amount: '15000',
        expected_goods: 'mango',
      }),
      false,
    );
    expect(built).toMatchObject({
      type: 'advance',
      input: { seller_account_id: 31, expected_goods: 'mango', confirm_cash_below_zero: false },
    });
    if (built === null) throw new Error('expected a voucher');
    expect(entryOf(built)).toMatchObject({ type: 'advance', seller_account_id: 31 });
  });
});

describe('helperTarget (PRD FR-7.58 to FR-7.60)', () => {
  it('is none without an account', () => {
    expect(helperTarget(draft('receipt'))).toEqual({ kind: 'none' });
  });

  it('shows the open bills of a buyer for a receipt and for cash lent', () => {
    expect(helperTarget(draft('receipt', { account: AKRAM, accountType: 'buyer' }))).toEqual({
      kind: 'bills',
      accountId: 21,
      personKey: null,
    });
    expect(helperTarget(draft('payment', { account: AKRAM, accountType: 'buyer' }))).toEqual({
      kind: 'bills',
      accountId: 21,
      personKey: null,
    });
  });

  it('shows what is payable to a seller for a payment', () => {
    expect(helperTarget(draft('payment', { account: RAHIM, accountType: 'seller' }))).toEqual({
      kind: 'payables',
      accountId: 31,
      personKey: null,
    });
  });

  it('shows the advances of a seller for an advance and for a receipt from a seller', () => {
    expect(helperTarget(draft('advance', { account: RAHIM, accountType: 'seller' }))).toEqual({
      kind: 'advances',
      accountId: 31,
    });
    expect(
      helperTarget(draft('receipt', { account: RAHIM, accountType: 'seller', from: 'seller' })),
    ).toEqual({ kind: 'advances', accountId: 31 });
  });

  it('waits for the person on a one-time account, then keys the bills by him', () => {
    const waiting = draft('receipt', { account: PUKHTA_BUYER, accountType: 'buyer' });
    expect(helperTarget(waiting)).toEqual({ kind: 'none' });
    expect(helperTarget({ ...waiting, person: SHAHID })).toEqual({
      kind: 'bills',
      accountId: 23,
      personKey: 'k-shahid',
    });
  });

  it('shows nothing for a payment to another kind of account', () => {
    expect(helperTarget(draft('payment', { account: AKRAM, accountType: 'other' }))).toEqual({
      kind: 'none',
    });
  });
});

describe('draftOfRow (PRD FR-15.3)', () => {
  it('loads a saved voucher back as text', () => {
    const row = voucherRow({
      voucher_type: 'receipt',
      entry_date: '2026-10-09',
      narration: 'part',
      person: SHAHID,
      reason: '',
    });
    const loaded = draftOfRow(
      row,
      'receipt',
      { accountType: 'buyer', collector: COLLECTOR },
      'international',
      'en',
    );
    expect(loaded).toMatchObject({
      type: 'receipt',
      date: '09-10-2026',
      amount: '10,000',
      narration: 'part',
      person: SHAHID,
      personText: 'Shahid',
      collector: COLLECTOR,
      from: 'buyer',
    });
  });

  it('shows a receipt from a seller as from a seller and uses the Urdu name in Urdu', () => {
    const row = voucherRow({ voucher_type: 'receipt', person: SHAHID });
    const loaded = draftOfRow(
      row,
      'receipt',
      { accountType: 'seller', collector: null },
      'international',
      'ur',
    );
    expect(loaded.from).toBe('seller');
    expect(loaded.personText).toBe('شاہد');
  });
});

describe('the address of the voucher screen', () => {
  it('reads a type and an id and drops anything else', () => {
    expect(parseVoucherSearch({ type: 'payment', id: '12' })).toEqual({ type: 'payment', id: 12 });
    expect(parseVoucherSearch({ type: 'journal', id: 'x' })).toEqual({});
    expect(parseVoucherSearch({ id: -3 })).toEqual({});
    expect(asScreenType('advance')).toBe('advance');
    expect(asScreenType('nope')).toBeNull();
  });

  it('marks the side menu item of the current type (PRD FR-15.10)', () => {
    expect(moduleMenuOf('/bikri/voucher', { type: 'advance' })).toBe('advance');
    expect(moduleMenuOf('/bikri/voucher', { type: 'payment' })).toBe('payment');
    expect(moduleMenuOf('/bikri/voucher', {})).toBe('receipt');
    expect(moduleMenuOf('/bikri/collection')).toBe('receipt');
    expect(moduleMenuOf('/bikri/book')).toBe('bikri-book');
  });
});
