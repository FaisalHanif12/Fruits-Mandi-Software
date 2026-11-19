// A card that opens beside any control when it is pointed at or reached with the keyboard (plan 75 T6,
// PRD FR-19.14). The control is the one the screen already has, a link or a button: it keeps its look,
// its click and its Enter key, so the card only adds to it. It is built on the Radix hover card.
//
// - Mouse: opens after a short wait (HOVER_OPEN_DELAY_MS), so passing over a row of controls does not
//   flash a card for each. It stays open while the pointer is on the control or on the card, and closes
//   a moment after the pointer leaves.
// - Keyboard: opens at once when the control gets the focus after a key was pressed (Tab, or the arrow
//   keys of a list; NFR-USE-7), and closes when the focus leaves. A focus that follows a click, or that a
//   screen sets on its first item as it opens after a click, opens nothing.
// - Touch: a long press opens it. The tap that ends the press does not follow the link.
// - `Esc` closes it and leaves the focus where it is. Esc does nothing else while the card is open
//   (FR-7.18: an open list or card is closed first).
// - It opens on the side where the text ends (right in English, left in Urdu) and flips to the other
//   side near the edge of the window. It is as wide as the room on its side allows and as tall as the
//   window allows (it scrolls inside), so it never covers the control it belongs to and never makes the
//   page scroll sideways.
// - Only one card is open at a time, so moving fast from one control to the next shows one card.
//
// The card's words are the screen's own, given as `card` and `label`.

import { HoverCard } from 'radix-ui';
import {
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
  type FocusEvent,
  type MouseEvent,
  type PointerEvent,
  type ReactElement,
  type ReactNode,
} from 'react';
import { useUiLanguage, type Direction } from '../i18n';

/** The wait before a card opens under the pointer (plan 75 Section 7.3). */
export const HOVER_OPEN_DELAY_MS = 300;

/** The wait before a card closes after the pointer leaves, long enough to cross the gap to the card. */
export const HOVER_CLOSE_DELAY_MS = 200;

/** How long a finger must stay down to open the card on a touch screen. */
export const LONG_PRESS_MS = 500;

/** Room kept between the card and the edge of the window; it is the spacing step `--space-4`. */
const EDGE_PADDING_PX = 16;

/** The gap between the control and its card; it is the spacing step `--space-2`. */
const GAP_PX = 8;

/** The card that is open now, so that opening another closes it at once. */
let closeOpenCard: (() => void) | null = null;

/**
 * What the user touched last: a key or the pointer. A card opens at once only for a focus that a key made.
 * The browser's own `:focus-visible` is not used, because it says "no" for a focus a script moves in some
 * engines (the arrow keys of a list move the focus by script), and a card must open then. One pair of
 * listeners is shared by every card, installed when the first one is drawn.
 */
let lastInputWasKey = false;
let watchingInput = false;

function watchInput(): void {
  if (watchingInput) return;
  watchingInput = true;
  document.addEventListener(
    'keydown',
    () => {
      lastInputWasKey = true;
    },
    true,
  );
  document.addEventListener(
    'pointerdown',
    () => {
      lastInputWasKey = false;
    },
    true,
  );
}

/**
 * Moves the focus to a control without opening its card. A screen that puts the focus on its first item as
 * it opens, or that jumps to a group, uses this: the user did not ask to see that card, and a key he pressed
 * just before (Enter on the menu) must not open it. The next arrow key or Tab opens cards again.
 */
export function focusQuietly(element: HTMLElement): void {
  lastInputWasKey = false;
  element.focus();
}

/**
 * The physical side of a control where the text of the screen ends: Radix places a card by physical side,
 * and this is right in English and left in Urdu. The card flips to the other side near the window's edge.
 */
export function endSideOf(direction: Direction): 'left' | 'right' {
  return direction === 'rtl' ? 'left' : 'right';
}

