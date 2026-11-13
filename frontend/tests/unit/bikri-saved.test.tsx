// A saved بکری opened read-only, the arrows between بکری, find and "new" (plan 13, PRD FR-12.13),
// in the Urdu and the English app, against a fake backend. A saved بکری shows the values stored
// when it was saved, never worked out again, and cannot be typed in.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as draftsApi from '../../src/shared/drafts/api';
import { displayName } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { listRow, savedEvent1, savedTwoLots } from '../helpers/bikri-saved';
import { named } from '../helpers/bikri-screen';
import {
  fail,
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
  vi.unstubAllGlobals();
  document.documentElement.style.removeProperty('--breakpoint-normal');
  document.documentElement.style.removeProperty('--breakpoint-wide');
});

const pageText = () => visible(document.body.textContent);

/** The values of the label of lot `number` of the two-lot saved بکری: its seller and its item (PRD FR-12.6). */
function savedLotNamed(language: Language, number: number) {
  const lot = savedTwoLots().lots[number - 1];
  if (lot === undefined) throw new Error('no such lot');
  return {
    lot: number,
    // The seller, the item and the counter the backend gave the lot (PRD FR-12.45): "Rahim, Banana-1".
    who: message(language, 'bikri-lot-name', {
      seller: displayName(lot.seller, language),
      item: displayName(lot.item, language),
      counter: String(lot.lot_counter),
    }),
    named: 'yes',
  };
}

