// Plan 95 item 11 (PRD FR-12.52, Umar 23-11-2026): the lot's "more" panel has a پیشگی field, the advance the seller takes
// when he brings these goods. It is sent with the lot as text (`LotInput.advance`), nothing is worked out on the screen, it
// stays through a language switch in the middle of the entry, and a lot with nothing typed sends no advance at all. In the
// Urdu and the English app, against the fake backend of Event 1.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { named, openBook, typeEvent1Lot } from '../helpers/bikri-screen';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the پیشگی of a lot (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });
  const field = (lang: Language = language) =>
    screen.getByRole('textbox', { name: named(message(lang, 'glossary-peshgi')) });

  it('is a field of the lot’s more panel, sent as typed, and nothing is sent while it is empty', async () => {
    const { backend, user } = await openBook(language);
    await typeEvent1Lot(user, language);
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.cash_paid).toBe('5000');
    }, SLOW_MS);
    expect(backend.lastDraft().lots[0]).not.toHaveProperty('advance');
    await user.click(field());
    await user.keyboard('20000');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.advance).toBe('20000');
    }, SLOW_MS);
  });

  it('keeps what was typed through a language switch in the middle of the entry', async () => {
    const { backend, user } = await openBook(language);
    await typeEvent1Lot(user, language);
    await user.click(field());
    await user.keyboard('20000');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.advance).toBe('20000');
    }, SLOW_MS);
    const next: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    await waitFor(() => {
      expect(field(next)).toHaveValue('20,000');
    }, SLOW_MS);
    expect(backend.lastDraft().lots[0]?.advance).toBe('20000');
  });
});
