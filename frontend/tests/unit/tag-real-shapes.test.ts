// The canned data of the plan 17e tests has the shape of the backend's real answers (T2). The
// backend wrote each real answer in its own test (tests/fixtures/*.json and the `insta` snapshots of
// `mandi-reports`, copied, never edited). This compares the keys and kinds at every level, so a test
// that passes on canned data cannot hide a field the real backend does not send, or one it sends and
// the canned data forgot. Then it checks the screen's own requests against the backend's, that every
// word the answers need exists in both languages, and the numbers of the worked day.

import { describe, expect, it } from 'vitest';
import { accountInputOf, pickedOf } from '../../src/shared/accounts';
import { LANGUAGES } from '../../src/i18n';
import { displayName } from '../../src/shared/i18n';
import { account, balance, cardOf } from '../helpers/accounts';
import { settings } from '../helpers/app';
import {
  collectionPrintData,
  frameOf,
  printDataFor,
  voucherPrintData,
} from '../helpers/print-data';
import {
  REAL_COLLECTION_SNAPSHOTS,
  REAL_SNAPSHOTS,
  realCollectionData,
  realPrintData,
} from '../helpers/print-real';
import { message } from '../helpers/render';
import { same, sendsTheSame } from '../helpers/shapes';
import * as real from '../helpers/tag-real';

/** Akram Toll Tax K, as the backend's test made him. */
const AKRAM = account(18, 'buyer', '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  tag: 'K',
  phone1: '03001234567',
});

describe('the canned data has the real shape: accounts', () => {
  it('an account row saved with a tag', () => {
    expect(same(real.ACCOUNTS_SAVE_TAG, AKRAM)).toBeNull();
    expect(real.ACCOUNTS_SAVE_TAG.tag).toBe('K');
  });

  it('the party card of an account with a tag', () => {
    expect(same(real.PARTY_CARD_TAG, cardOf(AKRAM))).toBeNull();
    expect(real.PARTY_CARD_TAG.tag).toBe('K');
    expect(real.PARTY_CARD_TAG.balance).toEqual(balance(2880, 'dr'));
  });

  it('the settings with contact persons and a trade mark', () => {
    const canned = settings({
      firm: { ...settings().firm, ...real.SETTINGS_GET_CONTACTS.firm },
    });
    expect(same(real.SETTINGS_GET_CONTACTS, canned)).toBeNull();
    expect(real.SETTINGS_GET_CONTACTS.firm.firm_contacts).toHaveLength(2);
    expect(real.SETTINGS_GET_CONTACTS.firm.trade_mark).toBe('RH');
  });
});

describe('the canned data has the real shape: prints', () => {
  it('the header and the footer of a report print, in both languages', () => {
    for (const [answer, language] of [
      [real.PRINT_HEADER_UR, 'ur'],
      [real.PRINT_HEADER_EN, 'en'],
    ] as const) {
      const canned = frameOf(
        language,
        'print-collection-list-title',
        [{ label_id: 'print-fact-date', value: '10-10-2026' }],
        { contacts: true, tradeMark: 'RH' },
      );
      expect(same(answer.header, canned.header)).toBeNull();
      expect(same(answer.footer, canned.footer)).toBeNull();
      expect(answer.language).toBe(language);
      expect(answer.urdu_digits).toBe(false);
    }
  });

  it('the collection list, with its groups, rows and totals', () => {
    for (const [answer, language] of [
      [real.COLLECTION_UR, 'ur'],
      [real.COLLECTION_EN, 'en'],
    ] as const) {
      const canned = collectionPrintData({
        language,
        contacts: true,
        tradeMark: 'RH',
        tagged: true,
      });
      expect(same(answer, canned)).toBeNull();
      expect(same(answer.groups[0], canned.groups[0])).toBeNull();
      expect(same(answer.groups[0]?.rows[0], canned.groups[0]?.rows[0])).toBeNull();
      expect(same(answer.total, canned.total)).toBeNull();
    }
    // An empty day has the same shape, with no group.
    expect(real.COLLECTION_EMPTY.groups).toEqual([]);
    expect(same(real.COLLECTION_EMPTY, collectionPrintData({ language: 'ur' }))).toBeNull();
  });

  it('the real snapshots of the collection list, with the contacts and filters and with Urdu digits', () => {
    for (const snapshot of REAL_COLLECTION_SNAPSHOTS) {
      const answer = realCollectionData(snapshot);
      const canned = collectionPrintData({
        language: snapshot.language,
        contacts: true,
        tradeMark: 'RH',
      });
      expect(same(answer, canned), snapshot.name).toBeNull();
      expect(answer.urdu_digits).toBe(snapshot.urduDigits);
    }
  });

  it('the documents of a بکری with a tag: a buyer with a tag, a line with a tag, the short code', () => {
    const options = { contacts: true, tradeMark: 'RH', tagged: true } as const;
    expect(
      same(
        real.BIKRI_TAG_SELLER_BILL_UR,
        printDataFor('seller_bill', { language: 'ur', ...options }),
      ),
    ).toBeNull();
    expect(
      same(
        real.BIKRI_TAG_BUYER_SLIPS_UR,
        printDataFor('buyer_slips', { language: 'ur', ...options }),
      ),
    ).toBeNull();
    expect(
      same(
        real.BIKRI_TAG_BUYER_SLIPS_EN,
        printDataFor('buyer_slips', { language: 'en', ...options }),
      ),
    ).toBeNull();
    expect(real.BIKRI_TAG_SELLER_BILL_UR.pages[0]?.lines[0]?.name_tag).toBe('K');
    expect(real.BIKRI_TAG_BUYER_SLIPS_UR.pages[0]?.party.tag).toBe('K');
    expect(real.BIKRI_TAG_BUYER_SLIPS_UR.pages[0]?.party.short_code).toBe('1');
  });

  it('every real snapshot of a بکری has the short code of a seller and a buyer', () => {
    for (const snapshot of REAL_SNAPSHOTS) {
      const data = realPrintData(snapshot);
      for (const page of data.pages) {
        // A one-time person has no code; a seller or a buyer has both codes.
        expect(page.party.short_code === undefined).toBe(page.party.code === null);
      }
    }
    const receipt = voucherPrintData('receipt', { language: 'ur', tagged: true });
    expect(receipt.pages[0]?.party.short_code).toBe('1');
  });
});

