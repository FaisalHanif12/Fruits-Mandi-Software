// The canned data of the garden tests has the shape of the backend's real answers (plan 17c, T2).
// The backend wrote each real answer in its own test (tests/fixtures/gardens_*.json, copied, never
// edited). This compares the keys and kinds at every level, so a test that passes on canned data
// cannot hide a field the real backend does not send, or one it sends and the canned data forgot.
// Then it checks that the screens build the request the backend's own test sent for the same entry,
// and that the numbers the screens show are the backend's (E2E-40 and E2E-40b).

import { describe, expect, it } from 'vitest';
import type { UiError } from '../../src/bindings/UiError';
import {
  ENTRY_FIELD_PATHS,
  GARDEN_FIELD_PATHS,
  buildEntry,
  buildGarden,
  newEntryDraft,
  newGardenDraft,
} from '../../src/features/gardens/draft';
import { LANGUAGES } from '../../src/i18n';
import { errorMessageId, messageVars } from '../../src/shared/api';
import {
  createGardenBooks,
  e40Closed,
  e40Garden,
  firmPartners,
  gardenPartners,
  gardenRow,
} from '../helpers/gardens-data';
import * as real from '../helpers/gardens-real';
import { overviewRow, THREE_PARTNERS } from '../helpers/partners';
import { message } from '../helpers/render';
import { same, sendsTheSame } from '../helpers/shapes';

const GET = { id: 1, history_offset: 0, history_limit: 0 };
const NO_LIST = {
  from: null,
  to: null,
  status: null,
  partner_id: null,
  search: null,
  offset: 0,
  limit: 0,
  sort: null,
  group_by_month: false,
};

