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
  /**
   * The ⓘ sits inside a grid cell: a 32 px target instead of 40, and out of the Tab order (the
   * grid has its own keys, and Ctrl+I opens the working from the cell, PRD FR-7.43).
   */
  inCell?: boolean;
}

/** The Fluent message of a working, the values for it, and whether this build has its words. */
function useWorkingMessage(working: Working | null) {
  const { l10n } = useLocalization();
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
  return { messageId, known, vars };
}

/** The words of a working: the title, the sum with the real numbers, and the level of the rule. */
function WorkingBody({ working }: { working: Working }) {
  const message = useWorkingMessage(working);
  if (message === null) return null;
  const { messageId, known, vars } = message;
  const level = vars['level'];
  return (
    <>
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
    </>
  );
}

const CONTENT_CLASSES =
  'z-30 flex max-w-reading flex-col gap-1 rounded-2 border border-border bg-surface p-3 text-ink shadow-overlay';

export function CalcInfo({ working, inCell = false }: CalcInfoProps) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  if (working === null) return null;
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
        tabIndex={inCell ? -1 : undefined}
        className={`inline-flex shrink-0 items-center justify-center rounded-1 text-ink-muted transition-calm ${inCell ? 'size-target' : 'size-control-compact'}`}
      >
        <Info aria-hidden="true" className="size-icon-sm" />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content lang={language} dir={direction} sideOffset={4} className={CONTENT_CLASSES}>
          <WorkingBody working={working} />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

interface CalcInfoAtProps {
  working: Working;
  /** The cell or field the working is about; the box opens next to it. */
  anchor: HTMLElement;
  /** Esc, or a click outside, closes the box. */
  onClose: () => void;
}

/**
 * The same working, opened by a key (Ctrl+I on a calculated grid cell, PRD FR-7.43) next to the
 * element it is about. The cell is not a button, so the box is opened by the screen and anchored
 * to the cell. Focus goes back to the cell when it closes.
 */
export function CalcInfoAt({ working, anchor, onClose }: CalcInfoAtProps) {
  const { language, direction } = useUiLanguage();
  return (
    <Popover.Root
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <Popover.Anchor virtualRef={{ current: anchor }} />
      <Popover.Portal>
        <Popover.Content
          lang={language}
          dir={direction}
          sideOffset={4}
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            if (anchor.isConnected) anchor.focus();
          }}
          className={CONTENT_CLASSES}
        >
          <WorkingBody working={working} />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
