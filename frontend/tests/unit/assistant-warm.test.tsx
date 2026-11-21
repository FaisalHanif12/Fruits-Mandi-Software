// The warm-up of the model connection (plan 82, NFR-SEC-9 v1.22), in the Urdu and the English app: the
// panel calls `assistant_warm` when it opens and when its question box gets its first key, fire and
// forget. It never waits for the answer, ignores what comes back, and shows nothing when the call
// fails. The backend decides whether anything is sent at all (at most once in 5 minutes, 12 a day).

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as assistantApi from '../../src/features/assistant/api';
import { useAssistantStore } from '../../src/features/assistant/store';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { press } from '../helpers/app';
import {
  boxOf,
  panelOf,
  resetAssistantFakes,
  setLanguage,
  shell,
} from '../helpers/assistant-shell';
import { renderIn } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/assistant/api');

beforeEach(() => {
  resetAssistantFakes();
});

afterEach(() => {
  useAssistantStore.getState().reset();
});

const warmCalls = () => vi.mocked(assistantApi.assistantWarm).mock.calls.length;

describe.each(LANGUAGES)('the warm-up of the model connection (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('is asked once when the panel opens, and not while it is shut', () => {
    renderIn(language, shell());
    expect(warmCalls()).toBe(0);
    press('F1');
    expect(warmCalls()).toBe(1);
    // Closing asks nothing; opening again is a new opening.
    press('F1');
    expect(warmCalls()).toBe(1);
    press('F1');
    expect(warmCalls()).toBe(2);
  });

  it('is asked again when the first key goes into the question box, and only for the first key', async () => {
    renderIn(language, shell());
    const user = userEvent.setup();
    press('F1');
    expect(warmCalls()).toBe(1);
    // A key that holds a modifier is not typing.
    await user.keyboard('{Shift}');
    expect(warmCalls()).toBe(1);
    await user.keyboard('a');
    expect(warmCalls()).toBe(2);
    await user.keyboard('kram');
    expect(warmCalls()).toBe(2);
    expect(boxOf(language)).toHaveValue('akram');
  });

  it('asks for the first key of each opening: the box is drawn again every time the panel opens', async () => {
    renderIn(language, shell());
    const user = userEvent.setup();
    press('F1');
    await user.keyboard('a');
    expect(warmCalls()).toBe(2);
    press('F1');
    // The shut window is taken down after its fade; the next opening draws a new box.
    await waitFor(() => {
      expect(panelOf(language)).toHaveAttribute('inert');
    });
    await act(async () => {
      await new Promise((resolve) => {
        window.setTimeout(resolve, 300);
      });
    });
    press('F1');
    expect(warmCalls()).toBe(3);
    await user.keyboard('b');
    expect(warmCalls()).toBe(4);
  });

  it('is never waited for: a call that never answers changes nothing on the panel', async () => {
    vi.mocked(assistantApi.assistantWarm).mockReturnValue(new Promise(() => undefined));
    renderIn(language, shell());
    const user = userEvent.setup();
    press('F1');
    expect(panelOf(language)).not.toHaveAttribute('inert');
    await user.type(boxOf(language), 'akram{Enter}');
    // The question went out at once, whatever the warm-up is doing.
    expect(vi.mocked(assistantApi.assistantAsk)).toHaveBeenCalledTimes(1);
  });

  it.each([
    [
      'a refusal of the backend',
      () => Promise.reject(new CommandError(refusal([{ code: 'ERR_INTERNAL' }]))),
    ],
    ['an error of the bridge', () => Promise.reject(new Error('no bridge'))],
  ])('shows nothing when it fails with %s', async (_why, fail) => {
    vi.mocked(assistantApi.assistantWarm).mockImplementation(fail);
    renderIn(language, shell());
    const user = userEvent.setup();
    press('F1');
    await user.keyboard('a');
    await waitFor(() => {
      expect(warmCalls()).toBe(2);
    });
    // The panel is as it was: no message, no error box, the key was typed.
    const panel = panelOf(language);
    expect(within(panel).queryByRole('alert')).toBeNull();
    expect(screen.queryByRole('alert')).toBeNull();
    expect(boxOf(language)).toHaveValue('a');
    expect(useAssistantStore.getState().entries).toHaveLength(0);
  });
});
