// The chat window of Ask the app (plans 49 and 51, PRD FR-7.73): a round launcher floats at the end of
// every screen above the status bar, and the window opens above it as a card in the app's own colours.
// It is mounted once by the app shell and never unmounts on a route change.
//
// - `F1`, the status bar button or the launcher opens it; `F1` inside it, `Esc`, the close button or
//   the launcher closes it, and the focus goes back to where it came from (FR-7.18).
// - Two sizes, normal (half the window wide, most of its height) and large (nearly the whole window):
//   `Ctrl+↑` and `Ctrl+↓` inside the window, or the size button (FR-7.73). It opens in at most 150 ms
//   (NFR-USE-24).
// - A focus trap while focus is inside: Tab goes round the window's own controls.
// - It floats over the screen and never changes it: nothing typed in the screen under it is touched by
//   opening, using or closing the window.
// - Everything it shows is kept in the store (store.ts), so a language switch loses nothing.
// - The quick questions work with the assistant off and with no internet (FR-7.80, FR-7.81).

import { Localized, useLocalization } from '@fluent/react';
import { Zap } from 'lucide-react';
import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import {
  focusRegion,
  matchesShortcut,
  SHORTCUTS,
  trapTab,
  useEscape,
  useShortcut,
} from '../../../shared/keyboard';
import { Button, Notice } from '../../../shared/ui';
import { useAuthStatus } from '../../auth/hooks';
import {
  useAssistantRunner,
  useAssistantStatus,
  useStartNewConversation,
  useStepLines,
} from '../hooks';
import { useAssistantNavigation } from '../navigation';
import { useAssistantStore } from '../store';
import { AssistantLauncher, CHAT_WINDOW_ID } from './AssistantLauncher';
import { ChatHeader, chatStateOf } from './ChatHeader';
import { Conversation } from './Conversation';
import { QuestionBox } from './QuestionBox';
import { QuickChips } from './QuickChips';
import { RoundChip } from './RoundChip';

/** How long the body of a shut window stays while it fades away: the fade is 150 ms (--motion-fast). */
const SLIDE_OUT_MS = 200;

/** The outcomes after which the next thing the Owner does is to type: the box takes the focus. */
const TYPE_NEXT = new Set(['need_dates', 'need_party', 'not_understood']);

