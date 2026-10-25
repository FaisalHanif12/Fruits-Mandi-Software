// The answers of the development page /dev/index (plan 17e): the account form and the firm settings,
// for looking at the tag (نشان) field, the similar accounts with their tags, and the contact persons
// and trade mark of the firm in a plain browser tab. dev-bridge.ts asks this table first while the
// address is /dev/index. Flags in the hash: `?screen=account|firm`, `&similar` (two people with the
// same name, one with a tag), `&contacts` (the firm has three contact persons and a trade mark).
// Never part of a build: only the development route imports it.

import type { AccountMatch } from '../../src/shared/accounts';

type Input = Record<string, unknown>;
type Handler = (input: Input) => unknown;

/** True on the development page of this area. */
export function isIndexPage(): boolean {
  return window.location.hash.startsWith('#/dev/index');
}

const SIMILAR: AccountMatch[] = [
  {
    id: 61,
    code: '03040061',
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
    code: '03040062',
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
  accounts_next_code: () => '03040063',
  accounts_similar: () => (flag('similar') ? SIMILAR : []),
  coa_heads_list: () => [],
  coa_controls_list: () => [],
  search_masters: () => [],
  search_recent: () => [],
  search_touch: () => null,
  settings_set_firm: (input) => input,
};

/** The firm of the settings screen: with three contact persons and a trade mark, or with none. */
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
          { name_ur: 'منشی آصف بھٹی', name_en: 'Munshi Asif Bhatti', phone: '0300-4058860' },
          { name_ur: 'ماہر عدنان', name_en: 'Maher Adnan', phone: '0300-1112233' },
          { name_ur: 'محمد اکرم', name_en: 'Muhammad Akram', phone: '0321-5556677' },
        ]
      : [],
    trade_mark: withContacts ? 'RH' : '',
  };
}
