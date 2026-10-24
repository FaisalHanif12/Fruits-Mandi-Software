// A saved بکری opened read-only, the arrows between بکری, find and "new" (plan 13, PRD FR-12.13),
// in the Urdu and the English app, against a fake backend. A saved بکری shows the values stored
// when it was saved, never worked out again, and cannot be typed in.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as draftsApi from '../../src/shared/drafts/api';
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

describe.each(LANGUAGES)('a saved بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const open = () => renderApp('/bikri/book?id=41');

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
        name: named(message(language, 'bikri-summary-lot', { lot: 1 })),
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
        name: named(message(language, 'bikri-summary-lot', { lot: 2 })),
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

  it('a one-time buyer is shown by his own name', async () => {
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
              pukhta: {
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
    expect(
      await screen.findByRole('combobox', { name: named(message(language, 'bikri-header-truck')) }),
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
    expect(
      await screen.findByRole('combobox', { name: named(message(language, 'bikri-header-truck')) }),
    ).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('moving between بکری and find (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const arrow = (id: string) => screen.findByRole('button', { name: named(message(language, id)) });

  it('previous goes by number from the one on screen and opens the one the backend names', async () => {
    vi.mocked(bikriApi.bikriNavigate).mockResolvedValue(40);
    const { router } = await renderApp('/bikri/book?id=41');
    const user = userEvent.setup();
    await user.click(await arrow('bikri-nav-previous'));
    expect(bikriApi.bikriNavigate).toHaveBeenCalledWith(41, 'previous');
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ id: 40 });
    });
  });

  it('first, next and last ask the backend too', async () => {
    vi.mocked(bikriApi.bikriNavigate).mockResolvedValue(7);
    await renderApp('/bikri/book?id=41');
    const user = userEvent.setup();
    await user.click(await arrow('bikri-nav-first'));
    await user.click(await arrow('bikri-nav-next'));
    await user.click(await arrow('bikri-nav-last'));
    expect(vi.mocked(bikriApi.bikriNavigate).mock.calls.map((call) => call[1])).toEqual([
      'first',
      'next',
      'last',
    ]);
  });

  it('at the end of the list a short note says so and the screen stays', async () => {
    vi.mocked(bikriApi.bikriNavigate).mockResolvedValue(null);
    const { router } = await renderApp('/bikri/book?id=41');
    const user = userEvent.setup();
    await user.click(await arrow('bikri-nav-next'));
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'bikri-nav-no-next'));
    });
    expect(router.state.location.search).toMatchObject({ id: 41 });
  });

  it('from the new بکری, previous starts from nothing', async () => {
    vi.mocked(bikriApi.bikriNavigate).mockResolvedValue(41);
    const { router } = await renderApp('/bikri/book');
    const user = userEvent.setup();
    await user.click(await arrow('bikri-nav-previous'));
    expect(bikriApi.bikriNavigate).toHaveBeenCalledWith(null, 'previous');
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ id: 41 });
    });
  });

  it('a failure is told in words', async () => {
    vi.mocked(bikriApi.bikriNavigate).mockRejectedValue(
      fail({ code: 'ERR_NOT_FOUND', field: 'current_id', values: {} }),
    );
    await renderApp('/bikri/book?id=41');
    const user = userEvent.setup();
    await user.click(await arrow('bikri-nav-next'));
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-not-found'));
    });
  });

  describe('find (F3)', () => {
    const openFind = async () => {
      const view = await renderApp('/bikri/book?id=41');
      await arrow('bikri-nav-next');
      press('F3');
      const dialog = await screen.findByRole('dialog');
      return { ...view, dialog, user: userEvent.setup() };
    };

    it('F3 opens it with the five filters and a hint, and nothing is asked yet', async () => {
      const { dialog } = await openFind();
      for (const id of [
        'bikri-find-number',
        'bikri-header-truck',
        'glossary-beopari',
        'bikri-header-date',
        'glossary-qism-maal',
      ]) {
        expect(visible(dialog.textContent)).toContain(message(language, id));
      }
      expect(visible(dialog.textContent)).toContain(message(language, 'bikri-find-hint'));
      expect(bikriApi.bikriFind).not.toHaveBeenCalled();
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