describe.each(LANGUAGES)('a saved بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const open = () => renderApp('/bikri/book?id=41');

  // Plan 71 H3: a بکری saved after v1.34 has none of the old header values, so its saved view shows the
  // date and when it was saved, and no empty truck, اڈا or زمیندار; the bill date shows only when it
  // is not the entry date.
  it('a بکری with none of the old header values shows only its date and when it was saved', async () => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(
      savedEvent1({
        truck_no: null,
        zamindar: null,
        adda_id: null,
        driver_name: null,
        driver_phone: null,
        total_freight: null,
        store_id: null,
        book: null,
        note: null,
      }),
    );
    await open();
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-saved-title', { number: 1 })),
    });
    const text = pageText();
    for (const id of ['bikri-header-date', 'bikri-saved-at']) {
      expect(text).toContain(visible(message(language, id)));
    }
    for (const id of [
      'bikri-header-bill-date',
      'bikri-header-truck',
      'glossary-zamindar',
      'glossary-adda',
      'bikri-header-total-freight',
      'bikri-header-driver',
      'bikri-header-driver-phone',
    ]) {
      expect(text, id).not.toContain(visible(message(language, id)));
    }
  });

  it('an old بکری shows its old header values, each only when it is there, and its bill date when it differs', async () => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(
      savedEvent1({ bill_date: '2026-10-11', book: 'B-7', note: null }),
    );
    await open();
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-saved-title', { number: 1 })),
    });
    const text = pageText();
    expect(text).toContain('LES-1234');
    expect(text).toContain('B-7');
    expect(text).toContain('11-10-2026');
    expect(text).toContain(visible(message(language, 'bikri-header-bill-date')));
    expect(text).not.toContain(visible(message(language, 'bikri-header-note')));
  });

  // Plan 75 (alignment R4): the lines of a saved lot at the normal step are nine columns in 852 px. They
  // all stay (NFR-USE-27): the table is dense there (8 px at each side of a cell, narrower figure
  // columns, balanced headings), so the English heading "Paid on the Spot" is two lines and not four. The
  // wide step has the room and keeps the plain table.
  it.each([
    [1366, true, ['bikri-col-seller-amount', 'bikri-col-seller-comm', 'bikri-col-buyer-comm']],
    [
      1920,
      false,
      ['bikri-col-seller-amount', 'bikri-col-seller-comm', 'bikri-col-buyer-comm', 'glossary-laga'],
    ],
  ])(
    'at %i px the lines of a saved lot show the whole line, dense %s',
    async (width, dense, shown) => {
      // The layout step is read from media queries; a window of this width matches the steps up to it.
      vi.stubGlobal('matchMedia', (query: string) => ({
        matches: width >= Number.parseInt(query.replace(/\D+/, ''), 10),
        media: query,
        addEventListener: () => undefined,
        removeEventListener: () => undefined,
      }));
      document.documentElement.style.setProperty('--breakpoint-normal', '1366px');
      document.documentElement.style.setProperty('--breakpoint-wide', '1920px');
      await open();
      const table = await screen.findByRole('table', {
        name: named(message(language, 'bikri-sales-label')),
      });
      const headers = within(table).getAllByRole('columnheader');
      const heads = headers.map((head) => visible(head.textContent));
      for (const id of [
        'bikri-col-qty',
        'bikri-col-rate',
        'bikri-col-buyer-total',
        'bikri-col-paid-on-spot',
        'bikri-line-left',
        ...shown,
      ]) {
        expect(heads, id).toContain(visible(message(language, id)));
      }
      for (const head of headers) {
        if (dense) expect(head).toHaveClass('px-2', 'text-balance');
        else expect(head).toHaveClass('px-3');
      }
    },
  );

  it('shows the values stored when it was saved, and the preview is never asked', async () => {
    await open();
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'bikri-saved-title', { number: 1 })),
      }),
    ).toBeInTheDocument();
    expect(bikriApi.bikriGet).toHaveBeenCalledWith(41);
    const text = pageText();
    // The header: day, truck, the زمیندار by the name of the app language.
    expect(text).toContain('10-10-2026');
    expect(text).toContain('LES-1234');
    expect(text).toContain(language === 'ur' ? 'رحیم' : 'Rahim');
    // The lot: the stored numbers of Event 1 (E2E-1): gross 29,400, deductions 10,084, net 19,316.
    expect(text).toContain('29,400');
    expect(text).toContain('10,084');
    expect(text).toContain('19,316');
    expect(text).toContain('1,764');
    // The lines (E2E-2): each buyer's bill, as stored.
    expect(text).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(text).toContain('12,880');
    expect(text).toContain('18,684');
    // The partners' share (E2E-4) and the entry total (E2E-3).
    expect(text).toContain('1,580');
    expect(text).toContain('1,184');
    expect(text).toContain('36,564');
    // Nothing is worked out again with today's rules, and nothing can be typed.
    expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
    // (The search box of the top bar is outside the screen.)
    const main = document.querySelector('main');
    if (main === null) throw new Error('no screen');
    expect(within(main).queryAllByRole('textbox')).toHaveLength(0);
    expect(within(main).queryAllByRole('combobox')).toHaveLength(0);
    expect(document.querySelector('.ag-root')).toBeNull();
    expect(text).toContain(message(language, 'bikri-saved-readonly'));
  });

  /** The text of the summary section. */
  const summaryText = () => {
    const heading = screen.getByRole('heading', {
      name: named(message(language, 'bikri-summary-title')),
    });
    const section = heading.closest('section');
    if (section === null) throw new Error('no summary');
    return visible(section.textContent);
  };

  it('the summary is the stored sums of the whole بکری, both sides, with the shared income', async () => {
    await open();
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-summary-title'));
    });
    const text = summaryText();
    // Seller side (E2E-1), buyer side (E2E-2) and the shared income (E2E-4).
    for (const figure of ['29,400', '3,000', '1,764', '250', '5,000', '10,084', '19,316']) {
      expect(text).toContain(figure);
    }
    for (const figure of ['400', '31,564', '3,948']) expect(text).toContain(figure);
    // A rate difference of 0 is not listed (PRD 12.5).
    expect(text).not.toContain(message(language, 'glossary-rate-farq'));
    expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
  });

  it('a بکری with two lots: the whole first, and a lot of its own when he picks it', async () => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(savedTwoLots());
    await open();
    await waitFor(() => {
      expect(summaryText()).toContain('41,400');
    });
    expect(summaryText()).toContain('44,364');
    expect(summaryText()).toContain('5,488');
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', {
        name: named(message(language, 'bikri-summary-lot', savedLotNamed(language, 1))),
      }),
    );
    await waitFor(() => {
      expect(summaryText()).toContain('29,400');
    });
    expect(summaryText()).not.toContain('41,400');
    // The second lot: choose it in the list that appears beside the switch.
    await user.click(
      screen.getByRole('combobox', { name: named(message(language, 'bikri-saved-pick-lot')) }),
    );
    await user.click(
      await screen.findByRole('option', {
        name: named(message(language, 'bikri-summary-lot', savedLotNamed(language, 2))),
      }),
    );
    await waitFor(() => {
      expect(summaryText()).toContain('12,000');
    });
    expect(summaryText()).toContain('10,160');
    expect(summaryText()).toContain('1,540');
  });

  it('says in words, in red, that a cancelled بکری is cancelled, with the reason', async () => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(
      savedEvent1({ status: 'cancelled', cancel_reason: 'wrong truck' }),
    );
    await open();
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-saved-cancelled'));
    });
    expect(pageText()).toContain(
      message(language, 'bikri-saved-cancel-reason', { reason: 'wrong truck' }),
    );
  });

  it('fruit taken from a lot is shown on its card, and only when some was taken (PRD 16.7, C16)', async () => {
    const base = savedEvent1();
    const lot = base.lots[0];
    if (lot === undefined) throw new Error('no lot');
    // E2E-42: 2 پیٹی were taken home; the lot's own figures are the backend's.
    vi.mocked(bikriApi.bikriGet).mockResolvedValue({
      ...base,
      lots: [{ ...lot, qty_sold: 45, qty_taken: 2 }],
    });
    await open();
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-saved-qty-taken'));
    });
    const label = screen.getAllByText(named(message(language, 'bikri-saved-qty-taken')))[0];
    expect(visible(label?.closest('div')?.textContent)).toContain('2');
  });

  it('a lot with no fruit taken does not show the figure', async () => {
    await open();
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-saved-title', { number: 1 })),
    });
    expect(screen.queryByText(named(message(language, 'bikri-saved-qty-taken')))).toBeNull();
  });

  it('a چکتا buyer is shown by his own name', async () => {
    const base = savedEvent1();
    const lot = base.lots[0];
    const line = lot?.sales[0];
    if (lot === undefined || line === undefined) throw new Error('no lot');
    vi.mocked(bikriApi.bikriGet).mockResolvedValue({
      ...base,
      lots: [
        {
          ...lot,
          sales: [
            {
              ...line,
              chukta: {
                name_ur: 'شاہد',
                name_en: 'Shahid',
                phone: '03000000000',
                person_key: 'shahid|03000000000',
              },
            },
          ],
        },
      ],
    });
    await open();
    await waitFor(() => {
      expect(pageText()).toContain(language === 'ur' ? 'شاہد' : 'Shahid');
    });
  });

  it('loading, then an error with a retry that opens it', async () => {
    let fails = true;
    vi.mocked(bikriApi.bikriGet).mockImplementation(() =>
      fails
        ? Promise.reject(fail({ code: 'ERR_NOT_FOUND', field: 'id', values: {} }))
        : Promise.resolve(savedEvent1()),
    );
    await open();
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-saved-failed'));
    });
    fails = false;
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'common-retry')) }),
    );
    await waitFor(() => {
      expect(pageText()).toContain('29,400');
    });
  });

  it('a language switch in the middle of looking at it shows the other names and the same numbers', async () => {
    await open();
    await waitFor(() => {
      expect(pageText()).toContain('29,400');
    });
    const user = userEvent.setup();
    await user.keyboard('{Alt>}l{/Alt}');
    const other = language === 'ur' ? 'en' : 'ur';
    await waitFor(() => {
      expect(pageText()).toContain(other === 'ur' ? 'رحیم' : 'Rahim');
    });
    expect(pageText()).toContain('29,400');
    expect(pageText()).toContain('12,880');
    expect(pageText()).toContain(message(other, 'bikri-saved-readonly'));
  });

  it('New opens an empty بکری; its key is on the button', async () => {
    const { router } = await open();
    const user = userEvent.setup();
    const button = await screen.findByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'bikri-new')),
    });
    expect(visible(button.textContent)).toContain('Alt+B');
    await user.click(button);
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('id');
    });
    // The new sale book: its first field is the date.
    expect(
      await screen.findByRole('textbox', { name: named(message(language, 'bikri-header-date')) }),
    ).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('Alt+B from a saved بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  it('opens a new empty بکری, the same as the New button', async () => {
    const { router } = await renderApp('/bikri/book?id=41');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-saved-readonly'));
    });
    const user = userEvent.setup();
    await user.keyboard('{Alt>}b{/Alt}');
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('id');
    });
    // The new sale book: its first field is the date.
    expect(
      await screen.findByRole('textbox', { name: named(message(language, 'bikri-header-date')) }),
    ).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('find, with no arrows and no find button (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  /** The saved بکری is on screen when its New button is there. */
  const savedIsOn = () =>
    screen.findByRole('button', {
      name: (label) => visible(label).startsWith(visible(message(language, 'bikri-new'))),
    });

  it('the toolbar has no find button and no arrows (Umar, 12-11-2026): F3 is the way to the find box', async () => {
    await renderApp('/bikri/book?id=41');
    await savedIsOn();
    // The old find button showed its key, F3; the four arrows were icon buttons beside it.
    const buttons = screen.getAllByRole('button');
    expect(buttons.filter((button) => button.textContent.includes('F3'))).toEqual([]);
    expect(bikriApi.bikriNavigate).not.toHaveBeenCalled();
    // The key is still there.
    press('F3');
    expect(await screen.findByRole('dialog')).toBeInTheDocument();
  });

  it('the new بکری has neither the arrows nor a find button, and F3 still opens the box', async () => {
    await renderApp('/bikri/book');
    await screen.findByRole('textbox', { name: named(message(language, 'bikri-header-date')) });
    expect(
      screen.getAllByRole('button').filter((button) => button.textContent.includes('F3')),
    ).toEqual([]);
    press('F3');
    expect(await screen.findByRole('dialog')).toBeInTheDocument();
  });

  describe('find (F3)', () => {
    const openFind = async () => {
      const view = await renderApp('/bikri/book?id=41');
      await savedIsOn();
      press('F3');
      const dialog = await screen.findByRole('dialog');
      return { ...view, dialog, user: userEvent.setup() };
    };

    it('F3 opens it with the six filters and a hint, and nothing is asked yet', async () => {
      const { dialog } = await openFind();
      for (const id of [
        'bikri-find-number',
        'bikri-header-truck',
        'glossary-beopari',
        'glossary-gahak',
        'bikri-find-walk-in',
        'bikri-header-date',
        'glossary-qism-maal',
      ]) {
        expect(visible(dialog.textContent)).toContain(message(language, id));
      }
      expect(visible(dialog.textContent)).toContain(message(language, 'bikri-find-hint'));
      expect(bikriApi.bikriFind).not.toHaveBeenCalled();
    });

    it('?find=true (an answer sends the Owner here to edit or cancel a بکری): the box is open at once, and the request leaves the address', async () => {
      const { router } = await renderApp('/bikri/book?find=true');
      const dialog = await screen.findByRole('dialog');
      expect(visible(dialog.textContent)).toContain(message(language, 'bikri-find-hint'));
      expect(bikriApi.bikriFind).not.toHaveBeenCalled();
      // A reload or a save does not open it again.
      await waitFor(() => {
        expect(router.state.location.search).toEqual({});
      });
    });

    it('a saved بکری in the address is not asked to open its find box: ?id=41&find=true has no box', async () => {
      await renderApp('/bikri/book?id=41&find=true');
      await savedIsOn();
      expect(screen.queryByRole('dialog')).toBeNull();
    });

    it('a number finds the بکری; Enter on the result opens it', async () => {
      vi.mocked(bikriApi.bikriFind).mockResolvedValue([
        listRow({ id: 33, number: 1, truck_no: 'LES-1234' }),
      ]);
      const { dialog, user, router } = await openFind();
      await user.keyboard('1');
      await waitFor(() => {
        expect(bikriApi.bikriFind).toHaveBeenLastCalledWith({
          number: 1,
          truck_no: null,
          seller_id: null,
          date: null,
          item_id: null,
          buyer_account_id: null,
          buyer_text: null,
        });
      });
      const result = await within(dialog).findByRole('button', {
        name: (name) => visible(name).includes(message(language, 'bikri-hit-title', { number: 1 })),
      });
      expect(visible(result.textContent)).toContain('LES-1234');
      result.focus();
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.search).toMatchObject({ id: 33 });
      });
      await waitFor(() => {
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      });
    });

    it('a truck number is sent as typed; a cancelled result says so in words', async () => {
      vi.mocked(bikriApi.bikriFind).mockResolvedValue([
        listRow({ id: 30, number: 7, truck_no: 'LES-9999', status: 'cancelled' }),
      ]);
      const { dialog, user } = await openFind();
      await user.click(
        within(dialog).getByRole('textbox', {
          name: named(message(language, 'bikri-header-truck')),
        }),
      );
      await user.keyboard('les');
      await waitFor(() => {
        expect(bikriApi.bikriFind).toHaveBeenLastCalledWith(
          expect.objectContaining({ truck_no: 'les', number: null }),
        );
      });
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(message(language, 'bikri-status-cancelled'));
      });
    });

    it('nothing found says so; an error has a retry; Esc closes the box', async () => {
      vi.mocked(bikriApi.bikriFind).mockResolvedValue([]);
      const { dialog, user } = await openFind();
      await user.keyboard('9');
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(message(language, 'bikri-find-none'));
      });
      vi.mocked(bikriApi.bikriFind).mockRejectedValue(
        fail({ code: 'ERR_DB_BUSY', field: null, values: {} }),
      );
      await user.keyboard('1');
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(message(language, 'bikri-find-failed'));
      });
      expect(
        within(dialog).getByRole('button', { name: named(message(language, 'common-retry')) }),
      ).toBeInTheDocument();
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      });
    });
  });
});

describe('the draft screen keeps the discard of its draft out of a saved view', () => {
  it('opening a saved بکری asks the backend nothing about drafts', async () => {
    resetClientState('en');
    signedIn(sessionUser({ language: 'en' }));
    fakeBikri();
    await renderApp('/bikri/book?id=41');
    await waitFor(() => {
      expect(bikriApi.bikriGet).toHaveBeenCalled();
    });
    expect(draftsApi.draftsSave).not.toHaveBeenCalled();
    expect(draftsApi.draftsDiscard).not.toHaveBeenCalled();
  });
});