describe('the canned data of the garden tests has the real shape', () => {
  it('the defaults, the list and the garden row', () => {
    const books = createGardenBooks();
    expect(same(real.GARDENS_DEFAULTS_FIRST, books.defaults())).toBeNull();
    expect(same(real.GARDENS_DEFAULTS_FROM_LAST_GARDEN, books.defaults())).toBeNull();
    expect(same(real.GARDENS_LIST_ALL, books.list(NO_LIST))).toBeNull();
    expect(same(real.GARDENS_LIST_ALL.rows[0], e40Garden())).toBeNull();
    expect(same(real.GARDENS_LIST_OPEN_PAGED, books.list(NO_LIST))).toBeNull();
    expect(same(real.GARDENS_SET_PARTNERS_THREE, gardenRow())).toBeNull();
    expect(same(real.GARDENS_SUMMARY_OPEN, books.summary())).toBeNull();
    expect(same(real.GARDENS_SUMMARY_AFTER_CLOSE, books.summary())).toBeNull();
  });

  it('a garden with its history and each partner’s money, open, closed, reopened and in loss', () => {
    const books = createGardenBooks();
    const canned = books.get(GET);
    for (const one of [
      real.GARDENS_GET_OPEN,
      real.GARDENS_GET_CLOSED,
      real.GARDENS_GET_REOPENED,
      real.GARDENS_GET_CLOSED_LOSS,
    ]) {
      expect(same(one, canned)).toBeNull();
    }
    expect(same(real.GARDENS_GET_OPEN.history[0], canned.history[0])).toBeNull();
    expect(same(real.GARDENS_GET_OPEN.partner_results[0], canned.partner_results[0])).toBeNull();
  });

  it('the history of a closed garden has the closing row with its credits', () => {
    const books = createGardenBooks({ gardens: [e40Closed()] });
    const closing = real.GARDENS_GET_CLOSED.history.find((row) => row.kind === 'close');
    expect(closing).toBeDefined();
    expect(closing?.credits.length).toBe(2);
    const canned = books.get(GET).history;
    expect(canned.length).toBeGreaterThan(0);
  });

  it('making a garden: the preview, the save with and without a first installment', () => {
    const books = createGardenBooks();
    const request = real.GARDENS_CREATE_E2E40_REQUEST;
    expect(
      same(real.GARDENS_CREATE_PREVIEW_FIRST_INSTALLMENT, books.createPreview(request)),
    ).toBeNull();
    expect(same(real.GARDENS_CREATE_E2E40, books.create(request))).toBeNull();
    expect(
      same(
        real.GARDENS_CREATE_NO_INSTALLMENT,
        books.create(real.GARDENS_CREATE_NO_INSTALLMENT_REQUEST),
      ),
    ).toBeNull();
    expect(real.GARDENS_CREATE_NO_INSTALLMENT.first_entry_id).toBeNull();
  });

  it('an installment, a cost and a sale in another shop: preview and save', () => {
    const books = createGardenBooks();
    expect(
      same(
        real.GARDENS_ENTRY_PREVIEW_INSTALLMENT,
        books.entryPreview(real.GARDENS_ENTRY_PREVIEW_INSTALLMENT_REQUEST),
      ),
    ).toBeNull();
    expect(
      same(
        real.GARDENS_ENTRY_PREVIEW_OTHER_SALE,
        books.entryPreview(real.GARDENS_ENTRY_PREVIEW_OTHER_SALE_REQUEST),
      ),
    ).toBeNull();
    expect(
      same(
        real.GARDENS_ENTRY_PREVIEW_CASH_BELOW_ZERO,
        books.entryPreview(real.GARDENS_ENTRY_PREVIEW_CASH_BELOW_ZERO_REQUEST),
      ),
    ).toBeNull();
    for (const [answer, request] of [
      [real.GARDENS_ENTRY_SAVE_INSTALLMENT, real.GARDENS_ENTRY_SAVE_INSTALLMENT_REQUEST],
      [real.GARDENS_ENTRY_SAVE_COST, real.GARDENS_ENTRY_SAVE_COST_REQUEST],
      [real.GARDENS_ENTRY_SAVE_OTHER_SALE, real.GARDENS_ENTRY_SAVE_OTHER_SALE_REQUEST],
    ] as const) {
      expect(same(answer, books.entrySave(request))).toBeNull();
    }
  });

  it('the cancel, the closing, the closing in loss and the undo', () => {
    // The backend's test closes garden 2, which has nothing in its account, for "nothing to move".
    const books = createGardenBooks({ gardens: [e40Garden(), { ...e40Garden(), id: 2 }] });
    expect(
      same(
        real.GARDENS_ENTRY_CANCEL_COST,
        books.entryCancel(real.GARDENS_ENTRY_CANCEL_COST_REQUEST),
      ),
    ).toBeNull();
    for (const [answer, request] of [
      [real.GARDENS_CLOSE_PREVIEW_E2E40, real.GARDENS_CLOSE_PREVIEW_E2E40_REQUEST],
      [real.GARDENS_CLOSE_PREVIEW_LOSS, real.GARDENS_CLOSE_PREVIEW_LOSS_REQUEST],
      [
        real.GARDENS_CLOSE_PREVIEW_NOTHING_TO_MOVE,
        real.GARDENS_CLOSE_PREVIEW_NOTHING_TO_MOVE_REQUEST,
      ],
    ] as const) {
      expect(same(answer, books.closePreview(request))).toBeNull();
    }
    expect(
      same(real.GARDENS_CLOSE_E2E40, books.close(real.GARDENS_CLOSE_E2E40_REQUEST)),
    ).toBeNull();
    expect(same(real.GARDENS_CLOSE_LOSS, books.close(real.GARDENS_CLOSE_LOSS_REQUEST))).toBeNull();
    expect(
      same(real.GARDENS_REOPEN_E2E40, books.reopen(real.GARDENS_REOPEN_E2E40_REQUEST)),
    ).toBeNull();
  });

  it('a closing with nothing to move posts nothing and still has its numbers', () => {
    const numbers = real.GARDENS_CLOSE_PREVIEW_NOTHING_TO_MOVE.numbers;
    expect(numbers?.posts_entry).toBe(false);
    expect(numbers?.balance_paisa).toBe(0);
    expect(real.GARDENS_CLOSE_PREVIEW_E2E40.numbers?.posts_entry).toBe(true);
  });

  it('the partner overview cards and the statement lines that come from a garden', () => {
    const [a] = THREE_PARTNERS;
    if (a === undefined) throw new Error('no partner');
    const card = real.PARTNERS_OVERVIEW_WITH_GARDEN_CARDS[0]?.open_ventures[0];
    expect(card).toBeDefined();
    const canned = overviewRow(a, {
      open_ventures: [
        {
          venture_id: 1,
          kind: 'garden',
          number: 1,
          code: '0080',
          name_ur: 'شاہ پور باغ',
          name_en: 'Shahpur Garden',
          share_bps: 6000,
          paid_shop_paisa: 0 as never,
          paid_pocket_paisa: 0 as never,
          money_back_paisa: 0 as never,
          balance_paisa: 0 as never,
        },
      ],
    });
    expect(same(real.PARTNERS_OVERVIEW_WITH_GARDEN_CARDS[0], canned)).toBeNull();
    expect(same(real.PARTNERS_OVERVIEW_AFTER_GARDEN_CLOSE[0], canned)).toBeNull();
    // The garden statement: a part paid from the shop and the closing, each with its numbers.
    const lines = real.PARTNERS_STATEMENT_GARDEN_ALIF.lines.filter(
      (line) => line.heading === 'garden',
    );
    expect(lines.map((line) => line.detail)).toEqual([
      'garden_paid',
      'garden_paid',
      'garden_paid',
      'garden_closed',
    ]);
  });

  it('every refusal has the shape of the others', () => {
    const refusals: UiError[] = [
      real.GARDENS_CREATE_SHARES_NOT_100,
      real.GARDENS_ENTRY_SAVE_CASH_BELOW_ZERO,
      real.GARDENS_ENTRY_SAVE_DATE_BEFORE_GARDEN,
      real.GARDENS_ENTRY_SAVE_GARDEN_CLOSED,
      real.GARDENS_ENTRY_CANCEL_GARDEN_CLOSED,
      real.GARDENS_CLOSE_BEFORE_LAST_ENTRY,
      real.GARDENS_REOPEN_NOT_CLOSED,
      real.GARDENS_SET_PARTNERS_LOCKED,
      real.GARDENS_SET_PARTNERS_NOT_100,
    ];
    const plain = (error: UiError) => ({
      ...error,
      errors: error.errors.map((e) => ({ ...e, values: {}, field: null })),
    });
    for (const one of refusals) {
      expect(one.ok).toBe(false);
      expect(one.errors.length).toBeGreaterThan(0);
      expect(typeof one.ref).toBe('string');
      expect(same(plain(real.GARDENS_CREATE_SHARES_NOT_100), plain(one))).toBeNull();
    }
  });
});

