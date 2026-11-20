// The box the Owner types his question in (PRD FR-7.73), a rounded box with a round send button (plan 51). It takes the letters as typed, in Urdu, Roman
// Urdu or English, with no Roman-to-Urdu conversion, like global search (FR-7.22): the model reads any
// of them. `Enter` sends; `↑` brings back the last question, `↑` again an older one, `↓` a newer one.
// It is the shared text input; the keys of the box are the only keys written here.

import { Localized, useLocalization } from '@fluent/react';
import { SendHorizontal } from 'lucide-react';
import { useRef, type KeyboardEvent } from 'react';
import { matchesShortcut, SHORTCUTS } from '../../../shared/keyboard';
import { Button, Field, TextInput } from '../../../shared/ui';

interface QuestionBoxProps {
  value: string;
  onChange: (text: string) => void;
  /** Send what is typed. The panel ignores it while a question is still being worked on. */
  onSend: () => void;
  /** `↑` asks for an older question, `↓` for a newer one. */
  onRecall: (direction: 'older' | 'newer') => void;
  /**
   * The first key pressed in the box since it was drawn (the window draws it again each time it opens):
   * the panel warms up the model connection then (plan 82). It never changes what the key does.
   */
  onFirstKey?: () => void;
  /** A question is being worked on: the button waits. */
  busy: boolean;
}

/** Keys that count as the Owner working in the box besides a character: editing and recalling. */
const EDITING = new Set(['Backspace', 'Delete', 'Enter', 'ArrowUp', 'ArrowDown']);

/** True for a key that goes into the box or edits it, not a key that moves the focus (Tab), holds a modifier or is a function key (F1). */
function isTyping(key: string): boolean {
  return key.length === 1 || EDITING.has(key);
}

export function QuestionBox({
  value,
  onChange,
  onSend,
  onRecall,
  onFirstKey,
  busy,
}: QuestionBoxProps) {
  const { l10n } = useLocalization();
  const keyed = useRef(false);
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    // A word still being composed (an input method) is not a finished question.
    if (event.nativeEvent.isComposing) return;
    // The first key that goes into the box or edits it, not F1 (which shuts the window) or a key that only moves the focus.
    if (!keyed.current && isTyping(event.key)) {
      keyed.current = true;
      onFirstKey?.();
    }
    if (matchesShortcut(event.nativeEvent, SHORTCUTS.confirm)) {
      event.preventDefault();
      onSend();
      return;
    }
    // The box is one line, so `↑` and `↓` have no other use in it (PRD FR-7.73).
    if (event.key === 'ArrowUp' && !event.ctrlKey && !event.altKey && !event.shiftKey) {
      event.preventDefault();
      onRecall('older');
    } else if (event.key === 'ArrowDown' && !event.ctrlKey && !event.altKey && !event.shiftKey) {
      event.preventDefault();
      onRecall('newer');
    }
  };
  return (
    <div className="flex items-center gap-3" data-slot="assistant-question">
      {/* The box has no key handler of the screen's own: Enter and the arrows are answered here, once. */}
      <div className="min-w-0 flex-1" onKeyDown={onKeyDown}>
        <Field label={<Localized id="assistant-box-label" />} hideLabel>
          {(control) => (
            <TextInput
              {...control}
              pill
              value={value}
              onChange={onChange}
              placeholder={l10n.getString('assistant-box-placeholder', {
                key: SHORTCUTS.confirm.label,
              })}
            />
          )}
        </Field>
      </div>
      {/* The key is in the name, not on the button: a round button has no room for its chip (FR-7.4). */}
      <Button
        round
        iconOnly
        variant="primary"
        onClick={onSend}
        busy={busy}
        accessibleName={`${l10n.getString('assistant-send')} (${SHORTCUTS.confirm.label})`}
      >
        <SendHorizontal aria-hidden="true" className="size-icon-md rtl:-scale-x-100" />
      </Button>
    </div>
  );
}
