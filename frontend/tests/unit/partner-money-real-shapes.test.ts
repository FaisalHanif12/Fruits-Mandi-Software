// The canned data of the capital, partner overview, partner statement, الگ کھاتہ and جنرل tests has
// the shape of the backend's real answers (plan 17, T4). The backend wrote each real answer in its
// own test (tests/fixtures/*.json, copied, never edited). This compares the keys and kinds at every
// level, so a test that passes on canned data cannot hide a field the real backend does not send,
// or one it sends and the canned data forgot. Then it checks that the screens build the request the
// backend's own test sent for the same entry, and that the E2E-34 statement of partner الف (the
// lines -4,160, +480 and +3,384) has the keys the backend really puts on a trip line.

import { describe, expect, it } from 'vitest';
import { buildKhata, newKhataDraft } from '../../src/features/cashbook/khata-draft';
import { buildGeneral, buildGeneralPreview } from '../../src/features/cashbook/general-draft';
import { inputOf } from '../../src/features/cashbook/entry-support';
import { buildCapital, newCapitalDraft } from '../../src/features/partners/capital-draft';
import { CommandError } from '../../src/shared/api';
import { FAMILY_KHATA, createCashbookBooks } from '../helpers/cashbook-books';
import { createPartnerMoneyBooks, GENERAL_ACCOUNTS } from '../helpers/partner-money-books';
import { THREE_PARTNERS, overviewRow } from '../helpers/partners';
import * as real from '../helpers/partner-money-real';
import { same, sendsTheSame } from '../helpers/shapes';

/** The refusal a fake backend throws, as the plain `{ ok, errors, ref }` of the wire. */
function refusalOf(make: () => unknown) {
  try {
    make();
  } catch (error) {
    if (error instanceof CommandError) return error.uiError;
    throw error;
  }
  throw new Error('nothing was refused');
}

function books() {
  const cash = createCashbookBooks({ cash: 90_000 });
  return { cash, money: createPartnerMoneyBooks(cash, { accounts: GENERAL_ACCOUNTS }) };
}

const capitalDraft = (type: 'capital_in' | 'capital_out', amount: string, narration = '') => ({
  ...newCapitalDraft(type, '2026-10-11', '2'),
  amount,
  narration,
});

describe('the canned capital data has the real shape', () => {
  it('the history, the preview with its warning, the save and the cash refusal', () => {
    const { money } = books();
    const saved = money.saveCapital(
      'capital_in',
      buildCapital(capitalDraft('capital_in', '50,000', 'start'), false),
    );
    expect(same(real.SAVED_CAPITAL_IN, saved)).toBeNull();
    expect(
      same(
        real.CAPITAL_HISTORY,
        money.capitalHistory({
          partner_id: 2,
          offset: 0,
          limit: 0,
          sort: null,
          group_by_month: false,
        }),
      ),
    ).toBeNull();
    const preview = money.capitalPreview(
      buildCapital(capitalDraft('capital_out', '60,000'), false),
    );
    expect(same(real.PREVIEW_CAPITAL_OUT, preview)).toBeNull();
    // The warning is the real one: the balance and the part above it, in paisa.
    expect(real.PREVIEW_CAPITAL_OUT.warnings[0]).toMatchObject({
      code: 'ERR_CAPITAL_EXCEEDS_BALANCE',
      field: 'amount',
    });
    expect(real.PREVIEW_CAPITAL_OUT.errors).toEqual([]);
    const refused = refusalOf(() =>
      money.saveCapital('capital_out', buildCapital(capitalDraft('capital_out', '200,000'), false)),
    );
    expect(same(real.REFUSED_CAPITAL_OUT_CASH, refused)).toBeNull();
  });

  it('the screen sends the request the backend’s own test sent', () => {
    const built = buildCapital(capitalDraft('capital_in', '50,000', 'start'), false);
    expect(sendsTheSame(inputOf(built), real.REQUEST_SAVE_CAPITAL_IN)).toBeNull();
    const out = buildCapital(capitalDraft('capital_out', '120,000'), false);
    expect(sendsTheSame(inputOf(out), real.REQUEST_SAVE_CAPITAL_OUT_CASH)).toBeNull();
    // The preview takes the entry with its type tag, as the generic voucher preview does.
    const previewed = buildCapital(capitalDraft('capital_out', '60,000'), false);
    expect(sendsTheSame(previewed, real.REQUEST_PREVIEW_CAPITAL_OUT.entry)).toBeNull();
    expect(real.REQUEST_PREVIEW_CAPITAL_OUT.replacing_voucher_id).toBeNull();
    expect(Object.keys(real.REQUEST_CAPITAL_HISTORY).sort()).toEqual([
      'limit',
      'offset',
      'partner_id',
    ]);
  });
});

