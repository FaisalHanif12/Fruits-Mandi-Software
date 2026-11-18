// The print preview of a party's statement with a row for each entry (PRD FR-7.36, FR-12.47, FR-19.3, FR-19.13,
// plan 78 A7): the same page that is printed, drawn in the print language, which can differ from the app
// language. It holds every entry of the period, not one page of it. The backend sends the page as a table (a
// row for each entry, the opening balance first, totals and the closing balance in words); the shared table
// preview draws it and offers Print and Save as PDF with the language and the paper chosen in its frame.

import { Localized } from '@fluent/react';
import { useMemo } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { StatementRowsPrintInput } from '../../../bindings/StatementRowsPrintInput';
import { TablePrintPreview } from '../../reports';
import { partyLedgersStatementRowsPrint, partyLedgersStatementRowsPrintData } from '../api';

interface StatementPreviewProps {
  accountId: number;
  /** The period as typed, DD-MM-YYYY. */
  from: string;
  to: string;
  /** Print the items of each entry in full, one line each. */
  items: boolean;
  /** Print the entries an edit or a cancel replaced too. */
  history: boolean;
  onBack: () => void;
}

export function StatementPreview({
  accountId,
  from,
  to,
  items,
  history,
  onBack,
}: StatementPreviewProps) {
  const scope = useMemo(
    () => ['party_ledgers', 'statement-rows', accountId, from, to, items, history] as const,
    [accountId, from, to, items, history],
  );
  const inputOf = (language: Language, paper: Paper): StatementRowsPrintInput => ({
    account_id: accountId,
    from,
    to,
    items,
    include_history: history,
    language,
    paper,
  });
  return (
    <TablePrintPreview
      scope={scope}
      read={(language, paper) => partyLedgersStatementRowsPrintData(inputOf(language, paper))}
      send={(to, language, paper) =>
        partyLedgersStatementRowsPrint({ statement: inputOf(language, paper), to, copies: null })
      }
      title={<Localized id="statement-print-title" />}
      onBack={onBack}
    />
  );
}
