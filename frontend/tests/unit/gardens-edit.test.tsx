// A saved garden entry, goods arrived and the header open for edit (plan 44, PRD FR-7.70, FR-7.71,
// FR-21.4), in the Urdu and the English app, against a fake backend: the Edit button opens the entry
// in its form, filled as saved, under the amber band; the review comes from the backend; the reason
// is asked when the entry is not of today; a closed day reopens first; goods arrived and the facts of
// the garden are changed the same way.

import { fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
import * as urduApi from '../../src/shared/urdu/api';
import * as arrivals from '../helpers/arrivals-real';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { noChangeReview } from '../helpers/edit-real';
import { CLOSED_DAY_ENTRY } from '../helpers/entry-flags';
import { e40Garden, e40History, fakeGardens } from '../helpers/gardens';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { fakeItemPickers } from '../helpers/trips';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeItemPickers();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
const startsWith = (text: string) => (name: string) => visible(name).startsWith(text);

/** The first of some elements, or a clear failure when there are none. */
function firstOf<T>(items: readonly T[]): T {
  const [first] = items;
  if (first === undefined) throw new Error('Expected at least one element.');
  return first;
}

/** The answer of a saved edit. */
const EDITED = {
  id: 11,
  number: null,
  version: 2,
  entry_id: 40,
  reversal_entry_id: 41,
  review: noChangeReview().review,
  edited: {
    at: '',
    local_date: '2026-10-10',
    local_time: '16:35',
    by: 'Owner',
    count: 1,
    reason: '',
  },
} as never;

describe.each(LANGUAGES)('a garden entry open for edit (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    vi.mocked(gardensApi.gardensEntryEditPreview).mockResolvedValue(noChangeReview());
  });

  const open = async () => {
    const view = await renderApp('/gardens/1');
    await screen.findByText(said(message(language, 'gardens-detail-title', { code: '0080' })));
    return { ...view, user: userEvent.setup() };
  };
  const selectRow = async (user: ReturnType<typeof userEvent.setup>, key: string) => {
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        `.ag-row[row-id="${key}"] [col-id="entry"]`,
      );
      if (found === null) throw new Error(`no row ${key}`);
      return found;
    });
    await user.click(row);
  };
  const entryEdit = () =>
    screen.getAllByRole('button', { name: startsWith(message(language, 'edit-action-edit')) });
  const field = (id: string) => {
    const label = named(message(language, id));
    return (
      screen.queryByRole('textbox', { name: label }) ??
      screen.getByRole('combobox', { name: label })
    );
  };

  it('opens the installment as saved, under the band, with the day read only', async () => {
    fakeGardens();
    const { user } = await open();
    await selectRow(user, 'e11');
    // The history's Edit button (the first of the page; arrivals have their own).
    await user.click(firstOf(entryEdit()));
    expect(
      await screen.findByText(
        startsWith(
          message(language, 'edit-band', {
            what: message(language, 'glossary-bagh'),
            number: `${message(language, 'gardens-kind-installment')} 1`,
            date: '10-10-2026',
          }),
        ),
      ),
    ).toBeInTheDocument();
    expect((field('gardens-installment-amount') as HTMLInputElement).value).toBe('30,000');
    expect((field('gardens-payment-reason') as HTMLInputElement).value).toBe('First installment');
    expect(
      screen.queryByRole('group', { name: named(message(language, 'gardens-type-label')) }),
    ).toBeNull();
  });

  it('saves with gardens_entry_edit_save: id, version, the entry as typed, no reason for today', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensEntryEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    await selectRow(user, 'e11');
    await user.click(firstOf(entryEdit()));
    const amount = await screen.findByRole('textbox', {
      name: named(message(language, 'gardens-installment-amount')),
    });
    await user.clear(amount);
    await user.type(amount, '35000');
    await waitFor(() => {
      expect(gardensApi.gardensEntryEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensEntryEditSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(gardensApi.gardensEntryEditSave).mock.calls[0]?.[0]).toMatchObject({
      id: 11,
      version: 1,
      reason: '',
      entry: { type: 'installment', garden_id: 1, amount: '35000', reason: 'First installment' },
      confirm_cash_below_zero: false,
    });
    // The page goes back to the form of a new entry.
    expect(
      await screen.findByRole('group', { name: named(message(language, 'gardens-type-label')) }),
    ).toBeInTheDocument();
  });

  it('an entry of an earlier day asks why, then saves with the reason', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensEntryEditPreview).mockResolvedValue(
      noChangeReview('2026-10-10', true),
    );
    vi.mocked(gardensApi.gardensEntryEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    await selectRow(user, 'e11');
    await user.click(firstOf(entryEdit()));
    await screen.findByRole('textbox', {
      name: named(message(language, 'gardens-installment-amount')),
    });
    await waitFor(() => {
      expect(gardensApi.gardensEntryEditPreview).toHaveBeenCalled();
    });
    press('F10');
    const reason = await screen.findByRole('combobox', {
      name: named(message(language, 'edit-reason-label')),
    });
    expect(gardensApi.gardensEntryEditSave).not.toHaveBeenCalled();
    await user.type(reason, 'wrong amount{Enter}');
    await waitFor(() => {
      expect(gardensApi.gardensEntryEditSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(gardensApi.gardensEntryEditSave).mock.calls[0]?.[0]).toMatchObject({
      id: 11,
      reason: 'wrong amount',
    });
  });

  it('a closed day: Edit reads "reopen the day and fix" and asks for the reason first', async () => {
    const history = e40History().map((row) =>
      row.entry_id === 11 ? { ...row, ...CLOSED_DAY_ENTRY } : row,
    );
    fakeGardens({ history });
    const { user } = await open();
    await selectRow(user, 'e11');
    const button = firstOf(
      screen.getAllByRole('button', {
        name: startsWith(message(language, 'edit-action-fix-day')),
      }),
    );
    expect(button).toBeEnabled();
    await user.click(button);
    expect(
      await screen.findByRole('combobox', { name: named(message(language, 'edit-fix-label')) }),
    ).toBeInTheDocument();
  });

  it('a sale or fruit row is not an entry of the garden: Edit stays off', async () => {
    fakeGardens();
    const { user } = await open();
    // The sale of بکری 5 is a row of the history but belongs to the sale book.
    const sale = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="i3"] [col-id="entry"]');
      if (found === null) throw new Error('no sale row');
      return found;
    });
    await user.click(sale);
    for (const button of entryEdit()) expect(button).toBeDisabled();
  });
});