describe('the canned partner overview and statement have the real shape', () => {
  it('a row of the overview carries the five new columns', () => {
    const [a] = THREE_PARTNERS;
    if (a === undefined) throw new Error('no partner');
    expect(same(real.OVERVIEW, [overviewRow(a)])).toBeNull();
    expect(real.OVERVIEW).toHaveLength(3);
  });

  it('the statement of ب and the page of الف', () => {
    const { money } = books();
    expect(
      same(
        real.STATEMENT_B,
        money.statement({
          partner_id: 2,
          from: null,
          to: null,
          offset: 0,
          limit: 0,
          sort: null,
          group_by_month: false,
        }),
      ),
    ).toBeNull();
    expect(
      same(
        real.STATEMENT_A_PAGE,
        money.statement({
          partner_id: 2,
          from: '10-10-2026',
          to: null,
          offset: 0,
          limit: 2,
          sort: null,
          group_by_month: false,
        }),
      ),
    ).toBeNull();
    // The real lines: the keys of every line, the kind of the expense line, and the totals.
    expect(real.STATEMENT_B.lines.map((line) => line.running_paisa)).toEqual([
      118_400, 28_400, 5_028_400,
    ]);
    expect(real.STATEMENT_B.lines[1]?.values).toEqual({ kind: 'shop_expense' });
    expect(real.STATEMENT_B.lines[2]?.values).toEqual({ kind: 'capital_in' });
    expect(real.STATEMENT_B.partner.code).toBe('');
    expect(real.REQUEST_STATEMENT_A_PAGE).toMatchObject({ from: '10-10-2026', limit: 2 });
    expect(real.REQUEST_STATEMENT_B).toMatchObject({ partner_id: 2, from: null, to: null });
  });

  it('the E2E-34 trip lines of partner الف carry the values the backend puts on them', () => {
    const { money } = books();
    const lines = money.statement({
      partner_id: 1,
      from: null,
      to: null,
      offset: 0,
      limit: 0,
      sort: null,
      group_by_month: false,
    }).lines;
    const amounts = lines.map((line) => line.amount_paisa);
    // -4,160, +480 and +3,384 in rupees, then two note lines with no amount.
    expect(amounts.slice(0, 3)).toEqual([-416_000, 48_000, 338_400]);
    expect(amounts.slice(3)).toEqual([null, null]);
    expect(lines.at(2)?.running_paisa).toBe(-29_600);
    // The keys of the backend (statement.rs): trip_credit has qty, sales, cost, result and share.
    expect(Object.keys(lines[2]?.values ?? {}).sort()).toEqual([
      'cost_paisa',
      'qty',
      'result_paisa',
      'sales_paisa',
      'share_bps',
    ]);
    expect(lines[0]?.values).toEqual({ kind: 'purchase' });
    expect(Object.keys(lines[4]?.values ?? {}).sort()).toEqual([
      'cost_paisa',
      'net_paisa',
      'result_paisa',
      'sales_paisa',
      'share_bps',
    ]);
  });
});

