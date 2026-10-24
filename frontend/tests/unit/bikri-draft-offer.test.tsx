// "Finish the last بکری?" at start-up (plan 13, PRD FR-12.10, NFR-REL-2, E2E-17): the draft that the
// screen kept before the app closed is offered once after sign-in. Restore opens the sale book with
// it, discard throws it away, Esc keeps it for the next start, and a draft that cannot be read is
// thrown away without a word. Urdu and English, against a fake backend.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { LOT, SALE } from '../../src/features/bikri/types';
import { LANGUAGES } from '../../src/i18n';
import * as draftsApi from '../../src/shared/drafts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri, RAHIM } from '../helpers/bikri';
import { combo, lotsGrid, named, salesGrid, textOf } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
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

/** The text of a draft the screen kept: a truck, a lot of 50 نگ from Rahim, one line for Akram. */
function keptDraft(): string {
  const stores = createBikriDraft('10-10-2026');
  stores.header.getState().patch({ truckNo: 'LES-1234' });
  const lot = stores.lots.getState().rows[0];
  if (lot === undefined) throw new Error('no lot row');
  stores.lots.getState().patchRow(lot.rowId, {
    typed: { [LOT.qty]: '50', [LOT.sellerFilled]: '1' },
    picked: { [LOT.seller]: { id: RAHIM.id, code: RAHIM.code, name_ur: 'رحیم', name_en: 'Rahim' } },
  });
  const line = stores.salesFor(lot.rowId).getState().rows[0];
  if (line === undefined) throw new Error('no line row');
  stores
    .salesFor(lot.rowId)
    .getState()
    .patchRow(line.rowId, {
      typed: { [SALE.qty]: '20', [SALE.rate]: '600' },
    });
  return serialiseDraft(stores);
}

beforeEach(() => {
  // Reset, not only clear: a rejection queued with mockRejectedValueOnce by a test that failed early
  // must not reach the next test.
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  usePendingRestore.setState({ snapshot: null });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the unfinished بکری at start-up (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const offer = () => screen.findByRole('alertdialog');
  const button = (box: HTMLElement, id: string) =>
    within(box).getByRole('button', {
      name: (name) => visible(name).startsWith(message(language, id)),
    });

  it('offers nothing when there is no draft', async () => {
    await renderApp('/');
    await waitFor(() => {
      expect(draftsApi.draftsGet).toHaveBeenCalledWith('bikri');
    });
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  });

  it('offers the draft with restore, discard and "not now"; the focus is on restore', async () => {
    vi.mocked(draftsApi.draftsGet).mockResolvedValue({
      payload_json: keptDraft(),
      saved_at: '2026-10-10T07:30:00Z',
    });
    await renderApp('/');
    const box = await offer();
    expect(visible(box.textContent)).toContain(message(language, 'bikri-draft-title'));
    expect(button(box, 'bikri-draft-restore')).toHaveFocus();
    expect(button(box, 'bikri-draft-discard')).toBeInTheDocument();
    expect(button(box, 'bikri-draft-later')).toBeInTheDocument();
  });

  it('restore opens the sale book with everything that was typed', async () => {
    vi.mocked(draftsApi.draftsGet).mockResolvedValue({
      payload_json: keptDraft(),
      saved_at: '2026-10-10T07:30:00Z',
    });
    const { router } = await renderApp('/');
    const user = userEvent.setup();
    await user.click(button(await offer(), 'bikri-draft-restore'));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(await combo(language, 'bikri-header-truck')).toHaveValue('LES-1234');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'qty')).toBe('50');
    });
    expect(textOf(salesGrid(language), 0, 'qty')).toBe('20');
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('600');
    // The restored work is unsaved work: nothing was thrown away.
    expect(draftsApi.draftsDiscard).not.toHaveBeenCalled();
    // The offer is not asked again on this sign-in.
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  });

  it('discard throws the draft away and stays where he is', async () => {
    vi.mocked(draftsApi.draftsGet).mockResolvedValue({
      payload_json: keptDraft(),
      saved_at: '2026-10-10T07:30:00Z',
    });
    const { router } = await renderApp('/');
    const user = userEvent.setup();
    await user.click(button(await offer(), 'bikri-draft-discard'));
    await waitFor(() => {
      expect(draftsApi.draftsDiscard).toHaveBeenCalledWith('bikri');
    });
    expect(router.state.location.pathname).toBe('/');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
  });

  it('Esc keeps the draft for the next start', async () => {
    vi.mocked(draftsApi.draftsGet).mockResolvedValue({
      payload_json: keptDraft(),
      saved_at: '2026-10-10T07:30:00Z',
    });
    const { router } = await renderApp('/');
    const user = userEvent.setup();
    await offer();
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
    expect(draftsApi.draftsDiscard).not.toHaveBeenCalled();
    expect(router.state.location.pathname).toBe('/');
  });

  it('a draft that cannot be read is thrown away without a word', async () => {
    vi.mocked(draftsApi.draftsGet).mockResolvedValue({
      payload_json: '{"v":1,"half":',
      saved_at: '2026-10-10T07:30:00Z',
    });
    await renderApp('/');
    await waitFor(() => {
      expect(draftsApi.draftsDiscard).toHaveBeenCalledWith('bikri');
    });
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  });

  it('names the sale book on the side menu after restore', async () => {
    vi.mocked(draftsApi.draftsGet).mockResolvedValue({
      payload_json: keptDraft(),
      saved_at: '2026-10-10T07:30:00Z',
    });
    await renderApp('/');
    const user = userEvent.setup();
    await user.click(button(await offer(), 'bikri-draft-restore'));
    expect(
      await screen.findByRole('button', {
        name: named(message(language, 'layout-module-bikri-book')),
      }),
    ).toHaveAttribute('aria-current', 'page');
  });
});