describe("the screen builds the request the backend's own test sent", () => {
  const draftOf = (tag: string) => ({
    id: null,
    version: null,
    account_type: 'buyer' as const,
    control_id: null,
    code: '',
    names: { urdu: 'اکرم ٹول ٹیکس', english: 'Akram Toll Tax' },
    tag,
    father: { urdu: '', english: '' },
    city: null,
    adda: null,
    phone1: '03001234567',
    phone2: '',
    address: '',
    linked_account: null,
    opening_date: '',
    opening_dr: '',
    opening_cr: '',
    opening_is_advance: false,
    credit_allowed: true,
    credit_limit: '',
    default_marka: null,
    collector: null,
    collection_weekday: null,
    is_active: true,
    note: '',
  });

  it('accounts_save with a tag typed in lower case: the screen sends it as typed', () => {
    const input = accountInputOf(draftOf('k'), []);
    expect(
      sendsTheSame(input, {
        ...real.ACCOUNTS_SAVE_TAG_REQUEST,
        name_ur: 'اکرم ٹول ٹیکس',
        name_en: 'Akram Toll Tax',
      }),
    ).toBeNull();
    expect(real.ACCOUNTS_SAVE_TAG_REQUEST.tag).toBe('k');
  });

  it('accounts_save with a tag the backend refuses: the same request, the refusal is at `tag`', () => {
    const input = accountInputOf(draftOf('1A'), []);
    expect(
      sendsTheSame(input, {
        ...real.ACCOUNTS_SAVE_TAG_REFUSED_REQUEST,
        name_ur: 'اکرم ٹول ٹیکس',
        name_en: 'Akram Toll Tax',
      }),
    ).toBeNull();
    expect(real.ACCOUNTS_SAVE_TAG_REFUSED.errors).toEqual([
      { code: 'ERR_TAG_INVALID', field: 'tag', values: { max: 4 } },
    ]);
  });

  it('the picked record keeps the tag the backend sent, in both languages', () => {
    const hit = real.SEARCH_ACCOUNTS_TAG[0];
    if (hit === undefined) throw new Error('no hit');
    const picked = pickedOf(hit);
    expect(displayName(picked, 'ur')).toBe('اکرم ٹول ٹیکس K');
    expect(displayName(picked, 'en')).toBe('Akram Toll Tax K');
  });
});

describe('every word the real answers need exists in both languages', () => {
  it('the labels of the headers, facts, totals and the refusals', () => {
    const ids = new Set<string>();
    for (const data of [real.COLLECTION_UR, real.COLLECTION_EN, real.COLLECTION_EMPTY]) {
      ids.add(data.header.title_id);
      for (const fact of data.header.facts) ids.add(fact.label_id);
      ids.add(data.total.label_id);
      ids.add(data.footer.printed_label_id);
      ids.add(data.footer.page_label_id);
      ids.add(data.footer.of_label_id);
    }
    for (const frame of [real.PRINT_HEADER_UR, real.PRINT_HEADER_EN]) {
      ids.add(frame.header.title_id);
      for (const fact of frame.header.facts) ids.add(fact.label_id);
    }
    for (const id of ids) {
      for (const language of LANGUAGES) expect(() => message(language, id)).not.toThrow();
    }
    for (const id of [
      'err-tag-invalid',
      'err-out-of-range',
      'err-text-too-long',
      'err-spelling-unconfirmed',
    ]) {
      for (const language of LANGUAGES)
        expect(() => message(language, id, { max: 3, min: 0, words: 'x' })).not.toThrow();
    }
  });

  it('the numbers of the worked day are in the collection list: 2,880 + 18,684 = 21,564 as sent', () => {
    expect(real.COLLECTION_UR.groups[0]?.rows.map((row) => row.dues)).toEqual(['2,880', '18,684']);
    expect(real.COLLECTION_UR.groups[0]?.total).toBe('21,564');
    expect(real.COLLECTION_UR.total.amount).toBe('21,564');
    expect(real.COLLECTION_UR.buyer_count).toBe('2');
    expect(real.COLLECTION_UR.groups[0]?.rows[0]?.tag).toBe('K');
  });
});
