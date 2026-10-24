// The screens of plan 13 against the real answers of the backend (T4). The backend wrote, in its
// own golden test, what bikri_save, bikri_get, bikri_list, pukhta_register and drafts_get answer
// for Event 1 and Event 2 of PRD 4.4 and 4.5 (tests/fixtures/*.json, copied, never edited). Each
// screen here shows those numbers, in the Urdu and in the English app, and the save screen sends
// the draft of the backend's file. Money in the files is paisa; the screen shows whole rupees.

import { cleanup, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import * as bikriApi from '../../src/features/bikri/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import { LANGUAGES } from '../../src/i18n';
import * as draftsApi from '../../src/shared/drafts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import {
  DRAFT_ROW,
  GET_EVENT1,
  GET_EVENT2,
  LIST_DAY,
  REGISTER_DAY,
  REGISTER_DAY_BUYERS,
  SAVED_EVENT1,
  SAVE_EVENT1_REQUEST,
} from '../helpers/bikri-real';
import { named, openBook, salesGrid, textOf, typeEvent1Lot } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  // Reset, not only clear: a rejection queued with mockRejectedValueOnce by a test that failed early
  // must not reach the next test.
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const pageText = () => visible(document.body.textContent);

/** The draft in the form of the backend's file: our row keys renamed, commas out of the numbers. */
function normalized(draft: BikriDraft): unknown {
  const copy = structuredClone(draft);
  copy.lots.forEach((lot, lotIndex) => {
    lot.key = `lot-${String(lotIndex + 1)}`;
    lot.sales.forEach((sale, saleIndex) => {
      sale.key = `line-${String(saleIndex + 1)}`;
    });
  });
  return withoutCommas(copy);
}

function withoutCommas(value: unknown): unknown {
  if (typeof value === 'string') return /^[\d,]+$/.test(value) ? value.replaceAll(',', '') : value;
  if (Array.isArray(value)) return value.map(withoutCommas);
  if (typeof value === 'object' && value !== null) {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, withoutCommas(v)]));
  }
  return value;
}

describe.each(LANGUAGES)('a saved بکری from the backend’s answers (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const title = (number: number) =>
    screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-saved-title', { number })),
    });

  it('Event 1: gross 29,400, deductions 10,084, net 19,316, the two bills, the split and the entry total', async () => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(GET_EVENT1);
    await renderApp('/bikri/book?id=1');
    await title(1);
    const text = pageText();
    for (const figure of [
      '29,400',
      '10,084',
      '19,316',
      '1,764',
      '12,880',
      '18,684',
      '1,580',
      '1,184',
      '36,564',
    ]) {
      expect(text, figure).toContain(figure);
    }
    expect(text).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(text).toContain(language === 'ur' ? 'الف' : 'Partner A');
    expect(bikriApi.bikriGet).toHaveBeenCalledWith(1);
    expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
  });

  it('Event 2: net 10,160, the buyer pays 12,800, the split is 616 / 462 / 462, and Shahid is named', async () => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(GET_EVENT2);
    await renderApp('/bikri/book?id=2');
    await title(2);
    const text = pageText();
    for (const figure of ['12,000', '1,840', '10,160', '12,800', '616', '462', '25,600']) {
      expect(text, figure).toContain(figure);
    }
    expect(text).toContain(language === 'ur' ? 'شاہد' : 'Shahid');
    expect(text).toContain('LHR-0077');
  });
});

