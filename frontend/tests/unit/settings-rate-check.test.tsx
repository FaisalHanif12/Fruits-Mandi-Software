// Settings, Overrides tab: the rate check (PRD FR-7.55, Section 21). A tick turns the warning on or
// off, and a whole percent says how far apart a rate may be. The percent is text while it is typed
// and becomes a number only when it is sent. Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as settingsApi from '../../src/features/settings/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { fakeRules } from '../helpers/rules';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/rules/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the rate check setting (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function openOverrides() {
    await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('tab', { name: named(message(language, 'settings-tab-overrides')) }),
    );
    const tick = await screen.findByRole('checkbox', {
      name: named(message(language, 'settings-overrides-rate-check')),
    });
    const percent = screen.getByRole<HTMLInputElement>('textbox', {
      name: named(message(language, 'settings-overrides-rate-check-percent')),
    });
    return { user, tick, percent };
  }

  it('starts on, with the saved percent, and the percent can be typed', async () => {
    fakeRules();
    const { tick, percent } = await openOverrides();
    expect(tick).toBeChecked();
    expect(percent.value).toBe('30');
    expect(percent).not.toHaveAttribute('readonly');
  });

  it('a new percent is sent as a number when saved', async () => {
    fakeRules();
    const { user, percent } = await openOverrides();
    await user.clear(percent);
    await user.type(percent, '50');
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetOverrides).toHaveBeenCalledWith(
        expect.objectContaining({ rate_check_enabled: true, rate_check_percent: 50 }),
      );
    });
  });

  it('switched off, the percent cannot be changed and the tick is sent as off', async () => {
    fakeRules();
    const { user, tick, percent } = await openOverrides();
    await user.click(tick);
    expect(tick).not.toBeChecked();
    expect(percent).toHaveAttribute('readonly');
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetOverrides).toHaveBeenCalledWith(
        expect.objectContaining({ rate_check_enabled: false, rate_check_percent: 30 }),
      );
    });
  });

  it('a percent that is not a whole number is refused on the screen with the words of the range, and nothing is sent', async () => {
    fakeRules();
    const { user, percent } = await openOverrides();
    await user.clear(percent);
    await user.type(percent, '3.5');
    press('F10');
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'err-out-of-range', { min: 1, max: '1,000' }),
      );
    });
    expect(settingsApi.settingsSetOverrides).not.toHaveBeenCalled();
    // Typing again takes the message away.
    await user.clear(percent);
    await user.type(percent, '35');
    await waitFor(() => {
      expect(visible(document.body.textContent)).not.toContain(
        message(language, 'err-out-of-range', { min: 1, max: '1,000' }),
      );
    });
  });
});
