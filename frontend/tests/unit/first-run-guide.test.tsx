// The first-run guide (round 9, PRD FR-7.12): five steps (the dashboard, the انڈکس, the بکری بک, the
// وصولی and the reports) that open once the first time the Owner signs in, each opening its screen;
// Back, Next, Skip and Esc; a mark kept when it closes so it does not open again by itself; and the
// user menu shows it again. Urdu and English, by keyboard, with a language switch in the middle.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useGuideStore } from '../../src/app/guide/guide-store';
import { GUIDE_SCREEN, GUIDE_STEPS } from '../../src/app/guide/steps';
import { LANGUAGES, type Language } from '../../src/i18n';
import { SHORTCUTS } from '../../src/shared/keyboard';
import * as screenApi from '../../src/shared/screen-state/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeDashboard } from '../helpers/dashboard';
import { message, visible } from '../helpers/render';
import { must } from '../helpers/must';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

const get = vi.mocked(screenApi.prefsScreenGet);
const set = vi.mocked(screenApi.prefsScreenSet);

/** What the guide kept: nothing yet (a first sign-in), or that it was seen. */
function guideKept(seen: boolean) {
  get.mockImplementation(({ screen }) =>
    Promise.resolve({
      screen,
      value: screen === GUIDE_SCREEN && seen ? '{"done":true}' : null,
    }),
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  set.mockImplementation(({ screen, value }) =>
    Promise.resolve({ screen, value: value === '' ? null : value }),
  );
  useGuideStore.getState().close();
});

afterEach(() => {
  leaveAppWindow();
  useGuideStore.getState().close();
});

const named =
  (language: Language, id: string, vars: Record<string, string | number> = {}) =>
  (name: string) =>
    visible(name) === message(language, id, vars);

async function firstSignIn(language: Language, seen = false) {
  resetClientState(language);
  signedIn(sessionUser({ language }));
  guideKept(seen);
  return renderApp('/');
}

/** The guide card, or null while it is closed. */
function card(): HTMLElement | null {
  return document.querySelector('[data-guide-step]');
}

async function guideStep(id: string) {
  await waitFor(() => {
    expect(card()?.getAttribute('data-guide-step')).toBe(id);
  });
}

describe.each(LANGUAGES)('the first-run guide (%s)', (language) => {
  it('opens by itself at the first step the first time, with Next in focus', async () => {
    await firstSignIn(language);
    await guideStep('dashboard');
    expect(
      screen.getByRole('heading', { name: named(language, 'guide-step-dashboard-title') }),
    ).toBeInTheDocument();
    expect(visible(card()?.textContent)).toContain(
      message(language, 'guide-progress', { step: 1, total: GUIDE_STEPS.length }),
    );
    await waitFor(() => {
      expect(screen.getByRole('button', { name: named(language, 'guide-next') })).toHaveFocus();
    });
    // The first step has no Back.
    expect(
      screen.queryByRole('button', { name: named(language, 'guide-back') }),
    ).not.toBeInTheDocument();
  });

  it('does not open again once it was seen', async () => {
    await firstSignIn(language, true);
    await new Promise((resolve) => setTimeout(resolve, 400));
    expect(card()).toBeNull();
  });

  it('stays shut when its mark cannot be read, rather than open over the Owner’s work', async () => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    get.mockRejectedValue(new Error('no bridge'));
    await renderApp('/');
    await new Promise((resolve) => setTimeout(resolve, 400));
    expect(card()).toBeNull();
  });

  it('goes through the five steps, each opening its screen, and Done closes it and keeps the mark', async () => {
    const { router } = await firstSignIn(language);
    await guideStep('dashboard');
    const user = userEvent.setup();
    const next = () => screen.getByRole('button', { name: named(language, 'guide-next') });
    const stops: [string, string][] = [
      ['index', '/index'],
      ['bikri', '/bikri/book'],
      ['receipt', '/bikri/voucher'],
      ['reports', '/reports'],
    ];
    for (const [id, path] of stops) {
      await user.click(next());
      await guideStep(id);
      await waitFor(() => {
        expect(router.state.location.pathname).toBe(path);
      });
    }
    // The receipt step opened the voucher screen as وصولی.
    // The last step says Done, and the words name the keys of the screen (F10 to save).
    expect(screen.getByRole('button', { name: named(language, 'guide-done') })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: named(language, 'guide-done') }));
    await waitFor(() => {
      expect(card()).toBeNull();
    });
    await waitFor(() => {
      expect(set).toHaveBeenCalledWith({ screen: GUIDE_SCREEN, value: '{"done":true}' });
    });
  });

  it('Back goes to the step before and its screen', async () => {
    const { router } = await firstSignIn(language);
    await guideStep('dashboard');
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: named(language, 'guide-next') }));
    await guideStep('index');
    await user.click(screen.getByRole('button', { name: named(language, 'guide-back') }));
    await guideStep('dashboard');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });

  it('the bikri step names the save key in its words', async () => {
    await firstSignIn(language);
    await guideStep('dashboard');
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: named(language, 'guide-next') }));
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: named(language, 'guide-next') }));
    await guideStep('bikri');
    expect(visible(card()?.textContent)).toContain(SHORTCUTS.save.label);
  });

  it('Skip closes it and keeps the mark', async () => {
    await firstSignIn(language);
    await guideStep('dashboard');
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: named(language, 'guide-skip') }));
    await waitFor(() => {
      expect(card()).toBeNull();
    });
    await waitFor(() => {
      expect(set).toHaveBeenCalledWith({ screen: GUIDE_SCREEN, value: '{"done":true}' });
    });
  });

  it('Esc closes it, by keyboard, and keeps the mark', async () => {
    const { router } = await firstSignIn(language);
    await guideStep('dashboard');
    await userEvent.setup().keyboard('{Escape}');
    await waitFor(() => {
      expect(card()).toBeNull();
    });
    // The Esc was the guide's: the Owner stays on the screen he was on.
    expect(router.state.location.pathname).toBe('/');
    await waitFor(() => {
      expect(set).toHaveBeenCalledWith({ screen: GUIDE_SCREEN, value: '{"done":true}' });
    });
  });

  it('Enter goes on to the next step (Next has the focus)', async () => {
    await firstSignIn(language);
    await guideStep('dashboard');
    await waitFor(() => {
      expect(screen.getByRole('button', { name: named(language, 'guide-next') })).toHaveFocus();
    });
    await userEvent.setup().keyboard('{Enter}');
    await guideStep('index');
  });

  it('the user menu shows it again, from the first step, on the dashboard', async () => {
    const { router } = await firstSignIn(language, true);
    await router.navigate({ to: '/bikri/today' });
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: named(language, 'layout-user-menu') }),
    );
    await user.click(
      await screen.findByRole('menuitem', { name: named(language, 'layout-user-guide') }),
    );
    await guideStep('dashboard');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });

  it('a language switch in the middle keeps the step and changes the words', async () => {
    await firstSignIn(language);
    await guideStep('dashboard');
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: named(language, 'guide-next') }));
    await guideStep('index');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    await waitFor(() => {
      // The heading of the card itself (the screen behind it has a heading of the same word).
      expect(
        within(must(card())).getByRole('heading', {
          name: named(other, 'guide-step-index-title'),
        }),
      ).toBeInTheDocument();
    });
    expect(card()?.getAttribute('data-guide-step')).toBe('index');
    expect(visible(card()?.textContent)).toContain(
      message(other, 'guide-progress', { step: 2, total: GUIDE_STEPS.length }),
    );
  });
});
