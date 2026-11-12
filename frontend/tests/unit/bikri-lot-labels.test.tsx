// Every lot label names its seller and item (PRD FR-12.6, plan 57): the heading of the sale lines,
// the lot switch of the lines area, the lot switch of the summary and the "more" button, in Urdu and
// in English. A lot with nothing
// chosen yet keeps its number alone. A language switch changes the words and loses nothing.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import { previewFor } from '../helpers/bikri-data';
import { seedBikriDraft } from '../helpers/bikri-seed';
import { named, openBook } from '../helpers/bikri-screen';
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

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

const NAMES = {
  rahim: { ur: 'رحیم', en: 'Rahim' },
  karim: { ur: 'کریم', en: 'Karim' },
  banana: { ur: 'کیلا', en: 'Banana' },
  apple: { ur: 'سیب', en: 'Apple' },
};

/**
 * The words that name a lot, "Rahim, Banana-1" / "رحیم، کیلا-1": the backend's answers carry the counter of
 * the lot's name (PRD FR-12.45), 1 for these first lots. The seller alone while no item is chosen.
 */
const who = (language: Language, seller: string, item: string | null) =>
  item === null ? seller : message(language, 'bikri-lot-name', { seller, item, counter: '1' });

/** The names alone, for a lot the backend gave no counter (a preview that says null). */
const whoWithoutCounter = (language: Language, seller: string, item: string) =>
  message(language, 'bikri-lot-who', { seller, item });

/** The values of a label that names its lot. */
const label = (_language: Language, text: string) => ({ lot: 1, who: text, named: 'yes' });

/** The four places a lot is named on the sale book: heading, two switches and the "more" button. */
function expectNamed(language: Language, text: string) {
  const vars = label(language, text);
  // The heading of the sale lines, and the switch of the lines area under the same section.
  const heading = screen.getByRole('heading', {
    name: named(message(language, 'bikri-sales-title', vars)),
  });
  const lines = heading.closest('section');
  if (lines === null) throw new Error('no lines area');
  const lotButton = message(language, 'bikri-summary-lot', vars);
  expect(within(lines).getByRole('button', { name: named(lotButton) })).toBeInTheDocument();
  // The switch of the summary.
  const summary = screen
    .getByRole('heading', { name: named(message(language, 'bikri-summary-title')) })
    .closest('section');
  if (summary === null) throw new Error('no summary');
  expect(within(summary).getByRole('button', { name: named(lotButton) })).toBeInTheDocument();
  // The "more" button of the lot.
  expect(
    screen.getByRole('button', { name: named(message(language, 'bikri-more-toggle', vars)) }),
  ).toBeInTheDocument();
}

/** The draft is put on the screen a moment after it opens, so the labels are waited for. */
const SLOW_MS = { timeout: 8_000 };
const namedSoon = (language: Language, text: string) =>
  waitFor(() => {
    expectNamed(language, text);
  }, SLOW_MS);

describe.each(LANGUAGES)('the labels of a lot (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('name the seller and the item of the lot in all four places', async () => {
    seedBikriDraft({ lot: 'rahim' });
    await openBook(language);
    await namedSoon(language, who(language, NAMES.rahim[language], NAMES.banana[language]));
  });

  it('name the other lot of the other seller by its own seller and item', async () => {
    seedBikriDraft({ lot: 'karim' });
    await openBook(language);
    await namedSoon(language, who(language, NAMES.karim[language], NAMES.apple[language]));
  });

  it('hold the name in one isolate, so a right-to-left reader sees the seller before the item', async () => {
    // Two isolates inside a third make the outer one read left to right (Unicode rule X5c), which
    // showed the item first in Urdu. The name must have no isolation marks of its own inside it.
    seedBikriDraft({ lot: 'rahim' });
    await openBook(language);
    const text = who(language, NAMES.rahim[language], NAMES.banana[language]);
    await namedSoon(language, text);
    const heading = screen.getByRole('heading', {
      name: named(message(language, 'bikri-sales-title', label(language, text))),
    });
    expect(heading.textContent).toContain(`\u2068${visible(text)}\u2069`);
  });

  it('change their words with the language and keep the lot', async () => {
    seedBikriDraft({ lot: 'rahim' });
    const { user } = await openBook(language);
    const next: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    await namedSoon(next, who(next, NAMES.rahim[next], NAMES.banana[next]));
  });
});

// Plan 71 L1 to L4, L8 (PRD FR-12.45): the name of a lot is its seller, its item and the counter the
// backend gives, "Rahim, Banana-2". A new lot shows the number it will get when it is saved, which the
// backend works out and sends in the preview (`lot_counter`, null while the seller or the item is not
// chosen). The screen only draws it.
describe.each(LANGUAGES)('the name of a lot with its counter (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** The answer of the preview with the counter 2 on the first lot, as the backend works it out. */
  const withCounter =
    (counter: number | null) =>
    (draft: BikriDraft): BikriPreview => {
      const answer = previewFor(draft);
      return {
        ...answer,
        lots: answer.lots.map((lot, index) => ({
          ...lot,
          lot_counter: index === 0 ? counter : null,
        })),
      };
    };

  const withCounterText = (counter: number) =>
    message(language, 'bikri-lot-name', {
      seller: NAMES.rahim[language],
      item: NAMES.banana[language],
      counter: String(counter),
    });

  it('shows the seller, the item and the counter in all four places', async () => {
    seedBikriDraft({ lot: 'rahim' });
    await openBook(language, withCounter(2));
    await namedSoon(language, withCounterText(2));
  });

  it('does not show a counter the backend has not given: the names alone', async () => {
    seedBikriDraft({ lot: 'rahim' });
    await openBook(language, withCounter(null));
    await namedSoon(
      language,
      whoWithoutCounter(language, NAMES.rahim[language], NAMES.banana[language]),
    );
  });

  it('keeps the counter and changes the words with the language', async () => {
    seedBikriDraft({ lot: 'rahim' });
    const { user } = await openBook(language, withCounter(3));
    await namedSoon(language, withCounterText(3));
    const next: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    await namedSoon(
      next,
      message(next, 'bikri-lot-name', {
        seller: NAMES.rahim[next],
        item: NAMES.banana[next],
        counter: '3',
      }),
    );
  });

  it('keeps the counter in the same isolate as the names, so a right-to-left reader sees it after the item', async () => {
    seedBikriDraft({ lot: 'rahim' });
    await openBook(language, withCounter(2));
    await namedSoon(language, withCounterText(2));
    const heading = screen.getByRole('heading', {
      name: named(message(language, 'bikri-sales-title', label(language, withCounterText(2)))),
    });
    // One isolate holds the whole name, with the counter inside it and no isolation marks of its own.
    expect(heading.textContent).toContain(`\u2068${visible(withCounterText(2))}\u2069`);
  });
});
