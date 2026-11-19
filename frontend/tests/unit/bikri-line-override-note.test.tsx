// Plan 46, F2 (PRD FR-11.10): the details of a sale line say "Changing the Commission and Laga on a
// line is switched off" only when the Owner really switched it off in Settings. They used to read
// the switch from the preview, which has no answer for an empty line, so an empty line said the
// switch was off although it was on. The switch is now the Overrides setting, read the same way as
// in Settings; while it has not loaded, nothing is said to be off. Both languages.

import { act, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { OverridesSettings } from '../../src/bindings/OverridesSettings';
import * as settingsApi from '../../src/features/settings/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { previewFor } from '../helpers/bikri-data';
import { named, openBook, typeHeaderAndLot } from '../helpers/bikri-screen';
import {
  OVERRIDES_ON,
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const OVERRIDES_OFF: OverridesSettings = { ...OVERRIDES_ON, allow_line_override: false };

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** A preview that never comes: an empty line has none yet (the case of the bug). */
const NO_PREVIEW = () => new Promise<never>(() => undefined);

/** The note, or null. */
const note = (language: Language) =>
  screen.queryByText((text) => visible(text) === message(language, 'bikri-line-override-off'));

/** The details of the line that has the cursor: the card under the sale lines. */
async function details(language: Language): Promise<HTMLElement> {
  const title = await screen.findByRole('heading', {
    name: named(message(language, 'bikri-line-title', { line: 1 })),
  });
  const section = title.closest('section');
  if (section === null) throw new Error('the details of the line have no section');
  return section;
}

/** The commission and لاگا fields of the details. */
const chargeFields = (section: HTMLElement) => within(section).getAllByRole('textbox').slice(0, 3);

describe.each(LANGUAGES)('the override note of a sale line (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('an empty line with the switch on says nothing about it, and its fields can be typed in', async () => {
    vi.mocked(settingsApi.settingsGetOverrides).mockResolvedValue(OVERRIDES_ON);
    const { user } = await openBook(language, NO_PREVIEW);
    await typeHeaderAndLot(user, language);
    const section = await details(language);
    await waitFor(() => {
      expect(settingsApi.settingsGetOverrides).toHaveBeenCalled();
    });
    await waitFor(() => {
      // The method buttons only show while a line may change its charges.
      expect(within(section).getAllByRole('button').length).toBeGreaterThan(0);
    });
    expect(note(language)).toBeNull();
    chargeFields(section).forEach((field) => {
      expect(field).not.toHaveAttribute('readonly');
    });
  });

  it('the switch is the setting: a preview that says off does not switch the fields off', async () => {
    vi.mocked(settingsApi.settingsGetOverrides).mockResolvedValue(OVERRIDES_ON);
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      allow_line_override: false,
    }));
    await typeHeaderAndLot(user, language);
    const section = await details(language);
    await waitFor(() => {
      expect(settingsApi.settingsGetOverrides).toHaveBeenCalled();
    });
    await waitFor(() => {
      expect(within(section).getAllByRole('button').length).toBeGreaterThan(0);
    });
    expect(note(language)).toBeNull();
    chargeFields(section).forEach((field) => {
      expect(field).not.toHaveAttribute('readonly');
    });
  });

  it('with the switch off, an empty line says so and its fields cannot be typed in', async () => {
    vi.mocked(settingsApi.settingsGetOverrides).mockResolvedValue(OVERRIDES_OFF);
    const { user } = await openBook(language, NO_PREVIEW);
    await typeHeaderAndLot(user, language);
    const section = await details(language);
    await waitFor(() => {
      expect(note(language)).not.toBeNull();
    });
    chargeFields(section).forEach((field) => {
      expect(field).toHaveAttribute('readonly');
    });
    // No method buttons while it is off.
    expect(within(section).queryAllByRole('radio')).toHaveLength(0);
  });

  it('while the setting has not loaded, nothing is said to be off; the note comes when it answers off', async () => {
    let answer: (settings: OverridesSettings) => void = () => undefined;
    vi.mocked(settingsApi.settingsGetOverrides).mockReturnValue(
      new Promise<OverridesSettings>((resolve) => {
        answer = resolve;
      }),
    );
    const { user } = await openBook(language, NO_PREVIEW);
    await typeHeaderAndLot(user, language);
    const section = await details(language);
    await waitFor(() => {
      expect(settingsApi.settingsGetOverrides).toHaveBeenCalled();
    });
    expect(note(language)).toBeNull();
    chargeFields(section).forEach((field) => {
      expect(field).not.toHaveAttribute('readonly');
    });
    act(() => {
      answer(OVERRIDES_OFF);
    });
    await waitFor(() => {
      expect(note(language)).not.toBeNull();
    });
    chargeFields(section).forEach((field) => {
      expect(field).toHaveAttribute('readonly');
    });
  });
});