interface HoverDetailsProps {
  /** The one control that opens the card: a link or a button. It keeps its own look and click. */
  children: ReactElement;
  /** The name of the card for a screen reader, for example "About Credit Details". */
  label: string;
  /**
   * What the card holds. It is drawn only while the card is open. The card has no padding of its own: what
   * it holds brings the padding (16 px, `p-4`), so a footer that sticks to the bottom as the card scrolls can
   * reach its edge.
   */
  card: ReactNode;
}

export function HoverDetails({ children, label, card }: HoverDetailsProps) {
  const { language, direction } = useUiLanguage();
  const [open, setOpen] = useState(false);
  // The screen the card must stay inside (the `main` of the app shell): it does not cover the menu, the top
  // bar or the status bar. Found when the card opens.
  const [boundary, setBoundary] = useState<HTMLElement | null>(null);
  const triggerRef = useRef<HTMLAnchorElement>(null);
  const cardId = useId();
  const pressTimer = useRef(0);
  const pressedLong = useRef(false);

  const close = useCallback(() => {
    setOpen(false);
  }, []);

  const show = () => {
    setBoundary(triggerRef.current?.closest('main') ?? null);
    setOpen(true);
  };

  // One card at a time: when this one opens, the one that was open closes.
  useEffect(() => {
    if (!open) return undefined;
    if (closeOpenCard !== null && closeOpenCard !== close) closeOpenCard();
    closeOpenCard = close;
    return () => {
      if (closeOpenCard === close) closeOpenCard = null;
    };
  }, [open, close]);

  useEffect(() => {
    watchInput();
    return () => {
      window.clearTimeout(pressTimer.current);
    };
  }, []);

  const onFocus = (event: FocusEvent<HTMLElement>) => {
    // The hover card opens on any focus after its delay. This one opens at once, and only for a key,
    // so it stops the card's own handler (it skips a handler when the event is marked as handled).
    event.preventDefault();
    if (lastInputWasKey) show();
  };

  const onPointerDown = (event: PointerEvent<HTMLElement>) => {
    if (event.pointerType !== 'touch') return;
    pressedLong.current = false;
    window.clearTimeout(pressTimer.current);
    pressTimer.current = window.setTimeout(() => {
      pressedLong.current = true;
      show();
    }, LONG_PRESS_MS);
  };

  const endPress = () => {
    window.clearTimeout(pressTimer.current);
  };

  // The tap that ends a long press would follow the link; the card is what the user asked for.
  const onClickCapture = (event: MouseEvent<HTMLElement>) => {
    if (!pressedLong.current) return;
    pressedLong.current = false;
    event.preventDefault();
    event.stopPropagation();
  };

  return (
    <HoverCard.Root
      open={open}
      onOpenChange={(next) => {
        if (next) show();
        else close();
      }}
      openDelay={HOVER_OPEN_DELAY_MS}
      closeDelay={HOVER_CLOSE_DELAY_MS}
    >
      <HoverCard.Trigger
        ref={triggerRef}
        asChild
        onFocus={onFocus}
        onPointerDown={onPointerDown}
        onPointerUp={endPress}
        onPointerCancel={endPress}
        onPointerLeave={endPress}
        onClickCapture={onClickCapture}
        aria-describedby={open ? cardId : undefined}
      >
        {children}
      </HoverCard.Trigger>
      <HoverCard.Portal>
        <HoverCard.Content
          id={cardId}
          role="group"
          aria-label={label}
          lang={language}
          dir={direction}
          side={endSideOf(direction)}
          align="start"
          sideOffset={GAP_PX}
          {...(boundary === null ? {} : { collisionBoundary: boundary })}
          collisionPadding={EDGE_PADDING_PX}
          className="z-50 max-h-(--radix-hover-card-content-available-height) w-dialog max-w-(--radix-hover-card-content-available-width) overflow-y-auto overflow-x-hidden rounded-3 border border-border bg-surface break-words text-ink shadow-overlay"
        >
          {card}
        </HoverCard.Content>
      </HoverCard.Portal>
    </HoverCard.Root>
  );
}