describe.each(LANGUAGES)(
  'the list and the register from the backend’s answers (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      fakeBikri();
    });

    const cell = (rowId: string, colId: string) =>
      visible(
        document.querySelector(`.ag-row[row-id="${rowId}"]`)?.querySelector(`[col-id="${colId}"]`)
          ?.textContent,
      );

    it('today’s list is newest first, with the stored gross and net of both events', async () => {
      vi.mocked(bikriApi.bikriList).mockResolvedValue(LIST_DAY);
      await renderApp('/bikri/today');
      await waitFor(() => {
        expect(cell('1', 'number')).toBe('1');
      });
      const order = Array.from(
        document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)'),
      ).map((row) => row.getAttribute('row-id'));
      expect(order).toEqual(['2', '1']);
      expect(cell('1', 'gross')).toBe('29,400');
      expect(cell('1', 'net')).toBe('19,316');
      expect(cell('2', 'gross')).toBe('12,000');
      expect(cell('2', 'net')).toBe('10,160');
      expect(cell('2', 'sellers')).toBe(language === 'ur' ? 'کریم' : 'Karim');
      expect(cell('1', 'status')).toBe(message(language, 'bikri-status-saved'));
    });

    it('the register of the buyers shows the backend’s totals; with no side only the word shows', async () => {
      vi.mocked(pukhtaApi.pukhtaRegister).mockResolvedValue(REGISTER_DAY);
      await renderApp('/bikri/pukhta');
      await waitFor(() => {
        expect(cell('0', 'bikri')).toBe('2');
      });
      // No side: every sum is null, so the totals row has only its word.
      const pinned = () => document.querySelector('.ag-row-pinned');
      expect(visible(pinned()?.textContent)).toBe(message(language, 'glossary-meezan'));
      cleanup();
      vi.mocked(pukhtaApi.pukhtaRegister).mockResolvedValue(REGISTER_DAY_BUYERS);
      await renderApp('/bikri/pukhta');
      await waitFor(() => {
        expect(
          visible(document.querySelector('.ag-row-pinned [col-id="amount"]')?.textContent),
        ).toBe('12,800');
      });
      expect(visible(pinned()?.querySelector('[col-id="state"]')?.textContent)).toBe(
        message(language, 'pukhta-state-due', { amount: '0' }),
      );
    });

    it('the register shows Shahid’s line as paid, 12,800', async () => {
      vi.mocked(pukhtaApi.pukhtaRegister).mockResolvedValue(REGISTER_DAY);
      await renderApp('/bikri/pukhta');
      await waitFor(() => {
        expect(cell('0', 'bikri')).toBe('2');
      });
      expect(cell('0', 'amount')).toBe('12,800');
      expect(cell('0', 'name')).toBe(language === 'ur' ? 'شاہد' : 'Shahid');
      expect(cell('0', 'state')).toBe(message(language, 'pukhta-state-paid'));
    });
  },
);

describe.each(LANGUAGES)('saving Event 1 as the backend’s file has it (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('the draft F10 sends is the draft of bikri_save_event1_request.json, and the result opens a new sale', async () => {
    const { backend, user } = await openBook(language);
    vi.mocked(bikriApi.assistTruckMemory).mockResolvedValue({
      last: null,
      same_day: null,
      suggestions: [],
    });
    await typeEvent1Lot(user, language);
    await user.keyboard('1{Enter}20{Enter}600{Enter}');
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    });
    await user.keyboard('2{Enter}{Enter}580{Enter}');
    await waitFor(() => {
      expect(backend.previews.at(-1)?.lots[0]?.sales[1]?.seller_rate).toBe('580');
    });
    // The backend’s own answer for this save, with the number it gave.
    vi.mocked(bikriApi.bikriSave).mockResolvedValueOnce(SAVED_EVENT1);
    press('F10');
    await waitFor(() => {
      expect(bikriApi.bikriSave).toHaveBeenCalledTimes(1);
    });
    const [sent, confirm] = vi.mocked(bikriApi.bikriSave).mock.calls[0] ?? [];
    if (sent === undefined) throw new Error('the save was not asked for');
    expect(confirm).toBe(false);
    const wanted = withoutCommas(SAVE_EVENT1_REQUEST.draft) as BikriDraft;
    const got = normalized(sent) as BikriDraft;
    expect(got.header).toEqual(wanted.header);
    const { sales: gotSales, ...gotLot } = got.lots[0] ?? { sales: [] };
    const { sales: wantedSales, ...wantedLot } = wanted.lots[0] ?? { sales: [] };
    expect(gotLot).toEqual(wantedLot);
    // The two finished lines are the file’s; the blank row after them is dropped by the backend.
    expect(gotSales.slice(0, 2)).toEqual(wantedSales.slice(0, 2));
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'bikri-saved-toast', { number: SAVED_EVENT1.number }),
      );
    });
    // The result carries the number and the id the "open" link needs.
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'bikri-last-saved', { number: SAVED_EVENT1.number }),
      );
    });
  });
});

describe.each(LANGUAGES)('a draft the backend sends back (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  it('a draft that is not a snapshot of this screen is thrown away quietly, never half used', async () => {
    // The file is a plain draft as the backend's own test stored it; the screen stores its own
    // snapshot, so it cannot read this one. No question is asked and nothing is restored.
    vi.mocked(draftsApi.draftsGet).mockResolvedValue(DRAFT_ROW);
    await renderApp('/');
    await waitFor(() => {
      expect(draftsApi.draftsDiscard).toHaveBeenCalledWith('bikri');
    });
    expect(pageText()).not.toContain(message(language, 'bikri-draft-title'));
  });
});
