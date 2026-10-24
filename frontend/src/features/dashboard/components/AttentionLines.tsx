// The lines of the attention list (PRD FR-7.44), drawn the same on the dashboard (آج کے کام) and in
// the bell. Each line says what needs doing in words, with the figures the backend sent, and opens
// the right screen. When the backend names the most urgent record, a second button opens that
// record itself (a ledger, a statement, a بکری). ↑ ↓ move between the buttons and Enter opens
// (PRD FR-7.17). Colour is never the only signal: every line has its icon and its words.

import { Localized } from '@fluent/react';
import { ChevronRight } from 'lucide-react';
import type { AttentionItem } from '../../../bindings/AttentionItem';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { useRovingKeys } from '../../../shared/keyboard';
import { Num, ScriptText } from '../../../shared/ui';
import { KindIcon } from './KindIcon';
import { wordsOfItem } from '../attention';
import { destinationOfFirst, destinationOfKind, type Destination } from '../destinations';

interface AttentionLinesProps {
  items: readonly AttentionItem[];
  /** A line, or the name under it, was chosen. */
  onOpen: (destination: Destination) => void;
}

function Line({ item, onOpen }: { item: AttentionItem; onOpen: (to: Destination) => void }) {
  const language = useLanguage();
  const grouping = useGrouping();
  const words = wordsOfItem(item, (amount) => formatPaisa(amount, { grouping }));
  const first = item.first;
  const firstName =
    first === null
      ? ''
      : displayName({ name_ur: first.name_ur ?? '', name_en: first.name_en ?? '' }, language);
  return (
    <li className="flex flex-col">
      <button
        type="button"
        data-nav-item
        data-attention-kind={item.kind}
        onClick={() => {
          onOpen(destinationOfKind(item.kind));
        }}
        className="flex min-h-target min-w-0 flex-1 items-center gap-3 rounded-2 px-2 text-start transition-calm hover:bg-surface-sunken"
      >
        <KindIcon kind={item.kind} />
        <Localized id={words.id} vars={words.vars} elems={{ num: <Num /> }}>
          <span className="type-body min-w-0" />
        </Localized>
        <ChevronRight
          aria-hidden="true"
          className="ms-auto size-icon-md shrink-0 text-ink-muted rtl:rotate-180"
        />
      </button>
      {first === null || firstName === '' ? null : (
        <button
          type="button"
          data-nav-item
          onClick={() => {
            onOpen(destinationOfFirst(first));
          }}
          className="type-body ms-7 min-h-target self-start rounded-1 px-2 text-primary underline transition-calm"
        >
          <Localized
            id="attention-open-first"
            vars={{ name: firstName }}
            elems={{ name: <ScriptText /> }}
          >
            <span />
          </Localized>
        </button>
      )}
    </li>
  );
}

export function AttentionLines({ items, onOpen }: AttentionLinesProps) {
  const keys = useRovingKeys();
  return (
    <ul onKeyDown={keys} className="flex flex-col gap-1">
      {items.map((item) => (
        <Line key={item.kind} item={item} onOpen={onOpen} />
      ))}
    </ul>
  );
}