describe.each(LANGUAGES)('goods arrived and the garden header open for edit (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    vi.mocked(gardensApi.gardensArrivalEditPreview).mockResolvedValue(noChangeReview());
    vi.mocked(gardensApi.gardensHeaderEditPreview).mockResolvedValue(noChangeReview());
  });

  const open = async () => {
    const view = await renderApp('/gardens/1');
    await screen.findByText(said(message(language, 'gardens-detail-title', { code: '0080' })));
    return { ...view, user: userEvent.setup() };
  };

  it('goods arrived open as saved and save with gardens_arrival_edit_save', async () => {
    fakeGardens();
    // The real fixture: nothing of this arrival is sold, so the backend says it can be edited.
    const first = arrivals.ARRIVALS_IN_STOCK.rows[0];
    if (first === undefined) throw new Error('the real fixture has no arrival');
    expect(first.can_edit).toBe(true);
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue(arrivals.ARRIVALS_IN_STOCK);
    vi.mocked(gardensApi.gardensArrivalEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    // The arrival detail has its own Edit button, above the history's (which is off until a row has focus).
    const edit = await waitFor(() => {
      const buttons = screen.getAllByRole('button', {
        name: startsWith(message(language, 'edit-action-edit')),
      });
      const ready = buttons.find((button) => !(button as HTMLButtonElement).disabled);
      if (ready === undefined) throw new Error('no enabled Edit button');
      return ready;
    });
    await user.click(edit);
    const qty = await screen.findByRole('textbox', {
      name: named(message(language, 'goods-line-col-qty')),
    });
    expect((qty as HTMLInputElement).value).toBe(String(first.qty));
    await user.clear(qty);
    await user.type(qty, String(first.qty + 1));
    await waitFor(() => {
      expect(gardensApi.gardensArrivalEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensArrivalEditSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(gardensApi.gardensArrivalEditSave).mock.calls[0]?.[0]).toMatchObject({
      id: first.id,
      version: first.version,
      reason: '',
      arrival: { garden_id: 1, qty: String(first.qty + 1), item_id: first.item.id },
    });
  });

  it('the facts of the garden open for edit and save with gardens_header_edit_save', async () => {
    fakeGardens({ gardens: [e40Garden({ note: 'near the river' })] });
    vi.mocked(gardensApi.gardensHeaderEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    await user.click(
      screen.getByRole('button', { name: startsWith(message(language, 'gardens-header-edit')) }),
    );
    const note = await screen.findByRole('combobox', {
      name: named(message(language, 'gardens-field-note')),
    });
    expect((note as HTMLInputElement).value).toBe('near the river');
    expect(
      screen.getByLabelText<HTMLInputElement>(said(message(language, 'gardens-field-date')))
        .readOnly,
    ).toBe(true);
    fireEvent.change(note, { target: { value: 'near the canal' } });
    await waitFor(() => {
      expect(gardensApi.gardensHeaderEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensHeaderEditSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(gardensApi.gardensHeaderEditSave).mock.calls[0]?.[0];
    expect(sent).toMatchObject({ id: 1, reason: '', garden: { note: 'near the canal' } });
    // The day and the shares go back as they are.
    expect(sent?.garden.date).toBe('10-10-2026');
    expect(sent?.garden.partners).toHaveLength(2);
  });
});
