// The answers of the development page /dev/index (plan 17e): the account form and the firm settings,
// for looking at the tag (نشان) field, the similar accounts with their tags, and the contact persons
// and trade mark of the firm in a plain browser tab. dev-bridge.ts asks this table first while the
// address is /dev/index. Flags in the hash: `?screen=account|firm`, `&similar` (two people with the
// same name, one with a tag), `&duplicate` (the one with the tag is a duplicate), `&contacts` (the firm has three contact persons and a trade mark).
// Never part of a build: only the development route imports it.

import type { AccountMatch } from '../../src/shared/accounts';
import { SEARCH_GLOBAL_TAG } from './tag-real';

type Input = Record<string, unknown>;
type Handler = (input: Input) => unknown;

/** True on the development page of this area. */
export function isIndexPage(): boolean {
  return window.location.hash.startsWith('#/dev/index');
}

const SIMILAR: AccountMatch[] = [
  {
    id: 61,
    code: '00260',
    name_ur: 'عاشق ظفر',
    name_en: 'Ashiq Zafar',
    tag: 'K',
    father_or_firm_ur: 'ظفر',
    father_or_firm_en: 'Zafar',
    city_ur: 'کوٹ رادھا کشن',
    city_en: 'Kot Radha Kishan',
    phone1: '03001234567',
    account_type: 'buyer',
    is_duplicate: false,
  },
  {
    id: 62,
    code: '00261',
    name_ur: 'عاشق ظفر',
    name_en: 'Ashiq Zafar',
    father_or_firm_ur: 'ظفر',
    father_or_firm_en: 'Zafar',
    city_ur: 'کوٹ رادھا کشن',
    city_en: 'Kot Radha Kishan',
    phone1: '03001234567',
    account_type: 'buyer',
    is_duplicate: false,
  },
];

const flag = (name: string): boolean =>
  new URLSearchParams(window.location.hash.split('?')[1] ?? '').has(name);

export const indexHandlers: Record<string, Handler> = {
  // A پختہ party's code is the next of the پختہ series, from 0032 (plan 47).
  accounts_next_code: (input) => (input['is_pukhta_party'] === true ? '0032' : '00262'),
  // Plan 44 (PRD FR-10.11): `&closedopen` is a closed day with today open, `&closednight` a closed day
  // with today closed too. An opening amount on it is refused; an account without one is saved.
  accounts_save: (input) => {
    const wantsOpening = [input['opening_dr'], input['opening_cr']].some(
      (amount) => typeof amount === 'string' && amount.trim() !== '',
    );
    const open = new URLSearchParams(window.location.hash.split('?')[1] ?? '');
    if (wantsOpening && (open.has('closedopen') || open.has('closednight'))) {
      throw Object.assign(new Error('ERR_DATE_CLOSED'), {
        ok: false,
        errors: [
          {
            code: 'ERR_DATE_CLOSED',
            field: 'opening_date',
            values: {
              closed_up_to: '30-10-2026',
              today: '31-10-2026',
              today_open: open.has('closednight') ? 0 : 1,
            },
          },
        ],
        ref: 'E-DEV',
      });
    }
    throw Object.assign(new Error('ERR_NOT_IMPLEMENTED'), { ok: false, errors: [], ref: '' });
  },
  // `&duplicate`: the first one has the name, the tag and the phone typed (the backend's mark).
  accounts_similar: () =>
    flag('duplicate')
      ? SIMILAR.map((match, index) => ({ ...match, is_duplicate: index === 0 }))
      : flag('similar')
        ? SIMILAR
        : [],
  coa_heads_list: () => [],
  coa_controls_list: () => [],
  search_masters: () => [],
  search_recent: () => [],
  search_touch: () => null,
  // The global search: the backend's own real answer, Akram Toll Tax with the tag K.
  search_global: () => SEARCH_GLOBAL_TAG,
  settings_set_firm: (input) => input,
};

/** The firm of the settings screen: with three people (two owners, the munshi) and a trade mark, or none. */
export function devFirm(withContacts: boolean) {
  return {
    name_ur: 'ماہر عدنان اینڈ برادرز',
    name_en: 'Maher Adnan and Brothers',
    mandi_ur: 'نیو سبزی منڈی، کوٹ رادھا کشن',
    mandi_en: 'New Sabzi Mandi, Kot Radha Kishan',
    address: 'Kot Radha Kishan',
    phone: '0300-1112233',
    firm_contacts: withContacts
      ? [
          {
            name_ur: 'ماہر عدنان',
            name_en: 'Maher Adnan',
            role: 'owner' as const,
            phone: '0300-1112233',
          },
          {
            name_ur: 'محمد اکرم',
            name_en: 'Muhammad Akram',
            role: 'owner' as const,
            phone: '0321-5556677',
          },
          {
            name_ur: 'آصف بھٹی',
            name_en: 'Asif Bhatti',
            role: 'munshi' as const,
            phone: '0300-4058860',
          },
        ]
      : [],
    trade_mark: withContacts ? 'RH' : '',
  };
}
