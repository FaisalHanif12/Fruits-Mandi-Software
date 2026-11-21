// A seller's money paid on the spot with the backend's REAL answers (plan 91, PRD FR-12.23 v1.53): its four worked
// previews, drawn in the seller's summary in Urdu and in English with the numbers of the plan (previous 30,000 بنام,
// credit 20,000, paid 5,000: the rest 15,000, new 15,000 بنام; two lots paid 15,000; the shop owed him 10,000 and he took
// 28,000; paid 25,000 needing the Owner's yes), and its spread of 15,000 over two lots (12,000 and 3,000) written into
// the lots' own نقد fields. Nothing on the screen is worked out: every figure is read off the recorded answer.

import { renderHook, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { useSellerPaidSpread } from '../../src/features/bikri/hooks/useSellerPaidSpread';
import { LOT } from '../../src/features/bikri/types';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as urduApi from '../../src/shared/urdu/api';
import { IDS } from '../helpers/bikri-data';
import { names, openBook, typeEvent1Lot } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import {
  SELLER_PREVIEW_A,
  SELLER_PREVIEW_B,
  SELLER_PREVIEW_C,
  SELLER_PREVIEW_D,
  SELLER_SAVE_REFUSED_D,
  SELLER_SPREAD_TWO,
} from '../helpers/seller-paid-real';
import { refusal } from '../helpers/ui-error';
import type { BikriPreview } from '../../src/bindings/BikriPreview';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const RAHIM = IDS.rahim;

describe.each(LANGUAGES)(
  'a seller’s summary from the backend’s real previews (%s)',
  (language: Language) => {
    const m = (id: string, vars: Record<string, string | number> = {}) =>
      message(language, id, vars);
    const side = (which: 'dr' | 'cr') => m(which === 'dr' ? 'common-side-dr' : 'common-side-cr');
    const block = () => {
      const found = document.querySelector<HTMLElement>(`[data-seller-paid="${String(RAHIM)}"]`);
      if (found === null) throw new Error('no seller block');
      return found;
    };
    const lines = () =>
      Array.from(block().querySelectorAll('dl > div')).map((row): [string, string] => [
        visible(row.querySelector('dt')?.textContent),
        visible(row.querySelector('dd')?.textContent),
      ]);

    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    async function openWith(preview: BikriPreview) {
      const book = await openBook(language, () => preview);
      await typeEvent1Lot(book.user, language);
      await waitFor(() => {
        expect(document.querySelector('[data-seller-paid]')).not.toBeNull();
        expect(lines().length).toBeGreaterThan(0);
      });
      return book;
    }

    it('case A: previous 30,000 بنام, credit 20,000, paid 5,000: the rest 15,000 credited to his account, new 15,000 بنام', async () => {
      await openWith(SELLER_PREVIEW_A);
      expect(visible(block().textContent)).toContain(
        m('bikri-seller-paid-title', { name: names(language).rahim }),
      );
      expect(lines().map(([label]) => label)).toEqual([
        m('bikri-seller-paid-previous'),
        m('bikri-seller-paid-net'),
        m('bikri-seller-paid-paid'),
        m('bikri-seller-paid-rest'),
        m('bikri-seller-paid-new'),
      ]);
      expect(lines()[0]?.[1]).toContain('30,000');
      expect(lines()[0]?.[1]).toContain(side('dr'));
      expect(lines()[1]?.[1]).toBe('20,000');
      expect(lines()[2]?.[1]).toBe('5,000');
      expect(lines()[3]?.[1]).toBe('15,000');
      expect(lines().at(-1)?.[1]).toContain('15,000');
      expect(lines().at(-1)?.[1]).toContain(side('dr'));
    });

    it('case B: two lots, paid 15,000: the rest is 5,000 and the new balance 25,000 بنام', async () => {
      await openWith(SELLER_PREVIEW_B);
      expect(lines().map(([label]) => label)).toEqual([
        m('bikri-seller-paid-previous'),
        m('bikri-seller-paid-net'),
        m('bikri-seller-paid-paid'),
        m('bikri-seller-paid-rest'),
        m('bikri-seller-paid-new'),
      ]);
      expect(lines()[2]?.[1]).toBe('15,000');
      expect(lines()[3]?.[1]).toBe('5,000');
      expect(lines().at(-1)?.[1]).toContain('25,000');
      expect(lines().at(-1)?.[1]).toContain(side('dr'));
    });

    it('case C: the shop owed him 10,000 and he took 28,000: no warning, the line "paid above today’s sale" 8,000, new 2,000 جمع', async () => {
      await openWith(SELLER_PREVIEW_C);
      expect(lines()[0]?.[1]).toContain(side('cr'));
      expect(lines().find(([label]) => label === m('bikri-seller-paid-over'))?.[1]).toBe('8,000');
      expect(lines().at(-1)?.[1]).toContain('2,000');
      expect(lines().at(-1)?.[1]).toContain(side('cr'));
      expect(
        within(block()).queryByText((text) => visible(text) === m('bikri-seller-paid-above')),
      ).toBeNull();
    });

    it('case D: paid 25,000 is above his limit: the amber line, F10 asks naming him and 35,000 بنام, yes saves with the yes', async () => {
      const { backend, user } = await openWith(SELLER_PREVIEW_D);
      // The save refuses with the backend's own list until the yes comes (plan 91, one yes for the entry).
      vi.mocked(bikriApi.bikriSave).mockImplementation((draft, confirm) => {
        if (!confirm) {
          return Promise.reject(
            new CommandError(
              refusal(
                SELLER_SAVE_REFUSED_D.map((item) => ({
                  code: item.code,
                  field: item.field,
                  values: {},
                })),
              ),
            ),
          );
        }
        backend.saves.push({ draft, confirm });
        return Promise.resolve({ bikri_id: 41, number: 1, entry_id: 7, preview: SELLER_PREVIEW_A });
      });
      expect(
        within(block()).getByText((text) => visible(text) === m('bikri-seller-paid-above')),
      ).toBeInTheDocument();
      expect(lines().find(([label]) => label === m('bikri-seller-paid-over'))?.[1]).toBe('5,000');
      press('F10');
      const box = await screen.findByRole('alertdialog');
      expect(visible(box.textContent)).toContain(names(language).rahim);
      expect(visible(box.textContent)).toContain('35,000');
      await user.click(
        within(box).getByRole('button', {
          name: (name) => visible(name).startsWith(m('common-cash-confirm-yes')),
        }),
      );
      await waitFor(() => {
        expect(backend.saves).toHaveLength(1);
      });
      expect(backend.saves[0]?.confirm).toBe(true);
    });
  },
);

describe('the spread of 15,000 over two lots (the backend’s real answer)', () => {
  it('writes 12,000 and 3,000 into the two lots’ own نقد fields, and asks with the amount as typed and no posting left out', async () => {
    const stores = createBikriDraft('2026-10-10');
    const seller = { id: RAHIM, code: '0010', name_ur: 'رحیم', name_en: 'Rahim' };
    const first = stores.lots.getState().rows[0]?.rowId ?? '';
    stores.lots.getState().patchRow(first, { picked: { [LOT.seller]: seller } });
    stores.lots.getState().addRow();
    const second = stores.lots.getState().rows[1]?.rowId ?? '';
    stores.lots.getState().patchRow(second, { picked: { [LOT.seller]: seller } });
    // The answer is the backend's, with its lot keys put to the keys of these two rows.
    vi.mocked(bikriApi.bikriSellerPaidSpread).mockResolvedValue({
      ...SELLER_SPREAD_TWO,
      lots: SELLER_SPREAD_TWO.lots.map((lot, index) => ({
        ...lot,
        key: index === 0 ? first : second,
      })),
    });
    const { result } = renderHook(() => useSellerPaidSpread(stores, null));
    result.current.spread(RAHIM, '15000');
    await waitFor(() => {
      expect(stores.lots.getState().rows.map((row) => row.typed[LOT.cash])).toEqual([
        '12000',
        '3000',
      ]);
    });
    const sent = vi.mocked(bikriApi.bikriSellerPaidSpread).mock.calls.at(-1)?.[0];
    expect(sent?.seller_paid).toEqual([{ seller_account_id: RAHIM, amount: '15000' }]);
    expect(sent?.entry_id ?? null).toBeNull();
  });
});
