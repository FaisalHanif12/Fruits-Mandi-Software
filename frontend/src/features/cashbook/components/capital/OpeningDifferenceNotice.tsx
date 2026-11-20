// The warning of the opening difference (ابتدائی فرق) on the capital card (PRD FR-16.14, plan 87). It is an amber
// note, not a row of the card: what it is in plain words, what it is made of (the partners' opening capital that is
// not in روکڑ, the party openings, any other entry), each with the amount and the side the backend gives it, the
// accounts behind them (folded, one row each, each opening its ledger) and, when a partner's opening capital sits
// on the difference, the button that puts it into روکڑ. Every amount and side is the backend's: the screen adds
// nothing up (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import type { Language } from '../../../../i18n';
import { isoToDisplayDate } from '../../../../shared/format';
import { displayName, useLanguage } from '../../../../shared/i18n';
import {
  Balance,
  Button,
  FoldSection,
  LinkButton,
  Notice,
  Table,
  type TableColumn,
} from '../../../../shared/ui';
import { targetOfLink, type CapitalTarget } from '../../capital-links';
import type {
  CapitalDifference,
  CapitalDifferenceGroup,
  CapitalDifferenceKind,
  CapitalDifferencePart,
} from '../../capital-types';

interface OpeningDifferenceNoticeProps {
  difference: CapitalDifference;
  /** The button was pressed: the screen opens the box of the reason and the sign. */
  onPlace: () => void;
  onOpen: (target: CapitalTarget) => void;
}

/** The words of what made a group or a part. */
const KIND_ID = {
  partner_capital: 'partner',
  account_opening: 'party',
  other_entry: 'other',
} as const satisfies Record<CapitalDifferenceKind, string>;

/** A part with the kind of the group it is in: the folded table has one row for each. */
interface PartRow extends CapitalDifferencePart {
  kind: CapitalDifferenceKind;
}

function rowsOf(groups: readonly CapitalDifferenceGroup[]): PartRow[] {
  return groups.flatMap((group) => group.parts.map((part) => ({ ...part, kind: group.kind })));
}

/** The name of the account of a part, in the app language; an entry that is not an opening has no account. */
function accountName(part: PartRow, language: Language, fallback: string): string {
  const who = part.account ?? part.partner;
  return who === null ? fallback : displayName(who, language);
}

export function OpeningDifferenceNotice({
  difference,
  onPlace,
  onOpen,
}: OpeningDifferenceNoticeProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const canPlace = difference.placeable_partners.length > 0;
  const rows = rowsOf(difference.groups);
  const otherName = l10n.getString('cashbook-capital-warning-part-other-entry');
  const columns: readonly TableColumn<PartRow>[] = [
    {
      id: 'account',
      header: <Localized id="cashbook-capital-col-account" />,
      cell: (part) => {
        const name = accountName(part, language, otherName);
        const target = targetOfLink(part.link, { account: part.account, partner: part.partner });
        return target === null ? (
          name
        ) : (
          <LinkButton
            onClick={() => {
              onOpen(target);
            }}
          >
            {name}
          </LinkButton>
        );
      },
    },
    {
      id: 'what',
      header: <Localized id="cashbook-capital-col-what" />,
      cell: (part) => <Localized id={`cashbook-capital-warning-part-${KIND_ID[part.kind]}`} />,
    },
    {
      id: 'date',
      header: <Localized id="cashbook-capital-col-date" />,
      kind: 'number',
      size: 'date',
      cell: (part) => isoToDisplayDate(part.date),
    },
    {
      id: 'amount',
      header: <Localized id="cashbook-capital-col-opening" />,
      kind: 'number',
      size: 'money',
      cell: (part) => <Balance balance={{ amount_paisa: part.amount_paisa, side: part.side }} />,
    },
  ];
  return (
    <div className="flex flex-col gap-2">
      <Notice
        tone="warn"
        action={
          canPlace ? (
            <Button variant="primary" onClick={onPlace}>
              <Localized id="cashbook-capital-warning-place-button" />
            </Button>
          ) : undefined
        }
      >
        <span className="block type-field">
          {canPlace ? (
            <Localized id="cashbook-capital-warning-place" />
          ) : (
            <Localized
              id="cashbook-capital-warning-old"
              elems={{
                balance: (
                  <Balance
                    balance={{ amount_paisa: difference.amount_paisa, side: difference.side }}
                    inherit
                  />
                ),
              }}
            >
              <span />
            </Localized>
          )}
        </span>
        <Localized
          id={
            canPlace ? 'cashbook-capital-warning-place-text' : 'cashbook-capital-warning-old-text'
          }
        >
          <span className="block" />
        </Localized>
        {/* What the difference is made of, one line for each kind, each the backend's net and side. */}
        <dl className="mt-2 flex flex-col gap-1">
          {difference.groups.map((group) => (
            <div key={group.kind} className="flex items-baseline justify-between gap-x-4">
              <Localized id={`cashbook-capital-warning-sum-${KIND_ID[group.kind]}`}>
                <dt className="min-w-0 flex-1" />
              </Localized>
              <dd className="shrink-0 text-end">
                <Balance balance={{ amount_paisa: group.amount_paisa, side: group.side }} />
              </dd>
            </div>
          ))}
        </dl>
      </Notice>
      {/* The accounts that make the difference: one row each, folded so the card stays short (a shop with many
          openings has many rows); the heading says how many there are. */}
      <FoldSection title={<Localized id="cashbook-capital-warning-parts" />} filled={rows.length}>
        <Table
          label={l10n.getString('cashbook-capital-warning-parts')}
          columns={columns}
          rows={rows}
          rowKey={(part) =>
            `${part.kind}-${String(part.account?.id ?? part.partner?.id ?? 0)}-${part.date}`
          }
        />
      </FoldSection>
    </div>
  );
}