describe('what the backend sends is what the screens show', () => {
  it('E2E-40: the garden after its entries has paid 55,000 (33,000 shop, 22,000 pocket), money in 66,400, balance 11,400', () => {
    const row = real.GARDENS_GET_OPEN.row;
    expect(row.rent_paisa).toBe(5_000_000);
    expect(row.paid_paisa).toBe(5_500_000);
    expect(row.paid_shop_paisa).toBe(3_300_000);
    expect(row.paid_pocket_paisa).toBe(2_200_000);
    expect(row.rent_due_paisa).toBe(0);
    expect(row.money_in_paisa).toBe(6_640_000);
    expect(row.other_shop_paisa).toBe(1_000_000);
    expect(row.balance_paisa).toBe(1_140_000);
    expect(row.balance_working.message_id).toBe('working-garden-balance');
  });

  it('E2E-40: the history runs oldest first with the running balance after each row', () => {
    const rows = real.GARDENS_GET_OPEN.history;
    expect(rows.map((row) => row.kind)).toEqual([
      'installment',
      'installment',
      'cost',
      'sale',
      'other_sale',
    ]);
    expect(rows.map((row) => row.running_balance_paisa)).toEqual([
      -3_000_000, -5_000_000, -5_500_000, 140_000, 1_140_000,
    ]);
    // The sale in our shop is a row of the history and cannot be cancelled here.
    expect(rows[3]?.can_cancel).toBe(false);
    expect(rows[3]?.entry_id).toBeNull();
  });

  it('E2E-40: the closing sends 66,400 to the partners as 39,840 and 26,560, and the garden account is 0', () => {
    const numbers = real.GARDENS_CLOSE_PREVIEW_E2E40.numbers;
    expect(numbers?.balance_paisa).toBe(6_640_000);
    expect(numbers?.total_cost_paisa).toBe(5_500_000);
    expect(numbers?.result_paisa).toBe(1_140_000);
    expect(numbers?.credits.map((part) => part.amount_paisa)).toEqual([3_984_000, 2_656_000]);
    expect(numbers?.partner_results.map((part) => part.net_paisa)).toEqual([684_000, 456_000]);
    expect(real.GARDENS_CLOSE_E2E40.row.status).toBe('closed');
    expect(real.GARDENS_CLOSE_E2E40.row.account_balance_paisa).toBe(0);
  });

  it('E2E-40b: a loss closes 10,000 into 6,000 and 4,000, and the result is minus 45,000', () => {
    const numbers = real.GARDENS_CLOSE_PREVIEW_LOSS.numbers;
    expect(numbers?.result_paisa).toBe(-4_500_000);
    expect(numbers?.credits.map((part) => part.amount_paisa)).toEqual([600_000, 400_000]);
  });

  it('the real refusals point at the fields the screens show them at', () => {
    const code = (error: UiError) => error.errors[0]?.code;
    const field = (error: UiError) => error.errors[0]?.field;
    expect(code(real.GARDENS_ENTRY_SAVE_DATE_BEFORE_GARDEN)).toBe('ERR_DATE_BEFORE_GARDEN');
    expect(field(real.GARDENS_ENTRY_SAVE_DATE_BEFORE_GARDEN)).toBe('date');
    expect(ENTRY_FIELD_PATHS.installment).toContain('date');
    expect(code(real.GARDENS_CLOSE_BEFORE_LAST_ENTRY)).toBe('ERR_CLOSE_BEFORE_LAST_ENTRY');
    expect(field(real.GARDENS_CLOSE_BEFORE_LAST_ENTRY)).toBe('date');
    expect(code(real.GARDENS_ENTRY_SAVE_GARDEN_CLOSED)).toBe('ERR_GARDEN_CLOSED');
    expect(field(real.GARDENS_ENTRY_SAVE_GARDEN_CLOSED)).toBe('garden_id');
    expect(code(real.GARDENS_CREATE_SHARES_NOT_100)).toBe('ERR_SHARES_NOT_100');
    expect(field(real.GARDENS_CREATE_SHARES_NOT_100)).toBe('partners');
    expect(GARDEN_FIELD_PATHS).toContain('partners');
    // The cash question points at no field: the screen turns it into its box (PRD FR-6.7).
    expect(code(real.GARDENS_ENTRY_SAVE_CASH_BELOW_ZERO)).toBe('ERR_CASH_BELOW_ZERO_NOT_CONFIRMED');
    expect(field(real.GARDENS_ENTRY_SAVE_CASH_BELOW_ZERO)).toBeNull();
  });
});

