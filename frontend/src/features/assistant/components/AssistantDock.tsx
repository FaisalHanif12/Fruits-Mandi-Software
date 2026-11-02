// The question panel (plan 49, PRD FR-7.73): it rises from the bottom of every screen, above the
// status bar, and stays while the Owner moves between screens. It is mounted once by the app shell and
// never unmounts on a route change.
//
// - `F1` or the status bar button opens it; `F1` inside it, `Esc` or the close button closes it, and
//   the focus goes back to where it came from (FR-7.18).
// - Two heights, a third and two thirds of the window: `Ctrl+↑` and `Ctrl+↓` inside the panel, or the
//   button (FR-7.73). It slides in at most 150 ms (NFR-USE-24).
// - A focus trap while focus is inside: Tab goes round the panel's own controls.
// - It draws over the lower part of the screen and never changes it: nothing typed in the screen
//   under it is touched by opening, using or closing the panel.
// - Everything it shows is kept in the store above (store.ts), so a language switch loses nothing.
// - The quick questions work with the assistant off and with no internet (FR-7.80, FR-7.81).

import { Localized, useLocalization } from '@fluent/react';
import {
  ChevronDown,
  ChevronsDown,
  ChevronsUp,
  ChevronUp,
  MessageCircleQuestion,
  RotateCcw,
  X,
} from 'lucide-react';
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
import { Conversation } from './Conversation';
import { QuestionBox } from './QuestionBox';
import { QuickChips } from './QuickChips';

/** The Fluent message of the control that changes the height. */
function heightMessageId(tall: boolean): string {
  return tall ? 'assistant-shorter' : 'assistant-taller';
}

/** How long the body of a shut panel stays while it slides away: the slide is 150 ms (--motion-fast). */
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
  /** Where the focus was when the panel opened: it goes back there when the panel closes. */
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
  // focus again, as long as the Owner is still in the panel.
  const lastOutcome = entries.at(-1)?.reply?.outcome ?? null;
  const lastId = entries.at(-1)?.id ?? null;
  useEffect(() => {
    if (lastOutcome !== null && TYPE_NEXT.has(lastOutcome) && focusInside()) focusBox();
  }, [lastOutcome, lastId]);

  // `F1` opens the panel; with it open it closes it, or, when the focus is on the screen, brings the
  // focus back into the box (PRD 7.3: open or close).
  useShortcut(SHORTCUTS.askApp, () => {
    if (!open) openDock();
    else if (focusInside()) closeDock();
    else focusBox();
  });
  // Esc closes the panel while the focus is in it. A list open inside it (the party box) closes
  // first, because it registered later (FR-7.18).
  useEscape('overlay', closeDock, () => open && focusInside());

  // The body stays on the page for the length of the slide out, then goes (see SLIDE_OUT_MS).
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
    // The two heights (FR-7.73). These keys are the panel's own, so they answer only from inside it.
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
  // In both heights the quick questions fold behind a button once a conversation has begun, so the
  // answer and its buttons have the room (NFR-USE-15 at 1024 x 600). In an empty conversation they
  // always show. After a limit or a lost connection they are what still works, so they are open.
  const chipsNeeded = lastOutcome === 'limit_reached' || lastOutcome === 'offline';
  const foldable = entries.length > 0 && !chipsNeeded;
  const showChips = !foldable || chipsOpen;
  const HeightIcon = tall ? ChevronsDown : ChevronsUp;
  const heightLabel = l10n.getString(heightMessageId(tall));
  return (
    // The section is a landmark, so a screen reader can jump to it; closed, it is hidden and inert.
    <section
      ref={section}
      aria-label={l10n.getString('assistant-dock-label')}
      inert={!open}
      onKeyDown={onKeyDown}
      className={`absolute inset-x-0 bottom-0 flex flex-col border-t border-border-control bg-surface text-ink shadow-overlay transition-slide ${tall ? 'h-dock-tall' : 'h-dock-short'} ${open ? 'visible translate-y-0' : 'invisible translate-y-full'}`}
    >
      {/* Closed, the panel holds nothing: the controls of a shut panel are not on the page for a
          screen reader or a test to find. They stay while it slides away. */}
      {showBody ? (
        <>
          <div className="flex shrink-0 items-center gap-3 border-b border-border px-4 py-1">
            <MessageCircleQuestion
              aria-hidden="true"
              className="size-icon-md shrink-0 text-primary"
            />
            <h2 className="type-section min-w-0 flex-1">
              <Localized id="assistant-dock-title" />
            </h2>
            <Button
              onClick={() => {
                newConversation();
                focusBox();
              }}
              disabled={entries.length === 0}
            >
              <RotateCcw aria-hidden="true" className="size-icon-sm" />
              <Localized id="assistant-new-conversation" />
            </Button>
            <Button
              iconOnly
              accessibleName={`${heightLabel} (${(tall ? SHORTCUTS.dockShorter : SHORTCUTS.dockTaller).label})`}
              onClick={() => {
                setTall(!tall);
              }}
            >
              <HeightIcon aria-hidden="true" className="size-icon-md" />
            </Button>
            <Button iconOnly accessibleName={l10n.getString('assistant-close')} onClick={closeDock}>
              <X aria-hidden="true" className="size-icon-md" />
            </Button>
          </div>
          <Conversation entries={entries} onAsk={runner.ask} onPick={runner.pick} busy={working} />
          <div className="flex shrink-0 flex-col gap-2 border-t border-border bg-surface-sunken px-4 py-2">
            {showChips ? <QuickChips onQuick={runner.quick} busy={working} /> : null}
            <div className="flex items-end gap-3">
              {foldable ? (
                // Once a conversation has begun the quick questions fold behind this button, on the row
                // of the box, so the answer keeps the room (NFR-USE-15 at 1024 x 600).
                <button
                  type="button"
                  aria-expanded={chipsOpen}
                  onClick={() => {
                    setChipsOpen(!chipsOpen);
                  }}
                  className="inline-flex h-control shrink-0 items-center gap-2 rounded-2 border border-border-control bg-surface px-4 type-control text-ink transition-calm"
                >
                  {chipsOpen ? (
                    <ChevronDown aria-hidden="true" className="size-icon-sm" />
                  ) : (
                    <ChevronUp aria-hidden="true" className="size-icon-sm" />
                  )}
                  <Localized id="assistant-chips-label" />
                </button>
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
  );
}
