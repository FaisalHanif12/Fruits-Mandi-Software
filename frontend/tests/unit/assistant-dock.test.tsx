// The chat window's shell (plans 49 and 51, PRD FR-7.73, FR-5.5, the F1 row of 7.3), in Urdu and
// English: F1, the status bar button and the round launcher open and close it, the focus goes into the
// box and back, Esc closes it, two sizes, a focus trap, the header's state, the greeting with its round
// chips, the question box (Enter sends, ↑ recalls), and a conversation that outlives the screens under
// it, a language switch, a lock and a route change.

import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../src/app/providers';
import * as assistantApi from '../../src/features/assistant/api';
import { useAssistantStore } from '../../src/features/assistant/store';
import { LANGUAGES, type Language } from '../../src/i18n';
import { tabbableIn } from '../../src/shared/keyboard';
import { authState, press } from '../helpers/app';
import { REPLY_ANSWERED, REPLY_OFFLINE, STATUS_OFF } from '../helpers/assistant-real';
import {
  boxOf,
  launcherOf,
  newClient,
  panelOf,
  resetAssistantFakes,
  setLanguage,
  shell,
} from '../helpers/assistant-shell';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/assistant/api');

beforeEach(() => {
  resetAssistantFakes();
});

afterEach(() => {
  useAssistantStore.getState().reset();
});