export function AssistantDock() {
  const { l10n } = useLocalization();
  const open = useAssistantStore((state) => state.open);
  const tall = useAssistantStore((state) => state.tall);
  const draft = useAssistantStore((state) => state.draft);
  const entries = useAssistantStore((state) => state.entries);
  const { openDock, closeDock, setTall, setDraft, recallQuestion } = useAssistantStore.getState();
  const newConversation = useStartNewConversation();
  const section = useRef<HTMLElement>(null);
  /** Where the focus was when the window opened: it goes back there when the window closes. */
  const origin = useRef<HTMLElement | null>(null);
  const locked = useAuthStatus().data?.locked === true;
  const status = useAssistantStatus();
  const runner = useAssistantRunner();
  const navigation = useAssistantNavigation();
  const [chipsOpen, setChipsOpen] = useState(false);
  useStepLines();

  const focusInside = () => section.current?.contains(document.activeElement) === true;
  const focusBox = () => {
    section.current
      ?.querySelector<HTMLInputElement>('[data-slot="assistant-question"] input')
      ?.focus();
  };

  // The conversation is forgotten when the app locks or the Owner signs out (FR-7.73). Signing out
  // takes the whole shell away, and the cleanup below clears the store with it.
  useEffect(() => {
    if (locked) useAssistantStore.getState().reset();
  }, [locked]);
  useEffect(
    () => () => {
      useAssistantStore.getState().reset();
    },
    [],
  );

  // Open: remember where the focus was, then put it in the question box. Close: give it back.
  const wasOpen = useRef(false);
  useEffect(() => {
    if (open && !wasOpen.current) {
      origin.current =
        document.activeElement instanceof HTMLElement ? document.activeElement : null;
      focusBox();
    } else if (!open && wasOpen.current) {
      const back = origin.current;
      origin.current = null;
      if (back !== null && back.isConnected && back !== document.body) back.focus();
      else focusRegion('screen');
    }
    wasOpen.current = open;
  }, [open]);

  // After a reply that asks the Owner to type (the dates, the party, other words), the box has the
  // focus again, as long as the Owner is still in the window.
  const lastOutcome = entries.at(-1)?.reply?.outcome ?? null;
  const lastId = entries.at(-1)?.id ?? null;
  useEffect(() => {
    if (lastOutcome !== null && TYPE_NEXT.has(lastOutcome) && focusInside()) focusBox();
  }, [lastOutcome, lastId]);

  // `F1` opens the window; with it open it closes it, or, when the focus is on the screen, brings the
  // focus back into the box (PRD 7.3: open or close).
  useShortcut(SHORTCUTS.askApp, () => {
    if (!open) openDock();
    else if (focusInside()) closeDock();
    else focusBox();
  });
  // Esc closes the window while the focus is in it. A list open inside it (the party box) closes
  // first, because it registered later (FR-7.18).
  useEscape('overlay', closeDock, () => open && focusInside());

  // The body stays on the page for the length of the fade out, then goes (see SLIDE_OUT_MS).
  const [kept, setKept] = useState(open);
  if (open && !kept) setKept(true);
  useEffect(() => {
    if (open) return undefined;
    const timer = window.setTimeout(() => {
      setKept(false);
    }, SLIDE_OUT_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [open]);
  const showBody = open || kept;
  const working = entries.some((entry) => entry.working);
  const send = () => {
    const text = draft.trim();
    if (text === '' || working) return;
    runner.ask(text);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    const container = section.current;
    if (container === null) return;
    if (trapTab(event, container)) return;
    // The two sizes (FR-7.73). These keys are the window's own, so they answer only from inside it.
    if (matchesShortcut(event.nativeEvent, SHORTCUTS.dockTaller)) {
      event.preventDefault();
      setTall(true);
    } else if (matchesShortcut(event.nativeEvent, SHORTCUTS.dockShorter)) {
      event.preventDefault();
      setTall(false);
    }
  };

  // The box shows unless the status says questions are off or have no key. While the status is not
  // known (or cannot be read) it shows, and the backend refuses with a message if it must.
  const known = status.data;
  const keySaved = known?.key_saved ?? false;
  const asking = known === undefined || (known.enabled && known.key_saved);
  // In an empty conversation the quick questions are in the body, under the greeting. Once a
  // conversation has begun they fold behind a chip in the box's row, in both sizes, so the answer has
  // the room (NFR-USE-15 at 1024 x 600). After a limit or a lost connection they are what still works,
  // so they are open.
  const started = entries.length > 0;
  const chipsNeeded = lastOutcome === 'limit_reached' || lastOutcome === 'offline';
  const foldable = started && !chipsNeeded;
  const showFooterChips = started && (chipsNeeded || chipsOpen);
  const state = chatStateOf({ working, asking, lastOutcome });
  return (
    <>
      <AssistantLauncher />
      {/* The section is a landmark, so a screen reader can jump to it; closed, it is hidden and inert. */}
      <section
        ref={section}
        id={CHAT_WINDOW_ID}
        aria-label={l10n.getString('assistant-dock-label')}
        inert={!open}
        data-open={open}
        data-size={tall ? 'large' : 'normal'}
        onKeyDown={onKeyDown}
        className="chat-window flex flex-col overflow-hidden rounded-5 border border-border bg-surface text-ink shadow-overlay"
      >
        {/* Closed, the window holds nothing: the controls of a shut window are not on the page for a
            screen reader or a test to find. They stay while it fades away. */}
        {showBody ? (
          <>
            <ChatHeader
              state={state}
              onNew={() => {
                newConversation();
                focusBox();
              }}
              onClose={closeDock}
            />
            <Conversation
              entries={entries}
              onAsk={runner.ask}
              onPick={runner.pick}
              busy={working}
              chips={<QuickChips onQuick={runner.quick} busy={working} />}
            />
            <div className="flex shrink-0 flex-col gap-3 border-t border-border bg-surface px-4 py-3">
              {showFooterChips ? (
                <QuickChips onQuick={runner.quick} busy={working} pickerAbove />
              ) : null}
              <div className="flex items-center gap-3">
                {foldable ? (
                  // Once a conversation has begun the quick questions fold behind this round button, on the row
                  // of the box, so the answer keeps the room (NFR-USE-15 at 1024 x 600).
                  <RoundChip
                    accessibleName={l10n.getString('assistant-chips-label')}
                    expanded={chipsOpen}
                    onClick={() => {
                      setChipsOpen(!chipsOpen);
                    }}
                  >
                    <Zap aria-hidden="true" className="size-icon-md" />
                  </RoundChip>
                ) : null}
                <div className="min-w-0 flex-1">
                  {asking ? (
                    <QuestionBox
                      value={draft}
                      onChange={setDraft}
                      onSend={send}
                      onRecall={recallQuestion}
                      busy={working}
                    />
                  ) : (
                    <Notice
                      tone="info"
                      action={
                        navigation === null ? undefined : (
                          <Button onClick={navigation.openSettings}>
                            <Localized id="assistant-open-settings" />
                          </Button>
                        )
                      }
                    >
                      <Localized id={keySaved ? 'assistant-off' : 'assistant-no-key'}>
                        <span />
                      </Localized>
                    </Notice>
                  )}
                </div>
              </div>
            </div>
          </>
        ) : null}
      </section>
    </>
  );
}