describe('every refusal the backend really sent reads as words in both languages', () => {
  const refusals: [string, UiError][] = [
    ['garden create, shares 90%', real.GARDENS_CREATE_SHARES_NOT_100],
    ['entry before the garden’s day', real.GARDENS_ENTRY_SAVE_DATE_BEFORE_GARDEN],
    ['entry on a closed garden', real.GARDENS_ENTRY_SAVE_GARDEN_CLOSED],
    ['cancel on a closed garden', real.GARDENS_ENTRY_CANCEL_GARDEN_CLOSED],
    ['close before the last entry', real.GARDENS_CLOSE_BEFORE_LAST_ENTRY],
    ['undo of an open garden', real.GARDENS_REOPEN_NOT_CLOSED],
    ['shares of a garden with entries', real.GARDENS_SET_PARTNERS_LOCKED],
    ['entry that takes cash below zero', real.GARDENS_ENTRY_SAVE_CASH_BELOW_ZERO],
  ];

  it.each(LANGUAGES)('%s: no placeholder is left open and the real values show', (language) => {
    const texts = refusals.map(([, error]) => {
      const item = error.errors[0];
      if (item === undefined) throw new Error('refusal without an error');
      return message(
        language,
        errorMessageId(item.code),
        messageVars(item.values, 'international'),
      );
    });
    for (const text of texts) {
      expect(text).not.toMatch(/[{}$]/);
      expect(text).not.toContain('undefined');
    }
    // 9000 bps is 90%; the dates are written day first, never as 2026-10-10.
    expect(texts[0]).toContain('90');
    expect(texts[1]).toContain('10-10-2026');
    expect(texts[4]).toContain('12-10-2026');
    expect(texts[4]).not.toContain('2026-10-12');
    // Cash after the entry: −3,700,000 paisa is written −37,000 rupees, never as paisa.
    expect(texts[7]).toContain('37,000');
  });
});

