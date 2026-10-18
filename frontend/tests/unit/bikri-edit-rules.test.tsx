// The edit rules of a saved بکری told in words (plan 16, PRD FR-12.18 to FR-12.20), in the Urdu and
// the English app. The backend refuses an edit it does not allow with ONE code,
// ERR_EDIT_NOT_ALLOWED, whose field is the lot or the line and whose value `rule` is a fixed name.
// For each rule the screen shows its own words (what is wrong and what to do), at the place the
// field points to, keeps what the user typed, and never shows a code. The numbers are the
// backend's; the screen works out none.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16, refused } from '../helpers/bikri-plan16';
import { cellOf, named, salesGrid, textOf } from '../helpers/bikri-screen';
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
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

/** Every rule the backend names, the field it points to, and the entry it can come from. */
const RULES = [
  { rule: 'day_changed', field: 'header.entry_date', entry: 7 },
  { rule: 'lot_removed', field: 'lots', entry: 7 },
  { rule: 'lot_not_in_posting', field: 'lots[0].lot_id', entry: 7 },
  { rule: 'lot_order_changed', field: 'lots[0].lot_id', entry: 7 },
  { rule: 'lot_seller_locked', field: 'lots[0].seller_account_id', entry: 7 },
  { rule: 'lot_item_locked', field: 'lots[0].item_id', entry: 7 },
  { rule: 'line_not_in_posting', field: 'lots[0].sales[0].sale_id', entry: 7 },
  { rule: 'posting_lots_locked', field: 'lots', entry: 9 },
] as const;

describe.each(LANGUAGES)('the edit rules in words (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const buyer = (entry: number) =>
    entry === 7
      ? language === 'ur'
        ? 'اکرم ٹول ٹیکس'
        : 'Akram Toll Tax'
      : language === 'ur'
        ? 'ارشد ملتانی'
        : 'Arshad Multani';

  /** The entry open for edit, a new rate typed and F10 pressed, so the backend can refuse it. */
  const openAndSave = async (entry: number, field: string, rule: string) => {
    const plan = fakePlan16();
    vi.mocked(bikriApi.bikriEditSave).mockRejectedValue(
      refused('ERR_EDIT_NOT_ALLOWED', field, { rule }),
    );
    await renderApp(`/bikri/book?id=41&edit=${String(entry)}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-edit-title', { number: 1 })),
    });
    await waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'buyer')).toBe(buyer(entry));
    });
    const user = userEvent.setup();
    await user.click(cellOf(salesGrid(language), 0, 'rate'));
    await user.keyboard('610{Enter}');
    press('F10');
    return plan;
  };

  it.each(RULES)('$rule ($field) is told in its own words and keeps the edit', async (item) => {
    await openAndSave(item.entry, item.field, item.rule);
    const words = message(language, 'err-edit-not-allowed', { rule: item.rule });
    expect(await screen.findByText(said(words))).toBeInTheDocument();
    // The edit stays on screen as typed, and nothing is shown as a code.
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('610');
    expect(visible(document.body.textContent)).not.toContain('ERR_EDIT_NOT_ALLOWED');
    expect(visible(document.body.textContent)).not.toContain(item.rule);
  });

  it('each rule has words of its own: no two rules say the same, and none is the plain fallback', () => {
    const texts = RULES.map((item) =>
      message(language, 'err-edit-not-allowed', { rule: item.rule }),
    );
    const other = message(language, 'err-edit-not-allowed', { rule: 'a_rule_not_in_this_build' });
    expect(new Set(texts).size).toBe(texts.length);
    for (const text of texts) expect(text).not.toBe(other);
    // A rule from a newer backend still gets plain words, never the code.
    expect(other.length).toBeGreaterThan(10);
    expect(other).not.toContain('a_rule_not_in_this_build');
  });

  it('a refusal above the grid he is typing in is brought into view, and the focus stays', async () => {
    const scroll = vi.spyOn(Element.prototype, 'scrollIntoView');
    await openAndSave(7, 'lots[0].item_id', 'lot_item_locked');
    const words = message(language, 'err-edit-not-allowed', { rule: 'lot_item_locked' });
    const alert = await screen.findByText(said(words));
    await waitFor(() => {
      expect(scroll).toHaveBeenCalledWith({ block: 'nearest' });
    });
    // It scrolled to the list that holds the words, not somewhere else.
    expect(scroll.mock.contexts.some((target) => (target as Element).contains(alert))).toBe(true);
    expect(salesGrid(language).contains(document.activeElement)).toBe(true);
  });

  it('a lot or line the rule points at is named in the list', async () => {
    await openAndSave(7, 'lots[0].sales[0].sale_id', 'line_not_in_posting');
    expect(
      await screen.findByText(said(message(language, 'bikri-where-line', { lot: 1, line: 1 }))),
    ).toBeInTheDocument();
  });
});
