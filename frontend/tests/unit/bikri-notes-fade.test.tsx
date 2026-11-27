// Plan 97 item 4 (PRD FR-12.52, FR-12.6, Umar 24-11-2026): every warning and error note on the بکری screen hides after 5
// seconds and shows again for 5 seconds each time the user repeats the action. The red mark on the cell stays until the
// problem is fixed, and the save is still refused while it stands.
// - The note under the sale lines (a quantity over the lot): the page, with real keys, in both languages.
// - The three kinds of notes under the grids, the refusal of a save, and the list above the lots: the pieces, with the
//   screen's count (`NoticePulseContext`) going up.
// Time is real in the page test (the grid and the preview do not work with a frozen clock) and fake in the piece tests.

import { Localized, LocalizationProvider } from '@fluent/react';
import { act, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { UiError } from '../../src/bindings/UiError';
import { LotNotes, MissingNotes, QtyNotes } from '../../src/features/bikri/components/QtyNotes';
import { TimedErrorNotice } from '../../src/features/bikri/components/TimedErrorNotice';
import { CashNote, Field, MoneyInput, TimedNote } from '../../src/shared/ui';
import { NOTICE_MS } from '../../src/features/bikri/notices';
import { LANGUAGES } from '../../src/i18n';
import { createLocalization } from '../../src/i18n/localization';
import { NoticePulseContext } from '../../src/shared/hooks';
import * as urduApi from '../../src/shared/urdu/api';
import { previewWithOverLine } from '../helpers/bikri-data';
import { seedBikriDraft } from '../helpers/bikri-seed';
import { cellOf, focusedIn, openBook, salesGrid } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
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

const SLOW_MS = { timeout: 8_000 };

describe.each(LANGUAGES)(
  'the note of a quantity over the lot fades on the screen (%s)',
  (language) => {
    beforeEach(() => {
      vi.resetAllMocks();
      healthyBackend();
      fakePartners();
      vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
      vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });
    afterEach(() => {
      leaveAppWindow();
    });
    const words = message(language, 'err-qty-exceeds-lot', { available: '50' });
    const notes = () =>
      screen.queryAllByRole('alert').filter((note) => visible(note.textContent).includes(words));

    it('hides after 5 seconds with the red mark staying, and Enter on the refused quantity shows it again', async () => {
      seedBikriDraft({ lot: 'rahim', chuktaBuyer: true, line: { qty: '50', rate: '1200' } });
      const { user } = await openBook(language, previewWithOverLine);
      const sales = salesGrid(language);
      await waitFor(() => {
        expect(cellOf(sales, 0, 'qty').textContent).toContain('50');
      }, SLOW_MS);
      await user.click(cellOf(sales, 0, 'qty'));
      await user.keyboard('70{Enter}');
      await waitFor(() => {
        expect(notes()).toHaveLength(1);
      }, SLOW_MS);
      // It goes by itself after 5 seconds, and the cell stays red.
      await waitFor(
        () => {
          expect(notes()).toHaveLength(0);
        },
        { timeout: NOTICE_MS + 3_000 },
      );
      expect(cellOf(sales, 0, 'qty')).toHaveClass('cell-error');
      // The cursor is held on the refused quantity. The same key again shows the words again.
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('0:qty');
      }, SLOW_MS);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(notes()).toHaveLength(1);
      }, SLOW_MS);
      expect(cellOf(sales, 0, 'qty')).toHaveClass('cell-error');
    }, 40_000);
  },
);