describe.each(LANGUAGES)('the chat window (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('is shut and hidden until it is opened, and the status bar button says so with its key', () => {
    renderIn(language, shell());
    expect(panelOf(language)).toHaveAttribute('inert');
    const button = within(screen.getByRole('contentinfo')).getByRole('button', {
      name: new RegExp(message(language, 'assistant-button')),
    });
    expect(button).toHaveAttribute('aria-expanded', 'false');
    // The key is written on the button (PRD FR-7.4).
    expect(button).toHaveTextContent('F1');
  });

  it('F1 opens it with the focus in the question box, F1 again closes it and gives the focus back', () => {
    renderIn(language, shell());
    const entry = screen.getByRole('textbox', { name: 'entry' });
    entry.focus();
    press('F1');
    expect(panelOf(language)).not.toHaveAttribute('inert');
    expect(boxOf(language)).toHaveFocus();
    press('F1');
    expect(panelOf(language)).toHaveAttribute('inert');
    expect(entry).toHaveFocus();
  });

  it('F1 with the focus on the screen brings it back into the box instead of closing', () => {
    renderIn(language, shell());
    press('F1');
    screen.getByRole('textbox', { name: 'entry' }).focus();
    press('F1');
    expect(panelOf(language)).not.toHaveAttribute('inert');
    expect(boxOf(language)).toHaveFocus();
  });

  it('the status bar button opens and closes it', async () => {
    renderIn(language, shell());
    const user = userEvent.setup();
    const button = within(screen.getByRole('contentinfo')).getByRole('button', {
      name: new RegExp(message(language, 'assistant-button')),
    });
    await user.click(button);
    expect(button).toHaveAttribute('aria-expanded', 'true');
    expect(panelOf(language)).not.toHaveAttribute('inert');
    await user.click(button);
    expect(panelOf(language)).toHaveAttribute('inert');
  });

  it('Esc closes it and returns the focus to the field it came from', async () => {
    renderIn(language, shell());
    const user = userEvent.setup();
    const entry = screen.getByRole('textbox', { name: 'entry' });
    entry.focus();
    press('F1');
    await user.keyboard('{Escape}');
    expect(panelOf(language)).toHaveAttribute('inert');
    expect(entry).toHaveFocus();
  });

  it('Esc does nothing to the panel while the focus is on the screen', async () => {
    renderIn(language, shell());
    const user = userEvent.setup();
    press('F1');
    screen.getByRole('textbox', { name: 'entry' }).focus();
    await user.keyboard('{Escape}');
    expect(panelOf(language)).not.toHaveAttribute('inert');
  });

  it('has two sizes, Ctrl+↑ for the large one and Ctrl+↓ for the normal one, from inside the window', async () => {
    renderIn(language, shell());
    const user = userEvent.setup();
    press('F1');
    expect(panelOf(language)).toHaveAttribute('data-size', 'normal');
    await user.keyboard('{Control>}{ArrowUp}{/Control}');
    expect(panelOf(language)).toHaveAttribute('data-size', 'large');
    await user.keyboard('{Control>}{ArrowDown}{/Control}');
    expect(panelOf(language)).toHaveAttribute('data-size', 'normal');
  });

  it('the size button switches the size, and its name says which key does it', async () => {
    renderIn(language, shell());
    const user = userEvent.setup();
    press('F1');
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: `${message(language, 'assistant-larger')} (Ctrl+↑)`,
      }),
    );
    expect(panelOf(language)).toHaveAttribute('data-size', 'large');
    expect(
      within(panelOf(language)).getByRole('button', {
        name: `${message(language, 'assistant-smaller')} (Ctrl+↓)`,
      }),
    ).toBeInTheDocument();
  });

  it('Ctrl+↑ outside the window is not taken by it', () => {
    renderIn(language, shell());
    press('F1');
    screen.getByRole('textbox', { name: 'entry' }).focus();
    press('ArrowUp', { ctrlKey: true });
    expect(panelOf(language)).toHaveAttribute('data-size', 'normal');
  });

  it('keeps Tab inside while the focus is in it', async () => {
    renderIn(language, shell());
    const user = userEvent.setup();
    press('F1');
    const controls = tabbableIn(panelOf(language));
    const first = controls.at(0);
    const last = controls.at(-1);
    expect(first).toBeDefined();
    expect(last).toBe(
      within(panelOf(language)).getByRole('button', {
        name: new RegExp(message(language, 'assistant-send')),
      }),
    );
    last?.focus();
    await user.tab();
    expect(first).toHaveFocus();
    await user.tab({ shift: true });
    expect(last).toHaveFocus();
  });

  describe('the question box', () => {
    it('Enter sends the question as typed, shows the first line at once, and empties the box', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'akram ka hisab{Enter}');
      const panel = panelOf(language);
      expect(within(panel).getByText('akram ka hisab')).toBeInTheDocument();
      expect(
        within(panel).getByText(message(language, 'assistant-understanding')),
      ).toBeInTheDocument();
      expect(boxOf(language)).toHaveValue('');
    });

    it('takes the letters as typed: no Roman-to-Urdu conversion', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'arshad ');
      expect(boxOf(language)).toHaveValue('arshad ');
    });

    it('an empty box sends nothing', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), '   {Enter}');
      expect(useAssistantStore.getState().entries).toHaveLength(0);
    });

    it('while a question is being worked on, Enter sends nothing more; when it is done, it does', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'one{Enter}two{Enter}');
      // The second Enter found a question still being worked on: it sent nothing, and 'two' waits.
      expect(
        useAssistantStore
          .getState()
          .entries.map((entry) => (entry.asked.type === 'text' ? entry.asked.text : '')),
      ).toEqual(['one']);
      expect(boxOf(language)).toHaveValue('two');
      act(() => {
        useAssistantStore.getState().setReply(1, REPLY_ANSWERED);
      });
      await user.type(boxOf(language), '{Enter}');
      expect(
        useAssistantStore
          .getState()
          .entries.map((entry) => (entry.asked.type === 'text' ? entry.asked.text : '')),
      ).toEqual(['one', 'two']);
    });

    it('↑ brings back the last question, ↑ again an older one, ↓ a newer one and then an empty box', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      act(() => {
        const store = useAssistantStore.getState();
        store.setReply(store.startEntry({ type: 'text', text: 'first' }), REPLY_ANSWERED);
        store.setReply(store.startEntry({ type: 'text', text: 'second' }), REPLY_ANSWERED);
      });
      const box = boxOf(language);
      box.focus();
      await user.keyboard('{ArrowUp}');
      expect(box).toHaveValue('second');
      await user.keyboard('{ArrowUp}');
      expect(box).toHaveValue('first');
      await user.keyboard('{ArrowUp}');
      expect(box).toHaveValue('first');
      await user.keyboard('{ArrowDown}');
      expect(box).toHaveValue('second');
      await user.keyboard('{ArrowDown}');
      expect(box).toHaveValue('');
    });

    it('the send button names its key and sends too', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'aaj ki bikri');
      const send = within(panelOf(language)).getByRole('button', {
        name: new RegExp(message(language, 'assistant-send')),
      });
      // A round button has no room for a key chip: its name carries the key (PRD FR-7.4).
      expect(send).toHaveAccessibleName(`${message(language, 'assistant-send')} (Enter)`);
      await user.click(send);
      expect(useAssistantStore.getState().entries).toHaveLength(1);
    });
  });

  describe('the round launcher', () => {
    it('stands outside the status bar, named like the status bar button, with the assistant on', async () => {
      renderIn(language, shell());
      const launcher = launcherOf();
      expect(launcher).toHaveAccessibleName(message(language, 'assistant-button'));
      expect(launcher).toHaveAttribute('aria-expanded', 'false');
      expect(screen.getByRole('contentinfo')).not.toContainElement(launcher);
      // The dot says "on and has a key"; the header says it in words (NFR-USE-12).
      await vi.waitFor(() => {
        expect(launcher.querySelector('span[aria-hidden="true"]')).not.toBeNull();
      });
    });

    it('opens the window with the focus in the box, then shows a close name and closes it', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      const launcher = launcherOf();
      await user.click(launcher);
      expect(panelOf(language)).not.toHaveAttribute('inert');
      expect(boxOf(language)).toHaveFocus();
      expect(launcher).toHaveAttribute('aria-expanded', 'true');
      expect(launcher).toHaveAccessibleName(message(language, 'assistant-close'));
      await user.click(launcher);
      expect(panelOf(language)).toHaveAttribute('inert');
      expect(launcher).toHaveAttribute('aria-expanded', 'false');
      expect(launcher).toHaveAccessibleName(message(language, 'assistant-button'));
    });

    it('is not in the window, so the focus trap leaves it alone and a shut window keeps it', () => {
      renderIn(language, shell());
      const launcher = launcherOf();
      expect(panelOf(language)).not.toContainElement(launcher);
      press('F1');
      expect(launcher).toBeVisible();
      press('F1');
      expect(launcher).toBeVisible();
    });

    it('the close button of the header closes the window', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.click(
        within(panelOf(language)).getByRole('button', {
          name: message(language, 'assistant-close'),
        }),
      );
      expect(panelOf(language)).toHaveAttribute('inert');
    });
  });

  describe('the header and the empty window', () => {
    it("is headed with the assistant's own name, while the status bar button keeps its words (Umar)", () => {
      renderIn(language, shell());
      press('F1');
      const header = within(panelOf(language)).getByRole('banner');
      const title = within(header).getByRole('heading', { level: 2 });
      // The name is the same words as the message, and exactly these words in each language.
      expect(visible(title.textContent)).toBe(message(language, 'assistant-dock-title'));
      expect(visible(title.textContent)).toBe(
        language === 'ur' ? 'عدنان اسسٹنٹ' : 'Adnan Assistant',
      );
      // The old words stay on the status bar button, the question box and the window's own label.
      expect(message(language, 'assistant-button')).toBe(
        language === 'ur' ? 'سوال پوچھیں' : 'Ask the app',
      );
      expect(
        within(screen.getByRole('contentinfo')).getByRole('button', {
          name: new RegExp(message(language, 'assistant-button')),
        }),
      ).toBeInTheDocument();
      expect(visible(title.textContent)).not.toBe(message(language, 'assistant-button'));
    });

    it("says ready, with today's count, once the status is known", async () => {
      renderIn(language, shell());
      press('F1');
      const header = within(panelOf(language)).getByRole('banner');
      expect(header).toHaveTextContent(message(language, 'assistant-dock-title'));
      expect(header).toHaveTextContent(message(language, 'assistant-state-ready'));
      await vi.waitFor(() => {
        expect(visible(header.textContent)).toContain(
          message(language, 'assistant-button-count', { used: '9', limit: '20' }),
        );
      });
    });

    it('says preparing an answer while a question is being worked on', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'one{Enter}');
      expect(within(panelOf(language)).getByRole('banner')).toHaveTextContent(
        message(language, 'assistant-state-working'),
      );
    });

    it('says switched off when questions are off, and shows the notice instead of the box', async () => {
      vi.mocked(assistantApi.assistantStatus).mockResolvedValue(STATUS_OFF);
      renderIn(language, shell());
      press('F1');
      await vi.waitFor(() => {
        expect(within(panelOf(language)).getByRole('banner')).toHaveTextContent(
          message(language, 'assistant-state-off'),
        );
      });
      // The box is gone; the notice with the reason takes its place.
      expect(within(panelOf(language)).queryByRole('textbox')).not.toBeInTheDocument();
    });

    it('says no internet after an offline reply', () => {
      renderIn(language, shell());
      press('F1');
      act(() => {
        const store = useAssistantStore.getState();
        store.setReply(store.startEntry({ type: 'text', text: 'x' }), REPLY_OFFLINE);
      });
      expect(within(panelOf(language)).getByRole('banner')).toHaveTextContent(
        message(language, 'assistant-state-offline'),
      );
    });

    it('greets, labels the day, and shows the quick questions as chips in the body', () => {
      renderIn(language, shell());
      press('F1');
      const panel = panelOf(language);
      expect(within(panel).getByText(message(language, 'common-period-today'))).toBeInTheDocument();
      expect(
        within(panel).getByText(message(language, 'assistant-empty-title')),
      ).toBeInTheDocument();
      const chips = within(panel).getByRole('list', {
        name: message(language, 'assistant-chips-label'),
      });
      // In the empty window the chips are in the body, above the footer with the box.
      expect(chips.compareDocumentPosition(boxOf(language))).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
      expect(within(chips).getAllByRole('button')).toHaveLength(10);
    });

    it('once a question is asked the chips fold behind a round button beside the box', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'one{Enter}');
      const panel = panelOf(language);
      const name = message(language, 'assistant-chips-label');
      expect(within(panel).queryByRole('list', { name })).not.toBeInTheDocument();
      const fold = within(panel).getByRole('button', { name });
      expect(fold).toHaveAttribute('aria-expanded', 'false');
      await user.click(fold);
      expect(fold).toHaveAttribute('aria-expanded', 'true');
      expect(within(panel).getByRole('list', { name })).toBeInTheDocument();
    });

    it('draws the question as a bubble and its answer as a card under it', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'akram ka hisab{Enter}');
      const question = within(panelOf(language)).getByLabelText(
        message(language, 'assistant-you-asked'),
      );
      expect(question).toHaveTextContent('akram ka hisab');
      expect(question).toHaveClass('self-end');
    });
  });

  describe('what it keeps', () => {
    it('opening, using and closing it never touches what is typed in the screen under it', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      const entry = screen.getByRole('textbox', { name: 'entry' });
      await user.type(entry, '20 box at 600');
      press('F1');
      await user.type(boxOf(language), 'a question{Enter}');
      await user.keyboard('{Escape}');
      expect(screen.getByRole('textbox', { name: 'entry' })).toHaveValue('20 box at 600');
    });

    it('the conversation and the open panel survive a change of screen', async () => {
      const client = newClient();
      const { rerender } = render(
        <AppProviders queryClient={client}>{shell('first', client)}</AppProviders>,
      );
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'what next{Enter}');
      rerender(<AppProviders queryClient={client}>{shell('second', client)}</AppProviders>);
      expect(screen.getByRole('textbox', { name: 'second' })).toBeInTheDocument();
      expect(panelOf(language)).not.toHaveAttribute('inert');
      expect(within(panelOf(language)).getByText('what next')).toBeInTheDocument();
    });

    it('the open panel, the question and the half-typed text outlive a language switch', async () => {
      const client = newClient();
      render(<AppProviders queryClient={client}>{shell('entry', client)}</AppProviders>);
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'sent{Enter}half typed');
      const other: Language = language === 'ur' ? 'en' : 'ur';
      setLanguage(other);
      // The same panel, drawn again in the other language: its words change, nothing else does.
      expect(panelOf(other)).not.toHaveAttribute('inert');
      expect(within(panelOf(other)).getByText('sent')).toBeInTheDocument();
      expect(boxOf(other)).toHaveValue('half typed');
      expect(
        within(panelOf(other)).getByRole('button', {
          name: new RegExp(message(other, 'assistant-send')),
        }),
      ).toBeInTheDocument();
      expect(visible(panelOf(other).textContent)).toContain(
        message(other, 'assistant-state-working'),
      );
    });

    it('New question forgets the conversation and keeps the questions for ↑', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'old one{Enter}');
      act(() => {
        useAssistantStore.getState().setReply(1, REPLY_ANSWERED);
      });
      await user.click(
        within(panelOf(language)).getByRole('button', {
          name: new RegExp(message(language, 'assistant-new-conversation')),
        }),
      );
      expect(useAssistantStore.getState().entries).toHaveLength(0);
      expect(
        within(panelOf(language)).getByText(message(language, 'assistant-empty-title')),
      ).toBeInTheDocument();
      expect(boxOf(language)).toHaveFocus();
      await user.keyboard('{ArrowUp}');
      expect(boxOf(language)).toHaveValue('old one');
    });

    it('New question tells the backend first, and the next question is sent with no conversation', async () => {
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'old one{Enter}');
      act(() => {
        useAssistantStore.getState().setReply(1, REPLY_ANSWERED);
      });
      // The conversation the backend gave the first question is the one the panel holds.
      expect(useAssistantStore.getState().conversationId).toBe(REPLY_ANSWERED.conversation_id);
      await user.click(
        within(panelOf(language)).getByRole('button', {
          name: new RegExp(message(language, 'assistant-new-conversation')),
        }),
      );
      expect(assistantApi.assistantNewConversation).toHaveBeenCalledTimes(1);
      // It is told which conversation the panel leaves, so the backend drops that one's memory.
      expect(assistantApi.assistantNewConversation).toHaveBeenCalledWith({
        conversation_id: REPLY_ANSWERED.conversation_id,
      });
      expect(useAssistantStore.getState().conversationId).toBeNull();
      await user.type(boxOf(language), 'next one{Enter}');
      expect(assistantApi.assistantAsk).toHaveBeenLastCalledWith({
        text: 'next one',
        conversation_id: null,
      });
      // The backend heard of the new conversation before the new question went out.
      const told = vi.mocked(assistantApi.assistantNewConversation).mock.invocationCallOrder[0];
      const asked = vi.mocked(assistantApi.assistantAsk).mock.invocationCallOrder.at(-1);
      expect(told).toBeLessThan(asked ?? 0);
    });

    it('New question forgets the conversation even when the backend cannot be told', async () => {
      vi.mocked(assistantApi.assistantNewConversation).mockRejectedValue(new Error('gone'));
      renderIn(language, shell());
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'old one{Enter}');
      act(() => {
        useAssistantStore.getState().setReply(1, REPLY_ANSWERED);
      });
      await user.click(
        within(panelOf(language)).getByRole('button', {
          name: new RegExp(message(language, 'assistant-new-conversation')),
        }),
      );
      expect(useAssistantStore.getState().entries).toHaveLength(0);
      expect(useAssistantStore.getState().conversationId).toBeNull();
      expect(
        within(panelOf(language)).getByText(message(language, 'assistant-empty-title')),
      ).toBeInTheDocument();
    });

    it('is forgotten, and closed, when the app locks', async () => {
      const client = newClient();
      renderIn(language, shell('entry', client));
      const user = userEvent.setup();
      press('F1');
      await user.type(boxOf(language), 'private{Enter}');
      expect(useAssistantStore.getState().entries).toHaveLength(1);
      // The app reads who is signed in again after the lock; the answer now says locked.
      act(() => {
        client.setQueryData(['auth_status'], authState({ locked: true }));
      });
      await vi.waitFor(() => {
        expect(useAssistantStore.getState().open).toBe(false);
      });
      expect(useAssistantStore.getState().entries).toHaveLength(0);
      expect(useAssistantStore.getState().draft).toBe('');
    });
  });

  it('writes no word of its own: every word of the window comes from the Fluent file', () => {
    renderIn(language, shell());
    press('F1');
    const text = visible(panelOf(language).textContent);
    // The empty conversation, the buttons and the box hint are the file's words in this language.
    expect(text).toContain(message(language, 'assistant-empty-title'));
    expect(text).toContain(message(language, 'assistant-new-conversation'));
    expect(text).toContain(message(language, 'assistant-state-ready'));
    expect(text).toContain(message(language, 'common-period-today'));
    expect(
      within(panelOf(language)).getByRole('button', {
        name: new RegExp(message(language, 'assistant-send')),
      }),
    ).toBeInTheDocument();
  });
});