describe('the screens build the request the backend tested', () => {
  const partners = gardenPartners();

  /** The parts the screen sends are one row for every partner; the backend's test sent none. */
  const withoutParts = (input: object) =>
    Object.fromEntries(Object.entries(input).filter(([key]) => key !== 'parts'));

  it('a garden of A 60% from the shop and B 40% from his pocket, with a first installment', () => {
    const request = real.GARDENS_CREATE_E2E40_REQUEST;
    const draft = newGardenDraft({ partners: firmPartners() }, '');
    draft.date = request.date;
    draft.names = {
      urdu: request.name_ur,
      english: request.name_en,
      spelling: { urdu: [], english: [], confirmed: request.confirmed_rules_words },
    };
    draft.item = { id: request.item_id ?? 0, code: '', name_ur: '', name_en: '' };
    draft.rent = request.rent;
    draft.paidNow = request.paid_now ?? '';
    draft.paidNowReason = request.paid_now_reason ?? '';
    // The shares and sources the backend's test sent, typed into the rows as a user would.
    draft.partners = draft.partners.map((row) => {
      const sent = request.partners.find((one) => one.partner_id === row.partner.id);
      return {
        ...row,
        share: sent?.share === '0' || sent === undefined ? '' : sent.share,
        source: sent?.source ?? 'shop',
      };
    });
    const built = buildGarden(draft, false);
    // The backend's test sent only the two partners in it; the screen sends the third with 0.
    expect(built.partners).toEqual([
      ...request.partners,
      { partner_id: 3, share: '0', source: 'shop' },
    ]);
    expect(sendsTheSame({ ...built, partners: request.partners }, request)).toBeNull();
  });

  it('a garden without a first installment sends no amount and no reason', () => {
    const request = real.GARDENS_CREATE_NO_INSTALLMENT_REQUEST;
    const draft = newGardenDraft({ partners: firmPartners() }, '');
    draft.date = request.date;
    draft.names = {
      urdu: request.name_ur,
      english: request.name_en,
      spelling: { urdu: [], english: [], confirmed: request.confirmed_rules_words },
    };
    draft.item = { id: request.item_id ?? 0, code: '', name_ur: '', name_en: '' };
    draft.rent = request.rent;
    const built = buildGarden(draft, false);
    expect(built.paid_now).toBeNull();
    expect(built.paid_now_reason).toBeNull();
  });

  it('an installment, a cost and a sale in another shop: the fields as typed', () => {
    const install = real.GARDENS_ENTRY_SAVE_INSTALLMENT_REQUEST;
    const draft = newEntryDraft('installment', '');
    draft.date = install.date;
    if (install.type === 'other_sale') throw new Error('not an installment');
    draft.amount = install.amount;
    draft.reason = install.reason;
    const sent = buildEntry(draft, install.garden_id ?? 0, partners, false);
    expect(sendsTheSame(withoutParts(sent), withoutParts(install))).toBeNull();
    if (sent.type === 'other_sale') throw new Error('not an installment');
    expect(sent.parts).toHaveLength(2);

    const cost = real.GARDENS_ENTRY_SAVE_COST_REQUEST;
    if (cost.type === 'other_sale') throw new Error('not a cost');
    const costDraft = newEntryDraft('cost', '');
    costDraft.date = cost.date;
    costDraft.amount = cost.amount;
    costDraft.reason = cost.reason;
    expect(
      sendsTheSame(
        withoutParts(buildEntry(costDraft, cost.garden_id ?? 0, partners, false)),
        withoutParts(cost),
      ),
    ).toBeNull();

    const sale = real.GARDENS_ENTRY_SAVE_OTHER_SALE_REQUEST;
    if (sale.type !== 'other_sale') throw new Error('not a sale');
    const saleDraft = newEntryDraft('other_sale', '');
    saleDraft.date = sale.date;
    saleDraft.amount = sale.amount;
    saleDraft.shop = sale.shop ?? '';
    saleDraft.qty = sale.qty;
    expect(
      sendsTheSame(buildEntry(saleDraft, sale.garden_id ?? 0, partners, false), sale),
    ).toBeNull();
  });

  it('the cash question is asked again with the answer: the same request with the flag on', () => {
    const request = real.GARDENS_ENTRY_SAVE_CASH_BELOW_ZERO_REQUEST;
    if (request.type === 'other_sale') throw new Error('not a payment');
    const draft = newEntryDraft('installment', '');
    draft.date = request.date;
    draft.amount = request.amount;
    draft.reason = request.reason;
    const again = buildEntry(draft, request.garden_id ?? 0, partners, true);
    if (again.type === 'other_sale') throw new Error('not a payment');
    expect(again.confirm_cash_below_zero).toBe(true);
    expect(request.confirm_cash_below_zero).toBe(false);
  });
});
