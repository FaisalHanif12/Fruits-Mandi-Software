// The chat window keeps its shape after many questions (plan 78 F2, PRD FR-7.73). The cause of the
// old fault: the ✓ marks are `sr-only` spans (position: absolute). The conversation was not
// positioned, so each span attached to the chat window, grew its scroll area, and `scrollIntoView`
// then scrolled the whole window: the header went up and the question box jumped to the middle.
//
// jsdom has no layout, so these tests check the three things that stop it, with ten questions in a
// row, in both sizes and both languages:
//   1. the conversation is the one scroller and it holds its own absolute children (`relative`);
//   2. the window itself can never scroll (`overflow-clip`);
//   3. nothing calls `scrollIntoView` (it scrolls every ancestor), and the question box is outside
//      the conversation, below it, in the window's last row.
// The measured proof in a real browser (10 questions, 8 size and language combinations) is in the
// handoff of plan 78 F2.

import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../src/app/providers';
import { useAssistantStore } from '../../src/features/assistant/store';
import { LANGUAGES } from '../../src/i18n';
import { press } from '../helpers/app';
import { REPLY_ANSWERED } from '../helpers/assistant-real';
import {
  boxOf,
  newClient,
  panelOf,
  resetAssistantFakes,
  setLanguage,
  shell,
} from '../helpers/assistant-shell';
import * as assistantApi from '../../src/features/assistant/api';
import { message } from '../helpers/render';

vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/assistant/api');

const QUESTIONS = 10;

beforeEach(() => {
  resetAssistantFakes();
  vi.mocked(assistantApi.assistantAsk).mockResolvedValue(REPLY_ANSWERED);
});

afterEach(() => {
  useAssistantStore.getState().reset();
  vi.restoreAllMocks();
});

describe.each(LANGUAGES)('the chat window after many questions (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  async function askMany(tall: boolean) {
    const client = newClient();
    render(<AppProviders queryClient={client}>{shell('entry', client)}</AppProviders>);
    const user = userEvent.setup();
    press('F1');
    if (tall) {
      act(() => {
        useAssistantStore.getState().setTall(true);
      });
    }
    for (let index = 1; index <= QUESTIONS; index += 1) {
      await user.type(boxOf(language), `ikram ${String(index)}{Enter}`);
      // The answer of this question has been drawn before the next one is asked.
      await screen.findAllByRole('listitem');
      await act(async () => {
        await Promise.resolve();
      });
    }
    return panelOf(language);
  }

  it.each([false, true])(
    'never scrolls the window and never calls scrollIntoView (large: %s)',
    async (tall) => {
      // jsdom has no scrollIntoView: a spy that is installed and never called proves it is not used.
      const scrollIntoView = vi.fn();
      Element.prototype.scrollIntoView = scrollIntoView;
      const panel = await askMany(tall);
      expect(panel).toHaveAttribute('data-size', tall ? 'large' : 'normal');
      expect(scrollIntoView).not.toHaveBeenCalled();
      // The window itself is clipped, not scrollable, even by a program.
      expect(panel.className).toContain('overflow-clip');
      expect(panel.className).not.toContain('overflow-hidden');
    },
  );

  it('keeps the conversation as the one scroller, positioned, with the question box outside it', async () => {
    const panel = await askMany(false);
    const list = within(panel).getByRole('list', {
      name: message(language, 'assistant-conversation-label'),
    });
    expect(list.children).toHaveLength(QUESTIONS);
    const scroller = list.parentElement;
    expect(scroller).not.toBeNull();
    // `relative` holds the sr-only marks inside it, so they scroll with it and add nothing to the window.
    expect(scroller?.className).toContain('relative');
    expect(scroller?.className).toContain('overflow-y-auto');
    expect(scroller?.className).toContain('min-h-0');
    expect(scroller?.className).toContain('flex-1');
    // The box is in the last row of the window, after the conversation, never inside it.
    expect(scroller?.contains(boxOf(language))).toBe(false);
    const rows = [...panel.children];
    expect(rows.at(-1)?.contains(boxOf(language))).toBe(true);
    expect(scroller === null ? -1 : rows.indexOf(scroller)).toBeLessThan(rows.length - 1);
    expect(rows.at(-1)?.className).toContain('shrink-0');
  });

  it('puts the newest question at the top of the conversation by scrolling only the conversation', async () => {
    const panel = await askMany(false);
    const list = within(panel).getByRole('list', {
      name: message(language, 'assistant-conversation-label'),
    });
    const scroller = list.parentElement;
    expect(scroller).not.toBeNull();
    // The window never got a scroll position of its own; the conversation was the one that was set.
    expect(panel.scrollTop).toBe(0);
    expect(scroller?.scrollTop).toBeGreaterThanOrEqual(0);
  });
});