describe.each(LANGUAGES)(
  'the notes under the grids and the refusal of a save fade (%s)',
  (language) => {
    beforeEach(() => {
      vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'Date'] });
    });
    afterEach(() => {
      vi.useRealTimers();
    });
    const draw = (pulse: number, ui: React.ReactNode) => (
      <LocalizationProvider l10n={createLocalization(language)}>
        <NoticePulseContext.Provider value={pulse}>{ui}</NoticePulseContext.Provider>
      </LocalizationProvider>
    );
    const pass = (ms: number) => {
      act(() => {
        vi.advanceTimersByTime(ms);
      });
    };

    it('QtyNotes: shows at once, hides after 5 s, shows again when the screen repeats the action', () => {
      const ui = (
        <QtyNotes
          notes={[{ rowId: 'r1', lot: 0, line: 0, words: 'only 50 are left' }]}
          whole={false}
        />
      );
      const view = render(draw(0, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
      pass(NOTICE_MS - 100);
      expect(screen.queryAllByRole('alert')).toHaveLength(1);
      pass(200);
      expect(screen.queryAllByRole('alert')).toHaveLength(0);
      // Not a user action: a new answer with the same note does not bring it back.
      view.rerender(draw(0, ui));
      expect(screen.queryAllByRole('alert')).toHaveLength(0);
      view.rerender(draw(1, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
      pass(NOTICE_MS + 100);
      expect(screen.queryAllByRole('alert')).toHaveLength(0);
    });

    it('MissingNotes: the same', () => {
      const ui = (
        <MissingNotes notes={[{ rowId: 'r1', lot: 0, line: 1, words: 'Enter the rate.' }]} whole />
      );
      const view = render(draw(0, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
      pass(NOTICE_MS + 100);
      expect(screen.queryAllByRole('alert')).toHaveLength(0);
      view.rerender(draw(1, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
    });

    it('LotNotes: the same', () => {
      const ui = <LotNotes notes={[{ rowId: 'l1', lot: 1, words: 'Choose the packing.' }]} />;
      const view = render(draw(0, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
      pass(NOTICE_MS + 100);
      expect(screen.queryAllByRole('alert')).toHaveLength(0);
      view.rerender(draw(1, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
    });

    it('the refusal of a save: shows when it comes, hides after 5 s, and shows again on the next try to save', () => {
      const refused: UiError = {
        ok: false,
        ref: 'REF-1',
        errors: [
          { code: 'ERR_QTY_EXCEEDS_LOT', field: 'lots[0].qty_received', values: { available: 5 } },
        ],
      };
      const ui = <TimedErrorNotice error={refused} />;
      const view = render(draw(0, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
      const words = visible(screen.getByRole('alert').textContent);
      expect(words).toContain(visible(message(language, 'err-qty-exceeds-lot', { available: 5 })));
      pass(NOTICE_MS + 100);
      expect(screen.queryAllByRole('alert')).toHaveLength(0);
      // The user tries to save again: the same refusal is back for another 5 seconds.
      view.rerender(draw(1, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
      pass(NOTICE_MS + 100);
      expect(screen.queryAllByRole('alert')).toHaveLength(0);
    });

    it('TimedNote (the problem of the garden’s goods arrived): the same', () => {
      const ui = (
        <TimedNote>
          <div role="alert">
            <Localized id="bikri-sales-sold-out" />
          </div>
        </TimedNote>
      );
      const view = render(draw(0, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
      pass(NOTICE_MS + 100);
      expect(screen.queryAllByRole('alert')).toHaveLength(0);
      view.rerender(draw(1, ui));
      expect(screen.getAllByRole('alert')).toHaveLength(1);
    });

    // Plan 97 item 4, Umar 24-11-2026: the words under a field of the sale book's panels fade too. The red mark and `aria-invalid`
    // stay, and the words stay in a line only a screen reader reads.
    const fieldWith = (fadeError: boolean) => (
      <Field
        label={<Localized id="glossary-kiraya" />}
        error={<Localized id="bikri-sales-sold-out" />}
        fadeError={fadeError}
      >
        {(props) => (
          <MoneyInput
            {...props}
            value=""
            onChange={() => {
              /* no change in this test */
            }}
          />
        )}
      </Field>
    );
    const words = () =>
      screen.queryAllByText((text) => visible(text) === message(language, 'bikri-sales-sold-out'));
    const visibleWords = () =>
      words().filter((node) => !node.closest('.sr-only') && !node.classList.contains('sr-only'));

    it('Field with fadeError: the words hide after 5 s, the red mark stays, the words stay for a screen reader, and a repeat shows them', () => {
      const view = render(draw(0, fieldWith(true)));
      expect(visibleWords()).toHaveLength(1);
      pass(NOTICE_MS + 100);
      expect(visibleWords()).toHaveLength(0);
      // The control is still marked invalid and still described by the words.
      const box = screen.getByRole('textbox');
      expect(box).toHaveAttribute('aria-invalid', 'true');
      const describedBy = box.getAttribute('aria-describedby') ?? '';
      expect(document.getElementById(describedBy)?.textContent).toContain(
        message(language, 'bikri-sales-sold-out').slice(0, 6),
      );
      view.rerender(draw(1, fieldWith(true)));
      expect(visibleWords()).toHaveLength(1);
    });

    it('Field without fadeError keeps its words under the field (every other form)', () => {
      render(draw(0, fieldWith(false)));
      pass(NOTICE_MS + 100);
      expect(visibleWords()).toHaveLength(1);
    });

    it('CashNote with fade: the amber note hides after 5 s and comes back on a repeat; without fade it stays', () => {
      const cash = {
        needs_confirm: true,
        cash_after_paisa: -500_000,
      } as unknown as Parameters<typeof CashNote>[0]['cash'];
      const view = render(draw(0, <CashNote cash={cash} fade />));
      const note = () =>
        screen.queryAllByText((text) =>
          visible(text).includes(message(language, 'common-cash-confirm-title')),
        );
      expect(note()).toHaveLength(1);
      pass(NOTICE_MS + 100);
      expect(note()).toHaveLength(0);
      view.rerender(draw(1, <CashNote cash={cash} fade />));
      expect(note()).toHaveLength(1);
      view.rerender(draw(1, <CashNote cash={cash} />));
      pass(NOTICE_MS + 100);
      expect(note()).toHaveLength(1);
    });
  },
);
