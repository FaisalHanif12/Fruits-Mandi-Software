// How was this worked out? (PRD FR-7.43, Tech Stack 4.7, Code Standards F42, B30.) The small ⓘ
// next to every calculated amount. It shows the working the backend sent with the amount, with the
// real numbers: "12,000 × 6 ÷ 100 = 720". The screen never builds a working itself: the backend
// names the message (`message_id`) and gives its values already as text (`args`, grouped as the
// app shows money); the words come from Fluent, in the app language. A working this build has no
// words for shows the plain "no working" line, never the raw ID. Ctrl+I on the ⓘ opens it too.

import { Localized, useLocalization } from '@fluent/react';
import { Info } from 'lucide-react';
import { Popover } from 'radix-ui';
import { useUiLanguage } from '../i18n';
import { SHORTCUTS, matchesShortcut } from '../keyboard';

/**
 * The working of one calculated amount, as the backend sends it (interim shape until the
 * generated binding arrives): the Fluent message that tells it and its values as text.
 */
export interface Working {
  message_id: string;
  args: Record<string, string>;
}

interface CalcInfoProps {
  working: Working | null;
}

export function CalcInfo({ working }: CalcInfoProps) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  if (working === null) return null;
  const known = l10n.getBundle(working.message_id) !== null;
  const label = l10n.getString('calc-info-label');
  return (
    <Popover.Root>
      <Popover.Trigger
        aria-label={label}
        title={label}
        onKeyDown={(event) => {
          // Ctrl+I opens it from the button as well (PRD 7.3).
          if (matchesShortcut(event.nativeEvent, SHORTCUTS.explain)) {
            event.preventDefault();
            event.currentTarget.click();
          }
        }}
        className="inline-flex size-control-compact shrink-0 items-center justify-center rounded-1 text-ink-muted transition-calm"
      >
        <Info aria-hidden="true" className="size-icon-sm" />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          lang={language}
          dir={direction}
          sideOffset={4}
          className="z-30 flex max-w-reading flex-col gap-1 rounded-2 border border-border bg-surface p-3 text-ink shadow-overlay"
        >
          <Localized id="calc-info-title">
            <p className="type-label" />
          </Localized>
          {known ? (
            <Localized id={working.message_id} vars={working.args}>
              <p className="type-body" />
            </Localized>
          ) : (
            <Localized id="calc-info-unknown">
              <p className="type-body text-ink-muted" />
            </Localized>
          )}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
