// The error of a field that stands in a compact row (the date of the sale book, plan 93 T8, Umar 22-11-2026). The words
// are long ("A future date is not allowed. Enter today (22-11-2026) or an earlier date."); drawn under the field they widen
// the one-line row of the screen, push its tabs out and break it. So they are drawn as a bubble that floats under the field,
// in a portal, kept inside the window in both directions: the row never changes size.
//
// - The field keeps its red border and `aria-invalid` for as long as the value is wrong (the `Field` does that), and the
//   words stay for a screen reader: a visually hidden line with the id the control points to (`aria-describedby`).
// - The bubble shows when the error appears or its words change on a new answer, and for `ERROR_BUBBLE_MS` (5 s) each time. It shows again
//   each time the user repeats the action that gives the error: a change or a blur in the control, or the owner's `pulse`
//   (a try to save). A click outside or Esc hides it. When the value becomes right the error is gone and so is the bubble.
// - Only the bubble is drawn in the window; it takes no focus and no pointer, so typing goes on.

import { CircleAlert } from 'lucide-react';
import { Popover } from 'radix-ui';
import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react';
import { useUiLanguage } from '../i18n';
import { radixCollisionPadding } from './dropdown-position';
import { ERROR_BUBBLE_MS } from './error-bubble';

interface FieldErrorBubbleProps {
  /** The id the control's `aria-describedby` points to. */
  id: string;
  /** What is wrong and what to do, in the app language. */
  error: ReactNode;
  /** A number that goes up each time the owner repeats the action (a try to save): the bubble shows again. */
  pulse?: number | undefined;
  /** The control. */
  children: ReactNode;
}

export function FieldErrorBubble({ id, error, pulse, children }: FieldErrorBubbleProps) {
  const { language, direction } = useUiLanguage();
  const [shown, setShown] = useState(false);
  // Goes up each time the bubble is asked to show: the 5 seconds start again.
  const [round, setRound] = useState(0);
  const words = useRef<HTMLParagraphElement>(null);
  const lastWords = useRef('');
  const lastPulse = useRef(pulse);

  const hide = useCallback(() => {
    setShown(false);
  }, []);
  /** Shows the bubble for another `ERROR_BUBBLE_MS`. */
  const reveal = useCallback(() => {
    setShown(true);
    setRound((count) => count + 1);
  }, []);
  // The timer belongs to this effect: it starts when the bubble shows, starts again when it is asked to show again, and
  // goes with the bubble. An effect that runs twice (React's strict mode in development) just starts it again, so the
  // bubble can never be left on screen with no timer behind it. Nothing but `reveal` restarts it: the same error sent
  // again by a preview in the background is not a user action and does not call it.
  useEffect(() => {
    if (!shown) return undefined;
    const timer = window.setTimeout(() => {
      setShown(false);
    }, ERROR_BUBBLE_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [shown, round]);
  // The error appeared or its words changed: show.
  useEffect(() => {
    const text = words.current?.textContent ?? '';
    if (text === lastWords.current) return;
    lastWords.current = text;
    reveal();
  });
  // The owner repeated the action (a try to save): show again.
  useEffect(() => {
    if (pulse === lastPulse.current) return;
    lastPulse.current = pulse;
    reveal();
  }, [pulse, reveal]);

  return (
    <Popover.Root
      open={shown}
      onOpenChange={(open) => {
        if (!open) hide();
      }}
    >
      <Popover.Anchor asChild>
        <div
          className="flex items-center"
          // The user repeats the action that gives the error: a change or a blur in the control.
          onInputCapture={reveal}
          onBlurCapture={reveal}
        >
          {children}
        </div>
      </Popover.Anchor>
      <p id={id} ref={words} className="sr-only">
        {error}
      </p>
      <Popover.Portal>
        <Popover.Content
          lang={language}
          dir={direction}
          side="bottom"
          align="start"
          sideOffset={4}
          collisionPadding={radixCollisionPadding()}
          // A message, not a dialog: it takes no focus (typing goes on) and the words are read from the line above.
          aria-hidden="true"
          data-error-bubble=""
          onOpenAutoFocus={(event) => {
            event.preventDefault();
          }}
          onCloseAutoFocus={(event) => {
            event.preventDefault();
          }}
          onFocusOutside={(event) => {
            event.preventDefault();
          }}
          className="z-50 flex w-dialog max-w-(--radix-popover-content-available-width) items-start gap-2 rounded-2 border border-due bg-surface p-3 text-due shadow-overlay type-body"
        >
          <CircleAlert aria-hidden="true" className="mt-1 size-icon-sm shrink-0" />
          <span className="min-w-0">{error}</span>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
