// How was this worked out? (PRD FR-7.43, Tech Stack 4.7, Code Standards F42, B30.) The small ⓘ
// next to every calculated amount. It shows the working the backend sent with the amount, with the
// real numbers: "12,000 × 6 ÷ 100 = 720". The screen never builds a working itself: the backend
// names the message (`message_id`) and gives each value with its kind (money, percent, count,
// level); this file only writes each value as text (money grouped as everywhere, a percent from its
// basis points, a level as its word) and Fluent puts them into the message in the app language.
// A working this build has no words for shows the plain "no working" line, never the raw ID.
// Ctrl+I on the ⓘ opens it too. Every number of the working is in <Num>: Inter, equal-width
// digits, left to right (Code Standards F25).

import { Localized, useLocalization } from '@fluent/react';
import { Info } from 'lucide-react';
import { Popover } from 'radix-ui';
import type { Working } from '../../bindings/Working';
import type { WorkingValue } from '../../bindings/WorkingValue';
import { formatBps, formatPaisa, useGrouping, type Grouping } from '../format';
import { levelMessageId, useUiLanguage } from '../i18n';
import { SHORTCUTS, matchesShortcut } from '../keyboard';
import { Num } from './Num';

export type { Working };

/** One value of a working as the text the message shows. */
function valueText(value: WorkingValue, grouping: Grouping, word: (id: string) => string): string {
  switch (value.kind) {
    case 'money':
      return formatPaisa(value.paisa, { grouping });
    case 'percent':
      return formatBps(value.bps);
    case 'count':
      return String(value.value);
    case 'level':
      return word(levelMessageId(value.level));
  }
}

interface CalcInfoProps {
  working: Working | null;
}

export function CalcInfo({ working }: CalcInfoProps) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  const grouping = useGrouping();
  if (working === null) return null;
  // A per-نگ charge of a lot names the quantity received, not the quantity of a line.
  const messageId =
    working.message_id.endsWith('per-unit') &&
    working.args.some((arg) => arg.name === 'qty_received')
      ? `${working.message_id}-received`
      : working.message_id;
  const known = l10n.getBundle(messageId) !== null;
  const vars: Record<string, string> = {};
  for (const arg of working.args) {
    vars[arg.name] = valueText(arg.value, grouping, (id) => l10n.getString(id));
  }
  const level = vars['level'];
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
            // dir="auto": a working that is only numbers and signs ("12,000 × 6 ÷ 100 = 720") reads
            // left to right in Urdu too, and one with words reads in the direction of its words
            // (NFR-URD-13).
            <Localized id={messageId} vars={vars} elems={{ num: <Num /> }}>
              <p dir="auto" className="type-body" />
            </Localized>
          ) : (
            <Localized id="calc-info-unknown">
              <p className="type-body text-ink-muted" />
            </Localized>
          )}
          {known && level !== undefined ? (
            <Localized id="calc-info-rule" vars={{ level }}>
              <p dir="auto" className="type-body text-ink-muted" />
            </Localized>
          ) : null}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