describe('the canned الگ کھاتہ data has the real shape', () => {
  it('the new khata, the two saves, the statement and the mistakes', () => {
    const { cash } = books();
    const created = cash.createKhata({ name_ur: 'x', name_en: 'x', note: 'family' });
    expect(same(real.KHATA_CREATED, created)).toBeNull();
    const out = buildKhata(
      {
        ...newKhataDraft('khata_out', '2026-10-10'),
        khata: { ...FAMILY_KHATA },
        amount: '10,000',
        reason: 'r',
      },
      false,
    );
    expect(same(real.SAVED_KHATA_OUT, cash.saveLentOrKhata(out))).toBeNull();
    const back = buildKhata(
      {
        ...newKhataDraft('khata_in', '2026-10-11'),
        khata: { ...FAMILY_KHATA },
        amount: '4,000',
        reason: 'r',
      },
      false,
    );
    expect(same(real.SAVED_KHATA_IN, cash.saveLentOrKhata(back))).toBeNull();
    expect(
      same(
        real.KHATA_STATEMENT,
        cash.khataStatement({
          khata_account_id: FAMILY_KHATA.id,
          from: null,
          to: null,
          offset: 0,
          limit: 0,
          sort: null,
          group_by_month: false,
        }),
      ),
    ).toBeNull();
    const refused = refusalOf(() =>
      cash.saveLentOrKhata(
        buildKhata({ ...newKhataDraft('khata_out', '2026-10-10'), amount: '10.5' }, false),
      ),
    );
    expect(same(real.REFUSED_KHATA_MISTAKES, refused)).toBeNull();
    // Out is minus, in is plus, from the khata’s side.
    expect(real.KHATA_STATEMENT.rows.map((row) => row.amount_paisa)).toEqual([-1_000_000, 400_000]);
  });

  it('the screen sends the request the backend’s own test sent', () => {
    const khata = { ...FAMILY_KHATA, id: 32 };
    const out = buildKhata(
      {
        ...newKhataDraft('khata_out', '2026-10-10'),
        khata,
        amount: '10,000',
        reason: 'خاندان کا خرچ',
      },
      false,
    );
    expect(sendsTheSame(inputOf(out), real.REQUEST_SAVE_KHATA_OUT)).toBeNull();
    const back = buildKhata(
      { ...newKhataDraft('khata_in', '2026-10-11'), khata, amount: '4,000', reason: 'واپس آئے' },
      false,
    );
    expect(sendsTheSame(inputOf(back), real.REQUEST_SAVE_KHATA_IN)).toBeNull();
    const mistakes = buildKhata(
      { ...newKhataDraft('khata_out', '2026-10-10'), amount: '10.5' },
      false,
    );
    expect(sendsTheSame(inputOf(mistakes), real.REQUEST_SAVE_KHATA_OUT_MISTAKES)).toBeNull();
    expect(Object.keys(real.REQUEST_KHATA_STATEMENT).sort()).toEqual([
      'from',
      'khata_account_id',
      'limit',
      'offset',
      'to',
    ]);
  });
});

describe('the canned جنرل data has the real shape', () => {
  const line = (account_id: number, dr: string, cr: string, narration: string | null) => ({
    account_id,
    person_key: null,
    dr,
    cr,
    narration,
  });

  it('the previews and the save', () => {
    const { money } = books();
    const balanced = [line(21, '1,000', '', 'fix'), line(31, '', '1,000', null)];
    expect(
      same(
        real.GENERAL_BALANCED,
        money.generalPreview(buildGeneralPreview('11-10-2026', balanced)),
      ),
    ).toBeNull();
    const unbalanced = [line(21, '1,000', '', null), line(1, '', '900', null)];
    expect(
      same(real.GENERAL_UNBALANCED, money.generalPreview(buildGeneralPreview('', unbalanced))),
    ).toBeNull();
    expect(
      same(
        real.SAVED_GENERAL,
        money.generalSave(buildGeneral('11-10-2026', 'correction', balanced, false)),
      ),
    ).toBeNull();
    const refused = refusalOf(() =>
      money.generalSave(buildGeneral('11-10-2026', '', unbalanced, false)),
    );
    expect(same(real.REFUSED_GENERAL_NOT_BALANCED, refused)).toBeNull();
    // The real unbalanced preview cannot be saved and says why at `lines`.
    expect(real.GENERAL_UNBALANCED.can_save).toBe(false);
    expect(real.GENERAL_UNBALANCED.errors[0]).toMatchObject({
      code: 'ERR_GENERAL_NOT_BALANCED',
      field: 'lines',
    });
  });

  it('the screen sends the request the backend’s own test sent', () => {
    const balanced = [line(20, '1,000', '', 'fix'), line(18, '', '1,000', null)];
    expect(
      sendsTheSame(buildGeneralPreview('11-10-2026', balanced), real.REQUEST_GENERAL_BALANCED),
    ).toBeNull();
    const unbalanced = [line(20, '1,000', '', null), line(1, '', '900', null)];
    expect(
      sendsTheSame(buildGeneralPreview('', unbalanced), real.REQUEST_GENERAL_UNBALANCED),
    ).toBeNull();
    expect(real.REQUEST_GENERAL_UNBALANCED.date).toBeNull();
    expect(
      sendsTheSame(
        buildGeneral('11-10-2026', 'correction', balanced, false),
        real.REQUEST_SAVE_GENERAL,
      ),
    ).toBeNull();
    expect(
      sendsTheSame(
        buildGeneral('11-10-2026', '', unbalanced, false),
        real.REQUEST_SAVE_GENERAL_NOT_BALANCED,
      ),
    ).toBeNull();
  });
});
