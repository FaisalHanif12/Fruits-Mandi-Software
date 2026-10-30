// The header of the sale book folded to one line (plan 38, sale book, tech lead's answer for a short
// window): date, truck, زمیندار and اڈا as text, while the user types in the lots or the sale lines. The
// real fields stay in the page (visually hidden, still in the Tab order), so the keyboard order is the
// same as ever and any field that gets focus opens the header again at once. A click on this line opens
// it too. It is only drawn in a short window (`short:`); a taller window keeps the header open.
// It is not read out: the real fields are, and they say the same.

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import { Code, Num } from '../../../shared/ui';
import { displayName, useLanguage } from '../../../shared/i18n';
import type { HeaderState } from '../draft-store';

interface HeaderSummaryProps {
  header: HeaderState;
  /** A press on the line: the date field takes focus and the header opens. */
  onOpen: () => void;
}

function Item({ labelId, children }: { labelId: string; children: ReactNode }) {
  return (
    <span className="inline-flex items-baseline gap-2">
      <Localized id={labelId}>
        <span className="text-ink-muted" />
      </Localized>
      {children}
    </span>
  );
}

export function HeaderSummary({ header, onOpen }: HeaderSummaryProps) {
  const language = useLanguage();
  return (
    <button
      type="button"
      tabIndex={-1}
      aria-hidden="true"
      // Mouse down, not click: the button takes focus first, which opens the header and takes this
      // line away before a click could arrive. Nothing here is reached by keyboard (tabIndex -1).
      onMouseDown={(event) => {
        event.preventDefault();
        onOpen();
      }}
      className="hidden w-full flex-wrap items-baseline gap-x-4 gap-y-1 rounded-2 border border-border bg-surface px-3 py-0 text-start type-body text-ink short:flex"
    >
      <Item labelId="bikri-header-date">
        <Num>{header.entryDate}</Num>
      </Item>
      {header.truckNo.trim() === '' ? null : (
        <Item labelId="bikri-header-truck">
          <Code>{header.truckNo.trim()}</Code>
        </Item>
      )}
      {header.zamindar === null ? null : (
        <Item labelId="glossary-zamindar">
          <span>{displayName(header.zamindar, language)}</span>
        </Item>
      )}
      {header.adda === null ? null : (
        <Item labelId="glossary-adda">
          <span>{displayName(header.adda, language)}</span>
        </Item>
      )}
    </button>
  );
}
