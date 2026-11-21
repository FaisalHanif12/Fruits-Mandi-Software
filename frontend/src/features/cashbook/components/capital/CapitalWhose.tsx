// "Whose it is" on the capital card (PRD FR-16.14, plan 87): each partner's کل رقم with his share of the total and a
// small bar, the opening capital and the advances still open (PRD FR-17.13). The amounts and the shares in basis
// points are the backend's; the screen only writes them (CLAUDE.md rule 1). A partner's total can be below 0 (he
// took more than he has): it is written without its minus sign, in the due colour, with the words "owes the firm",
// so no row of the card carries a minus figure. The total row is unsigned too, with the word "short" when
// the backend puts it on the `owe` side. Every line opens the page the backend names for it.

import { Localized, useLocalization } from '@fluent/react';
import { Landmark } from 'lucide-react';
import { formatBps, isNegativePaisa, isZeroPaisa, type Paisa } from '../../../../shared/format';
import { displayName, useLanguage } from '../../../../shared/i18n';
import {
  LinkButton,
  Num,
  RUNNING_NUMBER,
  StatusChip,
  Table,
  type TableColumn,
} from '../../../../shared/ui';
import { targetOfLink, type CapitalTarget } from '../../capital-links';
import type { CapitalSide, CapitalWhoseKind, CapitalWhoseLine } from '../../capital-types';
import { kindId } from '../../list-columns';

interface CapitalWhoseProps {
  lines: readonly CapitalWhoseLine[];
  /** The backend's total of the lines, unsigned, and the side it is on (`owe` is short). */
  total: Paisa;
  totalSide: CapitalSide;
  onOpen: (target: CapitalTarget) => void;
}

/** The kinds that always show, also at 0 (the others show only when the backend sends them with an amount). */
const ALWAYS: ReadonlySet<CapitalWhoseKind> = new Set(['partner_total', 'opening_capital']);

/** The bar of a share: its length is the backend's share in basis points, written as a percent. */
function ShareBar({ bps }: { bps: number }) {
  return (
    <span
      aria-hidden="true"
      className="block h-2 w-full overflow-hidden rounded-full bg-surface-sunken"
    >
      <span
        className="block h-full rounded-full bg-primary"
        style={{ inlineSize: `${formatBps(bps)}%` }}
      />
    </span>
  );
}

export function CapitalWhose({ lines, total, totalSide, onOpen }: CapitalWhoseProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const shown = lines.filter((line) => ALWAYS.has(line.kind) || !isZeroPaisa(line.amount_paisa));
  const nameOf = (line: CapitalWhoseLine) =>
    line.partner === null
      ? l10n.getString(`cashbook-capital-whose-${kindId(line.kind)}`)
      : l10n.getString('cashbook-capital-whose-partner-total', {
          partner: displayName(line.partner, language),
        });
  const columns: readonly TableColumn<CapitalWhoseLine>[] = [
    {
      id: 'what',
      header: <Localized id="cashbook-capital-col-what" />,
      cell: (line) => {
        const target = targetOfLink(line.link, { account: line.account, partner: line.partner });
        const name = nameOf(line);
        return (
          <span className="flex flex-wrap items-center gap-x-2">
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
            {isNegativePaisa(line.amount_paisa) ? (
              <StatusChip tone="warn" icon={Landmark}>
                <Localized id="cashbook-capital-whose-owes" />
              </StatusChip>
            ) : null}
          </span>
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
      cell: (line) =>
        isNegativePaisa(line.amount_paisa) ? (
          <span className="text-due">
            <Num withoutSign paisa={line.amount_paisa} />
          </span>
        ) : (
          <Num paisa={line.amount_paisa} />
        ),
      total: (
        <span className="inline-flex items-baseline gap-2">
          <Num paisa={total} />
          {totalSide === 'owe' ? (
            <Localized id="cashbook-capital-net-owe">
              <span className="type-label" />
            </Localized>
          ) : null}
        </span>
      ),
    },
    {
      id: 'share',
      header: <Localized id="cashbook-capital-col-share" />,
      kind: 'number',
      size: 'money',
      cell: (line) =>
        line.share_bps === null ? null : (
          <span className="flex flex-col items-center gap-1">
            <Localized
              id="cashbook-capital-share"
              vars={{ percent: formatBps(line.share_bps) }}
              elems={RUNNING_NUMBER}
            >
              <span />
            </Localized>
            <ShareBar bps={line.share_bps} />
          </span>
        ),
    },
  ];
  return (
    <section className="flex min-w-0 flex-col gap-2">
      <Localized id="cashbook-capital-whose-title">
        <h3 className="type-label text-ink" />
      </Localized>
      <Table
        label={l10n.getString('cashbook-capital-whose-title')}
        columns={columns}
        rows={shown}
        rowKey={(line) => `${line.kind}-${String(line.partner?.id ?? line.account?.id ?? 0)}`}
      />
    </section>
  );
}
