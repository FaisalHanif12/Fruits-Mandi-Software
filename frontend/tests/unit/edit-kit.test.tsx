// The one edit kit (plan 44, PRD FR-7.70, FR-7.71, FR-21.4), in the Urdu and the English app: the two
// buttons and what the flags make of them, the list of changes with its effect (the backend's own
// review, copied from its contract test), the partner figures that are behind the code, the edit
// stamp and the history. Nothing here works out a number; the screen shows what the backend sent.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { EditPreview } from '../../src/bindings/EditPreview';
import type { ChangeValue } from '../../src/bindings/ChangeValue';
import { LANGUAGES, bundles, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as editApi from '../../src/shared/edit/api';
import {
  clockPartsOf,
  changeValueText,
  fieldMessageId,
  lockedMessageId,
  placeText,
  type ValueContext,
} from '../../src/shared/edit';
import {
  EditReviewCard,
  EditStampLine,
  EditedMark,
  EntryActions,
  usePartnerCodeDialog,
} from '../../src/shared/ui';
import { ReactLocalization } from '@fluent/react';
import {
  AUDIT_EDIT_HISTORY_BIKRI,
  AUDIT_EDIT_STAMP_BIKRI,
  BIKRI_EDIT_REVIEW_AKRAM_610,
  ENTRY_FLAGS,
  FRUIT_EDIT_PREVIEW_HOME_3,
  VOUCHERS_EDIT_PREVIEW_RECEIPT_4000,
} from '../helpers/edit-real';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { printDataFor } from '../helpers/print-data';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/edit/api');

beforeEach(() => {
  vi.resetAllMocks();
});

// The names in a list of changes follow the language of the app (displayName), so the store is set.
function useAppLanguage(language: Language) {
  useLanguageStore.setState({ language });
}

function withQuery(ui: React.ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{ui}</QueryClientProvider>;
}

const named = (text: string) => (name: string) => visible(name) === text;
const startsWith = (text: string) => (name: string) => visible(name).startsWith(text);

describe.each(LANGUAGES)('the edit kit (%s)', (language: Language) => {
  beforeEach(() => {
    useAppLanguage(language);
  });

  describe('the two buttons', () => {
    const noop = () => undefined;

    it('an entry of an open day: Edit and Cancel are on', () => {
      renderIn(language, <EntryActions flags={ENTRY_FLAGS.open} onEdit={noop} onCancel={noop} />);
      expect(
        screen.getByRole('button', { name: startsWith(message(language, 'edit-action-edit')) }),
      ).toBeEnabled();
      expect(
        screen.getByRole('button', { name: startsWith(message(language, 'edit-action-cancel')) }),
      ).toBeEnabled();
    });

    it('an entry of a closed day: Edit reads "reopen the day and fix" (PRD FR-21.4)', async () => {
      const onEdit = vi.fn();
      renderIn(
        language,
        <EntryActions flags={ENTRY_FLAGS.closed_day} onEdit={onEdit} onCancel={noop} />,
      );
      const edit = screen.getByRole('button', {
        name: startsWith(message(language, 'edit-action-fix-day')),
      });
      expect(edit).toBeEnabled();
      await userEvent.setup().click(edit);
      expect(onEdit).toHaveBeenCalledTimes(1);
    });

    it('a locked entry has both buttons off and says why in words, not only in a tooltip', () => {
      renderIn(
        language,
        <EntryActions flags={ENTRY_FLAGS.goods_moved} onEdit={noop} onCancel={noop} />,
      );
      expect(
        screen.getByRole('button', { name: startsWith(message(language, 'edit-action-edit')) }),
      ).toBeDisabled();
      expect(
        screen.getByRole('button', { name: startsWith(message(language, 'edit-action-cancel')) }),
      ).toBeDisabled();
      expect(visible(document.body.textContent)).toContain(
        message(language, 'edit-locked-goods-moved'),
      );
    });

    it('a record without a cancel (a master) shows Edit alone', () => {
      renderIn(
        language,
        <EntryActions flags={ENTRY_FLAGS.open} onEdit={noop} onCancel={noop} noCancel />,
      );
      expect(
        screen.queryByRole('button', { name: startsWith(message(language, 'edit-action-cancel')) }),
      ).toBeNull();
    });

    it('every locked reason the backend can send has words in this language', () => {
      for (const reason of [
        'goods_moved',
        'lot_seller_locked',
        'garden_closed',
        'month_closed',
        'cancelled',
        'returns_stand',
      ] as const) {
        const id = lockedMessageId(reason);
        expect(id).not.toBeNull();
        expect(bundles[language].hasMessage(id ?? '')).toBe(true);
      }
      expect(lockedMessageId(null)).toBeNull();
    });
  });

  describe('what was changed and its effect', () => {
    it("اکرم's rate 600 to 610 (E2E-13): each change in words, then the balances and the partners", () => {
      renderIn(language, <EditReviewCard review={BIKRI_EDIT_REVIEW_AKRAM_610.review} />);
      const text = visible(document.body.textContent);
      const where = `${message(language, 'edit-review-where-line', {
        lot: 1,
        line: 1,
        label: ` (${language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax'})`,
      })} `;
      expect(text).toContain(
        message(language, 'edit-review-change', {
          where,
          field: message(language, 'edit-field-seller-rate'),
          before: '600',
          after: '610',
        }),
      );
      expect(text).toContain(
        message(language, 'edit-review-change', {
          where,
          field: message(language, 'edit-field-buyer-rate'),
          before: '600',
          after: '610',
        }),
      );
      // اکرم owes 12,880 and then 13,092; رحیم is owed 4,316 and then 4,504; the shares 1,580 to 1,590.
      for (const figure of [
        '12,880',
        '13,092',
        '4,316',
        '4,504',
        '1,580',
        '1,590',
        '1,184',
        '1,191',
      ]) {
        expect(text).toContain(figure);
      }
    });

    it('nothing changed says so, and shows no effect', () => {
      const none: EditPreview = {
        errors: [],
        review: { ...BIKRI_EDIT_REVIEW_AKRAM_610.review, changes: [] },
      };
      renderIn(language, <EditReviewCard review={none.review} />);
      expect(visible(document.body.textContent)).toContain(message(language, 'edit-review-none'));
      expect(visible(document.body.textContent)).not.toContain(
        message(language, 'edit-review-effect-title'),
      );
    });

    it('the cash before and after comes from the backend (a receipt of 10,000 changed to 4,000)', () => {
      renderIn(language, <EditReviewCard review={VOUCHERS_EDIT_PREVIEW_RECEIPT_4000.review} />);
      expect(visible(document.body.textContent)).toContain(
        message(language, 'edit-review-effect-cash', { before: '58,484', after: '52,484' }),
      );
    });

    it('fruit: the quantity and the value of the entry are listed, with the partner who pays', () => {
      renderIn(language, <EditReviewCard review={FRUIT_EDIT_PREVIEW_HOME_3.review} />);
      const text = visible(document.body.textContent);
      expect(text).toContain(message(language, 'edit-field-qty'));
      expect(text).toContain('1,160');
      expect(text).toContain('1,740');
    });

    it('partner figures behind the code are left out and a box takes their place', async () => {
      usePartnerCodeDialog.getState().hide();
      const locked = {
        ...BIKRI_EDIT_REVIEW_AKRAM_610.review,
        effect: {
          ...BIKRI_EDIT_REVIEW_AKRAM_610.review.effect,
          partners: [],
          partners_locked: true,
        },
      };
      renderIn(language, <EditReviewCard review={locked} />);
      const text = visible(document.body.textContent);
      expect(text).toContain(message(language, 'partners-code-box-text'));
      // Not shown as 0, and not shown at all.
      expect(text).not.toContain('1,590');
      await userEvent
        .setup()
        .click(
          screen.getByRole('button', { name: named(message(language, 'partners-code-enter')) }),
        );
      // The button opens the code box that the shell shows.
      expect(usePartnerCodeDialog.getState().open).toBe(true);
      usePartnerCodeDialog.getState().hide();
    });

    it('a review that is on its way says so', () => {
      renderIn(language, <EditReviewCard review={undefined} pending />);
      expect(visible(document.body.textContent)).toContain(
        message(language, 'edit-review-loading'),
      );
    });

    it('a review that could not be made says so, and the save is still the backend’s to check', () => {
      renderIn(
        language,
        <EditReviewCard review={undefined} error={{ ok: false, errors: [], ref: 'E-1' }} />,
      );
      expect(screen.getByRole('alert').textContent).toContain(
        message(language, 'edit-review-failed'),
      );
    });
  });

  describe('the words of a value and of a field', () => {
    const l10n = new ReactLocalization([bundles[language]]);
    const ctx: ValueContext = { l10n, language, grouping: 'international' };

    it('each kind of value is shown the way the screen shows it', () => {
      const cases: [ChangeValue, string][] = [
        [{ type: 'empty' }, message(language, 'edit-empty')],
        [{ type: 'text', text: 'LES-1234' }, 'LES-1234'],
        [{ type: 'number', value: 20 }, '20'],
        [{ type: 'money', paisa: 61000 as never }, '610'],
        [{ type: 'weight', grams: 12345 }, '12.345'],
        [{ type: 'bps', bps: 250 }, `2.5${message(language, 'common-percent-sign')}`],
        [{ type: 'date', iso: '2026-10-10' }, '10-10-2026'],
        [{ type: 'names', ur: 'اکرم', en: 'Akram' }, language === 'ur' ? 'اکرم' : 'Akram'],
        [{ type: 'flag', on: true }, message(language, 'common-yes')],
        [{ type: 'flag', on: false }, message(language, 'common-no')],
        [{ type: 'choice', word: 'credit' }, 'credit'],
      ];
      for (const [value, expected] of cases) {
        expect(changeValueText(value, ctx)).toBe(expected);
      }
    });

    it('a place starts the sentence: the header, a lot, a line with its buyer', () => {
      expect(
        placeText(
          {
            kind: 'entry',
            lot_no: null,
            lot_counter: null,
            item_ur: null,
            item_en: null,
            line_no: null,
            label_ur: null,
            label_en: null,
          },
          ctx,
        ),
      ).toBe('');
      expect(
        visible(
          placeText(
            {
              kind: 'lot',
              lot_no: 2,
              lot_counter: null,
              item_ur: null,
              item_en: null,
              line_no: null,
              label_ur: null,
              label_en: null,
            },
            ctx,
          ),
        ),
      ).toContain(message(language, 'edit-review-where-lot', { lot: 2, name: '' }));
      expect(
        placeText(
          {
            kind: 'header',
            lot_no: null,
            lot_counter: null,
            item_ur: null,
            item_en: null,
            line_no: null,
            label_ur: null,
            label_en: null,
          },
          ctx,
        ),
      ).toContain(message(language, 'edit-review-where-header'));
    });

    it('every field key in the backend files has a name in this language', () => {
      const keys = new Set<string>();
      for (const answer of [
        BIKRI_EDIT_REVIEW_AKRAM_610,
        VOUCHERS_EDIT_PREVIEW_RECEIPT_4000,
        FRUIT_EDIT_PREVIEW_HOME_3,
      ]) {
        for (const change of answer.review.changes) keys.add(change.field);
      }
      expect(keys.size).toBeGreaterThan(3);
      for (const key of keys) {
        expect(bundles[language].hasMessage(fieldMessageId(key))).toBe(true);
      }
    });
  });

  describe('the edit stamp and the history', () => {
    const stamp = AUDIT_EDIT_STAMP_BIKRI[0]?.stamp ?? null;

    it('the clock is told in the words of the language: 16:35 is "4:35" with its part of the day', () => {
      expect(clockPartsOf('16:35')).toEqual({
        hour: '4',
        minute: '35',
        part: 'evening',
        meridian: 'pm',
      });
      expect(clockPartsOf('00:05')).toMatchObject({ hour: '12', part: 'night', meridian: 'am' });
      expect(clockPartsOf('09:10')).toMatchObject({ hour: '9', part: 'morning', meridian: 'am' });
      expect(clockPartsOf('12:00')).toMatchObject({ hour: '12', part: 'noon', meridian: 'pm' });
      expect(clockPartsOf('late')).toBeNull();
    });

    it('the header line tells when, by whom and why; a second edit adds how many times', () => {
      if (stamp === null) throw new Error('the fixture has a stamp');
      renderIn(language, <EditStampLine stamp={{ ...stamp, count: 2 }} />);
      const text = visible(document.body.textContent);
      expect(text).toContain(
        message(language, 'edit-stamp', {
          date: '11-10-2026',
          time: message(language, 'edit-time', {
            hour: '2',
            minute: '00',
            part: 'noon',
            meridian: 'pm',
          }),
          by: 'Maher Adnan',
        }),
      );
      expect(text).toContain(message(language, 'edit-stamp-count', { count: 2 }));
      expect(text).toContain(
        message(language, 'edit-stamp-reason', { reason: stamp.reason ?? '' }),
      );
    });

    it('the row mark says "edited" in words and carries the time and the name', () => {
      if (stamp === null) throw new Error('the fixture has a stamp');
      renderIn(language, <EditedMark stamp={stamp} />);
      const mark = screen.getByText(message(language, 'edit-mark'));
      expect(mark.closest('[data-edited-mark]')?.getAttribute('title')).toContain('Maher Adnan');
    });

    it('the history lists every version with the old and the new value of each field', async () => {
      if (stamp === null) throw new Error('the fixture has a stamp');
      vi.mocked(editApi.auditEditHistory).mockResolvedValue(AUDIT_EDIT_HISTORY_BIKRI);
      renderIn(
        language,
        withQuery(<EditStampLine stamp={stamp} history={{ entity: 'bikri', id: 1 }} />),
      );
      const user = userEvent.setup();
      await user.click(
        screen.getByRole('button', { name: startsWith(message(language, 'edit-action-history')) }),
      );
      const dialog = await screen.findByRole('dialog');
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain('lots[0].sales[0].seller_rate_paisa');
      });
      const text = visible(dialog.textContent);
      // 600 becomes 610 and the buyer's total 12,880 becomes 13,092.
      for (const figure of ['600', '610', '12,880', '13,092']) expect(text).toContain(figure);
      expect(text).toContain(message(language, 'edit-history-action-edit'));
      expect(editApi.auditEditHistory).toHaveBeenCalledWith('bikri', 1);
      await user.click(
        within(dialog).getByRole('button', {
          name: named(message(language, 'edit-history-close')),
        }),
      );
    });
  });

  describe('the print of an edited entry', () => {
    it('a reprint after a correction says it is revised, with the day; the first print says nothing', () => {
      const revised = printDataFor('seller_bill', { language, revisedOn: '11-10-2026' });
      const first = printDataFor('seller_bill', { language });
      const { container, unmount } = renderIn(language, <BikriPrintDocument data={revised} />);
      expect(visible(container.textContent)).toContain(
        message(language, 'edit-print-revised', { date: '11-10-2026' }),
      );
      unmount();
      const again = renderIn(language, <BikriPrintDocument data={first} />);
      expect(visible(again.container.textContent)).not.toContain(
        message(language, 'edit-print-revised', { date: '11-10-2026' }),
      );
    });
  });
});
