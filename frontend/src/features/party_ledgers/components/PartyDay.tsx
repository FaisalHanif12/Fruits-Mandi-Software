// Level 3 of the party ledgers screen (PRD FR-12.43): the full slip of one party on one date, with a
// print button. It is the R35 day slip of the `reports` module for this party and date, drawn by
// the report screen's own pieces, so every figure equals the report, the party card and R13 (plan
// 66). F9 or the button shows the print preview (A4, A5, 80 mm, Urdu or English, PDF, Excel);
// Esc closes the preview, then goes back to the statement.

import { Localized } from '@fluent/react';
import { ArrowLeft, Printer } from 'lucide-react';
import { useCallback, useState } from 'react';
import type { LedgerSide } from '../../../bindings/LedgerSide';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import { Button, Code, ListPage } from '../../../shared/ui';
import { PartyDayPreview, PartyDaySlip } from '../../reports';
import { usePartyStatement } from '../hooks';
import { PartyMarks } from './PartyMarks';

interface PartyDayProps {
  accountId: number;
  /** The date of the slip, `YYYY-MM-DD`. */
  date: string;
  /** Back to the statement of the party; told the kind the backend says the party is. */
  onBack: (kind: LedgerSide | null) => void;
}

export function PartyDay({ accountId, date, onBack }: PartyDayProps) {
  const language = useLanguage();
  const [printing, setPrinting] = useState(false);
  // The names, the kind and the marks of the party come with his statement (the same answer level 2
  // read, so it is usually in the cache).
  const statement = usePartyStatement(accountId, '', '');
  const head = statement.data?.pages[0]?.head ?? null;
  const kind: LedgerSide | null = head === null ? null : head.kind;

  const back = useCallback(() => {
    onBack(kind);
  }, [onBack, kind]);
  useEscape('leave', () => {
    if (printing) setPrinting(false);
    else back();
  });
  useShortcut(SHORTCUTS.print, () => {
    if (!printing) setPrinting(true);
  });

  return (
    <ListPage
      titleId="party-ledgers-title"
      help={
        <Localized id="party-ledgers-help">
          <p />
        </Localized>
      }
      // In the print preview the frame has its own back button (to the slip), so only the slip has these.
      actions={
        printing ? undefined : (
          <>
            <Button onClick={back}>
              <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
              <Localized id="party-ledgers-back-statement" />
            </Button>
            <Button
              variant="primary"
              shortcut={SHORTCUTS.print}
              onClick={() => {
                setPrinting(true);
              }}
            >
              <Printer aria-hidden="true" className="size-icon-md" />
              <Localized id="report-preview" />
            </Button>
          </>
        )
      }
      filters={
        head === null ? undefined : (
          <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
            <Localized
              id="party-ledgers-day-title"
              vars={{
                name: displayName(head, language),
                code: head.code,
                date: isoToDisplayDate(date),
              }}
              elems={{ name: <span className="type-section" />, code: <Code />, date: <Code /> }}
            >
              <h2 />
            </Localized>
            <PartyMarks isDead={head.is_dead} isActive={head.is_active} />
          </div>
        )
      }
    >
      {printing ? (
        <div className="min-h-0 flex-1 overflow-auto">
          <PartyDayPreview
            accountId={accountId}
            date={date}
            onBack={() => {
              setPrinting(false);
            }}
          />
        </div>
      ) : (
        <PartyDaySlip accountId={accountId} date={date} />
      )}
    </ListPage>
  );
}
