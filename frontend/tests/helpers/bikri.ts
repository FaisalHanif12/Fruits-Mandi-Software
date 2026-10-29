// Test helper: the commands of the بکری screen answered the way a healthy backend would (plan 12
// contract). The wrappers are mocked in the test file with vi.mock; this sets their answers. The
// numbers are the backend's own, from the Event 1 file it wrote (bikri-data.ts); the screen only
// shows them and works out nothing.

import { vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as draftsApi from '../../src/shared/drafts/api';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { MasterKind } from '../../src/features/masters/types';
import * as searchApi from '../../src/shared/search/api';
import { account, balance, fakeAccountSearch, fakeAccounts, ref } from './accounts';
import { IDS, SELLER_MEMORY, TRUCK_MEMORY, previewFor } from './bikri-data';
import { CommandError } from '../../src/shared/api';
import { listRow, savedEvent1 } from './bikri-saved';
import { refusal } from './ui-error';
import { fakeMasters, record } from './masters';
import { bikriListOf, pukhtaRegisterOf } from './paged';

export const RAHIM = account(IDS.rahim, 'seller', '0010', 'رحیم', 'Rahim', {
  balance: balance(4316, 'cr'),
});
export const KARIM = account(IDS.karim, 'seller', '0011', 'کریم', 'Karim');
export const AKRAM = account(IDS.akram, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  city: ref(1, '0001', 'کوٹ رادھا کشن', 'Kot Radha Kishan'),
  balance: balance(2880, 'dr'),
});
export const ARSHAD = account(IDS.arshad, 'buyer', '0021', 'ارشد ملتانی', 'Arshad Multani', {
  balance: balance(18684, 'dr'),
});
export const PUKHTA_BUYER = account(
  IDS.pukhtaBuyer,
  'buyer',
  '0030',
  'پختہ کھاتہ (خریدار)',
  'Pukhta Khata (Buyer)',
);
export const PUKHTA_SELLER = account(
  IDS.pukhtaSeller,
  'seller',
  '0031',
  'پختہ کھاتہ (بیوپاری)',
  'Pukhta Khata (Seller)',
);

export const BANANA = record('item', IDS.banana, '0001', 'کیلا', 'Banana');
export const APPLE = record('item', IDS.apple, '0002', 'سیب', 'Apple');
export const BOX = record('packing', IDS.box, '0001', 'پیٹی', 'Box');
export const SWAT = record('adda', IDS.swat, '0001', 'سوات اڈا', 'Swat Adda');

export interface FakeBikri {
  /** Every draft the screen asked the preview for, in order. */
  previews: BikriDraft[];
  /** The latest draft the screen asked the preview for. */
  lastDraft: () => BikriDraft;
  /** Every save the screen sent, in order: the draft and the confirmation flag. */
  saves: { draft: BikriDraft; confirm: boolean }[];
  /** Every draft the screen kept with `drafts_save`, in order, as text. */
  drafts: string[];
}

/**
 * Answers all the commands the screen uses. `answer` decides the preview (default: the Event 1
 * reply fitted to the draft). The wrappers must have been mocked with vi.mock in the test file.
 */
export function fakeBikri(
  answer: (draft: BikriDraft) => BikriPreview | Promise<BikriPreview> = previewFor,
): FakeBikri {
  const store: FakeBikri = {
    previews: [],
    saves: [],
    drafts: [],
    lastDraft: () => {
      const last = store.previews.at(-1);
      if (last === undefined) throw new Error('the screen has not asked for a preview yet');
      return last;
    },
  };
  fakeAccounts([RAHIM, KARIM, AKRAM, ARSHAD, PUKHTA_BUYER, PUKHTA_SELLER]);
  fakeAccountSearch([RAHIM, KARIM, AKRAM, ARSHAD, PUKHTA_BUYER, PUKHTA_SELLER]);
  const masters = fakeMasters({ item: [BANANA, APPLE], packing: [BOX], adda: [SWAT] });
  vi.mocked(searchApi.findMaster).mockImplementation((scope, id) => {
    const found = masters.rows[scope as MasterKind].find((row) => row.id === id);
    return Promise.resolve(
      found === undefined
        ? null
        : {
            id: found.id,
            kind: scope,
            code: found.code,
            name_ur: found.names?.name_ur ?? '',
            name_en: found.names?.name_en ?? '',
            extra_ur: '',
            extra_en: '',
            is_active: true,
            sold_by_weight: false,
          },
    );
  });
  vi.mocked(bikriApi.bikriPreview).mockImplementation((draft) => {
    store.previews.push(draft);
    return Promise.resolve(answer(draft));
  });
  vi.mocked(bikriApi.assistTruckMemory).mockImplementation((truckNo) =>
    Promise.resolve(
      truckNo.toUpperCase().startsWith('LES')
        ? TRUCK_MEMORY
        : { last: null, same_day: null, suggestions: [] },
    ),
  );
  vi.mocked(bikriApi.assistSellerMemory).mockResolvedValue(SELLER_MEMORY);
  vi.mocked(bikriApi.bikriSplitFreight).mockResolvedValue([]);
  // The commands of plan 13. A test file mocks the modules it uses with vi.mock; a module that is
  // not mocked is left alone (its calls fail, and the screen copes, as it must).
  if (vi.isMockFunction(bikriApi.bikriSave)) {
    vi.mocked(bikriApi.bikriSave).mockImplementation((draft, confirm) =>
      Promise.resolve(answer(draft)).then((preview) => {
        // Like the real backend: an entry that takes cash below zero is refused until the user
        // has said yes (PRD FR-6.7). Nothing is saved then.
        if (preview.cash.needs_confirm && !confirm) {
          throw new CommandError(
            refusal([
              {
                code: 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED',
                field: null,
                values: { cash_after_paisa: -500_000 },
              },
            ]),
          );
        }
        store.saves.push({ draft, confirm });
        return { bikri_id: 41, number: 1, entry_id: 7, preview };
      }),
    );
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(savedEvent1());
    vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
      Promise.resolve(bikriListOf([listRow()], input.offset, input.limit)),
    );
    vi.mocked(bikriApi.bikriFind).mockResolvedValue([listRow()]);
    vi.mocked(bikriApi.bikriNavigate).mockResolvedValue(null);
  }
  if (vi.isMockFunction(draftsApi.draftsSave)) {
    vi.mocked(draftsApi.draftsSave).mockImplementation((_screen, payload) => {
      store.drafts.push(payload);
      return Promise.resolve({ saved_at: '2026-10-10T07:30:00Z' });
    });
    vi.mocked(draftsApi.draftsGet).mockResolvedValue(null);
    vi.mocked(draftsApi.draftsDiscard).mockResolvedValue(null);
  }
  if (vi.isMockFunction(pukhtaApi.searchPukhtaPeople)) {
    vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
    vi.mocked(pukhtaApi.pukhtaRegister).mockResolvedValue(pukhtaRegisterOf([]));
  }
  return store;
}
