// Answers of the development pages for the plan 76 screens that no other stand-in covers (the buyer dues page,
// the page of garden owners and others, the open ventures line of a partner), with the numbers of the plan
// (after Events 1 to 5, 10-10-26: اکرم 2,880 and ارشد 18,684; نواز 15,000 open, شفیق 3,000). Merged into the
// روکڑ stand-in (dev-bridge-rokar.ts) so `/dev/rokar?screen=buyer-dues` and `?screen=others-dues` open them
// in a plain browser tab. Never part of a build: only the development routes import it.

import type { BuyerDuesPage } from '../../src/bindings/BuyerDuesPage';
import type { LentDues } from '../../src/bindings/LentDues';
import type { PartnerVentureCard } from '../../src/bindings/PartnerVentureCard';
import { paisaForTest } from './paisa';

type Input = Record<string, unknown>;

const rupees = (value: number) => paisaForTest(value * 100);

function buyer(id: number, code: string, ur: string, en: string, owes: number, tag: string | null) {
  return {
    party: {
      account_id: id,
      code,
      short_code: code.slice(3),
      name_ur: ur,
      name_en: en,
      tag,
      person_key: null,
    },
    city: { ur: 'کوٹ رادھا کشن', en: 'Kot Radha Kishan' },
    balance: { amount_paisa: rupees(owes), side: 'dr' as const },
    last_change_date: '2026-10-10',
  };
}

const BUYER_DUES: BuyerDuesPage = {
  rows: [
    buyer(21, '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', 2_880, null),
    buyer(22, '0021', 'ارشد ملتانی', 'Arshad Multani', 18_684, 'K'),
  ],
  total_count: 2,
  total_paisa: rupees(21_564),
};

const NAWAZ = { id: 91, code: '01030091', name_ur: 'نواز', name_en: 'Nawaz' };
const SHAFIQ = { id: 92, code: '01030092', name_ur: 'شفیق', name_en: 'Shafiq' };

const LENT_DUES: LentDues = {
  month: '2026-10',
  partner_locked: false,
  people_total_count: 2,
  rows: [
    {
      kind: 'given',
      entry_date: '2026-10-01',
      person: NAWAZ,
      person_type: 'garden_owner',
      link: { kind: 'partner', partner_id: 1 },
      voucher_id: 301,
      number: 4,
      reason: 'اگلے سیزن کا باغ',
      amount_paisa: rupees(20_000),
    },
    {
      kind: 'returned',
      entry_date: '2026-10-10',
      person: NAWAZ,
      person_type: 'garden_owner',
      link: { kind: 'partner', partner_id: 1 },
      voucher_id: 302,
      number: 2,
      reason: '',
      amount_paisa: rupees(5_000),
    },
    {
      kind: 'given',
      entry_date: '2026-10-05',
      person: SHAFIQ,
      person_type: 'trip_person',
      link: { kind: 'shop', partner_id: null },
      voucher_id: 303,
      number: 5,
      reason: 'سفر کا خرچ',
      amount_paisa: rupees(3_000),
    },
  ],
  total_count: 3,
  totals: {
    given_paisa: rupees(23_000),
    returned_paisa: rupees(5_000),
    open_paisa: rupees(18_000),
    open_people_count: 2,
  },
  people: [
    {
      person: NAWAZ,
      person_type: 'garden_owner',
      link: { kind: 'partner', partner_id: 1 },
      lent_paisa: rupees(20_000),
      returned_paisa: rupees(5_000),
      open_paisa: rupees(15_000),
    },
    {
      person: SHAFIQ,
      person_type: 'trip_person',
      link: { kind: 'shop', partner_id: null },
      lent_paisa: rupees(3_000),
      returned_paisa: rupees(0),
      open_paisa: rupees(3_000),
    },
  ],
};

const VENTURES: PartnerVentureCard[] = [
  {
    venture_id: 1,
    kind: 'garden',
    number: 1,
    code: '0080',
    name_ur: 'شاہ پور باغ',
    name_en: 'Shahpur Garden',
    share_bps: 6000,
    paid_shop_paisa: rupees(33_000),
    paid_pocket_paisa: rupees(0),
    money_back_paisa: rupees(0),
    balance_paisa: rupees(11_400),
  },
  {
    venture_id: 2,
    kind: 'trip',
    number: 1,
    code: '0070',
    name_ur: 'سفر ۱',
    name_en: 'Trip 1',
    share_bps: 4000,
    paid_shop_paisa: rupees(4_160),
    paid_pocket_paisa: rupees(0),
    money_back_paisa: rupees(3_384),
    balance_paisa: rupees(-776),
  },
];

export const plan76Handlers: Record<string, (input: Input) => unknown> = {
  dashboard_buyer_dues: () => BUYER_DUES,
  lent_dues: () => LENT_DUES,
  partners_open_ventures: () => VENTURES,
};
