// The head of the sale book (plan 71 H1 to H4, PRD 12.2 v1.34): the date and the number in the top
// row, and nothing else of a header. These tests are about what the head does on the screen: the
// keys from the date, a language switch in the middle of an entry, and a draft kept before the
// header lost its fields. Urdu and English. The numbers are the fake backend's.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { parseSnapshot, serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import { LOT } from '../../src/features/bikri/types';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { IDS } from '../helpers/bikri-data';
import { withOldHeader } from '../helpers/bikri-old-draft';
import {
  focusedIn,
  lotsGrid,
  named,
  names,
  openBook,
  textOf,
  typeCodeAndEnter,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

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
  usePendingRestore.getState().clear();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  usePendingRestore.getState().clear();
  leaveAppWindow();
});

const other = (language: Language): Language => (language === 'ur' ? 'en' : 'ur');
const dateBox = (language: Language) =>
  screen.getByRole('textbox', { name: named(message(language, 'bikri-header-date')) });

describe.each(LANGUAGES)('a language switch in the head (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('keeps the date typed and the lot, shows the number in the other language, and the keys go on', async () => {
    const { backend, user } = await openBook(language);
    // The date is the first field: typing replaces it, Enter goes to the seller of the first lot.
    expect(dateBox(language)).toHaveFocus();
    await user.keyboard('11-10-2026{Enter}');
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('0:seller');
    });
    await typeCodeAndEnter(user, '0');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(names(language).rahim);
    });
    await waitFor(() => {
      expect(backend.lastDraft().header.entry_date).toBe('11-10-2026');
    });
    // The number of a new sale is the word for new, in the app language.
    expect(
      screen.getByText(
        (text) => visible(text) === visible(message(language, 'bikri-head-number-new')),
      ),
    ).toBeInTheDocument();

    const next = other(language);
    press('KeyL', { altKey: true });
    await waitFor(() => {
      expect(textOf(lotsGrid(next), 0, 'seller')).toBe(names(next).rahim);
    });
    // Nothing typed is lost: the date, and the draft the backend has seen.
    expect(dateBox(next)).toHaveValue('11-10-2026');
    expect(
      screen.getByText((text) => visible(text) === visible(message(next, 'bikri-head-number-new'))),
    ).toBeInTheDocument();
    expect(backend.lastDraft().header.entry_date).toBe('11-10-2026');
    // The cursor is back where it was, and the keys go on from there (the item, then the packing).
    await waitFor(() => {
      expect(focusedIn(lotsGrid(next))).not.toBeNull();
    });
  });
});

describe.each(LANGUAGES)('the bottom of the page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  // jsdom has no layout, so this holds the two classes that make the area run down to the status bar
  // (checked in a real WebKit window at 1366x768 in Urdu: the empty state's help line and button are
  // whole, with no empty band under them). The area is the one that scrolls.
  it("the empty state sits in a scroll area that runs down through the shell's band, with that band as its own bottom padding", async () => {
    await openBook(language);
    const title = await screen.findByText(
      (text) => visible(text) === visible(message(language, 'bikri-empty-title')),
    );
    const area = title.closest('.overflow-auto');
    expect(area).not.toBeNull();
    expect(area?.className).toContain('pb-corner');
    expect(area?.parentElement?.className).toContain('fill-to-corner');
  });
});

describe.each(LANGUAGES)('a draft kept before the header lost its fields (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /**
   * The draft of before: a truck, a زمیندار (Karim), an اڈا and the rest in the header, and one lot
   * whose seller is Rahim.
   */
  function offerOldDraft() {
    const rahim = { id: IDS.rahim, code: '0010', name_ur: 'رحیم', name_en: 'Rahim' };
    const karim = { id: IDS.karim, code: '0011', name_ur: 'کریم', name_en: 'Karim' };
    const swat = { id: IDS.swat, code: '0001', name_ur: 'سوات اڈا', name_en: 'Swat Adda' };
    const stores = createBikriDraft('10-10-2026');
    const lot = stores.lots.getState().rows[0];
    if (lot === undefined) throw new Error('no lot row');
    stores.lots.getState().patchRow(lot.rowId, {
      typed: { [LOT.qty]: '50' },
      picked: { [LOT.seller]: rahim },
    });
    const snapshot = parseSnapshot(
      withOldHeader(serialiseDraft(stores), {
        truckNo: 'LES-1234',
        zamindar: karim,
        adda: swat,
        driverName: 'Aslam',
        driverPhone: '03001234567',
        totalFreight: '3000',
        book: 'B1',
        note: 'old',
      }),
    );
    if (snapshot === null) throw new Error('not a snapshot');
    usePendingRestore.getState().offer(snapshot);
  }

  it('opens without error, shows the lot, no old field, and sends none of the old values', async () => {
    offerOldDraft();
    const { backend } = await openBook(language);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(names(language).rahim);
    });
    expect(textOf(lotsGrid(language), 0, 'qty')).toBe('50');
    // None of the old header fields is on the screen.
    for (const id of ['bikri-header-truck', 'bikri-header-driver', 'bikri-header-driver-phone']) {
      expect(
        screen.queryByRole('textbox', { name: named(message(language, id)) }),
        id,
      ).not.toBeInTheDocument();
    }
    for (const id of ['glossary-zamindar', 'glossary-adda']) {
      expect(
        screen.queryByRole('combobox', { name: named(message(language, id)) }),
        id,
      ).not.toBeInTheDocument();
    }
    // The draft asked about has the date alone: a new sale has none of the old values.
    await waitFor(() => {
      expect(backend.lastDraft().header).toEqual({ entry_date: '10-10-2026' });
    });
  });

  it('a new lot does not take the old زمیندار as its seller: it starts as the lot before it', async () => {
    offerOldDraft();
    const { user } = await openBook(language);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(names(language).rahim);
    });
    await user.click(
      screen.getByRole('button', {
        name: (label) => visible(label).startsWith(visible(message(language, 'bikri-lots-add'))),
      }),
    );
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 1, 'seller')).toBe(names(language).rahim);
    });
  });
});
