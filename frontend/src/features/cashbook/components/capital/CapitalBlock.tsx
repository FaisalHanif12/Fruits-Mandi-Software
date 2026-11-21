// One of the two blocks of the capital card (PRD FR-16.14, plan 87): "to receive and in hand" or "to pay". A table
// of lines, each with its amount and no minus sign (the side is the block), a total row that is the backend's
// subtotal, and every line that has a page of its own opens it. Nothing is worked out here (CLAUDE.md rule 1).
//
// The main lines of a block always show, also at 0; every other line shows only when it is not 0 (PRD FR-16.14).
// Cash below zero comes from the backend as the cash line of the paying block: it is written "cash short" with a
// note, never as a minus figure. A line that is one account (what is owed for labour, freight ...) is named by it.

import { Localized, useLocalization, type ReactLocalization } from '@fluent/react';
import { isZeroPaisa, type Paisa } from '../../../../shared/format';
import type { Language } from '../../../../i18n';
import { displayName, useLanguage } from '../../../../shared/i18n';
import { LinkButton, Num, Table, type TableColumn } from '../../../../shared/ui';
import { targetOfLink, type CapitalTarget } from '../../capital-links';
import type { CapitalWhereKind, CapitalWhereLine } from '../../capital-types';
import { kindId } from '../../list-columns';

interface CapitalBlockProps {
  /** The Fluent message ID of the block's name. */
  titleId: string;
  lines: readonly CapitalWhereLine[];
  /** The backend's subtotal of the block. */
  total: Paisa;
  /** The lines that show even at 0. */
  main: ReadonlySet<CapitalWhereKind>;
  onOpen: (target: CapitalTarget) => void;
}

/** The lines of a block that show: the main ones always, the others only when they are not 0. */
export function shownLines(
  lines: readonly CapitalWhereLine[],
  main: ReadonlySet<CapitalWhereKind>,
): CapitalWhereLine[] {
  return lines.filter((line) => main.has(line.kind) || !isZeroPaisa(line.amount_paisa));
}

/** Cash in the paying block is cash below zero (PRD FR-16.14): "cash short". */
function isCashShort(line: CapitalWhereLine): boolean {
  return line.kind === 'cash' && line.side === 'owe';
}

/** The words of a line: the account's own name when the line is one account, else the words of its kind. */
function nameOf(line: CapitalWhereLine, l10n: ReactLocalization, language: Language): string {
  if (line.kind === 'owed_for_charges' && line.account !== null) {
    return displayName(line.account, language);
  }
  if (isCashShort(line)) return l10n.getString('cashbook-capital-where-cash-short');
  return l10n.getString(`cashbook-capital-where-${kindId(line.kind)}`);
}

export function CapitalBlock({ titleId, lines, total, main, onOpen }: CapitalBlockProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const columns: readonly TableColumn<CapitalWhereLine>[] = [
    {
      id: 'what',
      header: <Localized id="cashbook-capital-col-what" />,
      cell: (line) => {
        const name = nameOf(line, l10n, language);
        const target = targetOfLink(line.link, { side: line.side, account: line.account });
        return (
          <div className="flex flex-col">
            {target === null ? (
              <span className="type-body">{name}</span>
            ) : (
              <LinkButton
                onClick={() => {
                  onOpen(target);
                }}
              >
                {name}
              </LinkButton>
            )}
            {isCashShort(line) ? (
              <Localized id="cashbook-capital-cash-short-note">
                <span className="type-body text-ink-muted" />
              </Localized>
            ) : null}
          </div>
        );
      },
      total: (
        <Localized id="glossary-meezan">
          <span />
        </Localized>
      ),
    },
    {
      id: 'amount',
      header: <Localized id="cashbook-capital-col-amount" />,
      kind: 'number',
      size: 'money',
      cell: (line) => <Num paisa={line.amount_paisa} />,
      total: <Num paisa={total} />,
    },
  ];
  return (
    <section className="flex min-w-0 flex-col gap-2">
      <Localized id={titleId}>
        <h3 className="type-label text-ink" />
      </Localized>
      <Table
        label={l10n.getString(titleId)}
        columns={columns}
        rows={shownLines(lines, main)}
        rowKey={(line) => `${line.kind}-${String(line.account?.id ?? 0)}`}
      />
    </section>
  );
}
