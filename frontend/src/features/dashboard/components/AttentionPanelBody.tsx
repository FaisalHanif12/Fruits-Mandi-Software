// The inside of the bell's panel (PRD FR-7.44, plan 100 J): three sections. First the sellers to pay and
// the buyers to collect from, the top five of each by amount with the days, the backend's total and a
// button that opens the whole list (the seller dues page on "the shop has to pay", the buyer dues page).
// Then the general reminders: the payables of مزدوری, کرایہ, کرایہ سٹور and منشیانہ, each with a button that
// opens ادائیگی with that account chosen, and the other lines of the attention list. Every item has "Later",
// which hides it until the next بکری is saved. Every figure is the backend's; nothing is added up here
// (CLAUDE.md rule 1). ↑ ↓ move between the buttons of a section and Enter opens (PRD FR-7.17). Colour is
// never the only signal: every section has its name and every line its words.

import { Localized, useLocalization } from '@fluent/react';
import { Clock3, HandCoins, Info, Users, type LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';
import type { AttentionPanel } from '../../../bindings/AttentionPanel';
import type { AttentionPartyGroup } from '../../../bindings/AttentionPartyGroup';
import type { AttentionPartyRow } from '../../../bindings/AttentionPartyRow';
import type { AttentionPayableRow } from '../../../bindings/AttentionPayableRow';
import { displayName, useLanguage } from '../../../shared/i18n';
import { useRovingKeys } from '../../../shared/keyboard';
import { Button, ErrorNotice, IconChip, Num, RUNNING_NUMBER, ScriptText } from '../../../shared/ui';
import { uiErrorOf } from '../../../shared/api';
import { useSnooze } from '../hooks';
import type { Destination } from '../destinations';
import { AttentionLines } from './AttentionLines';

interface AttentionPanelBodyProps {
  panel: AttentionPanel;
  /** A row or a button was chosen. */
  onOpen: (destination: Destination) => void;
}

interface SectionProps {
  icon: LucideIcon;
  titleId: string;
  /** The backend's total of the whole group, or nothing where the section has none. */
  total?: ReactNode;
  children: ReactNode;
}

/** One section of the panel: its picture and name, its total at the end, then its lines. */
function Section({ icon, titleId, total, children }: SectionProps) {
  return (
    <section className="flex min-w-0 flex-col gap-2 rounded-2 border border-border p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-2">
          <IconChip icon={icon} />
          <h3 className="type-label text-ink">
            <Localized id={titleId} />
          </h3>
        </div>
        {total}
      </div>
      {children}
    </section>
  );
}

/** "Later": hides the item until the next بکری is saved. Its full name says which item, for a screen reader. */
function LaterButton({
  itemKey,
  fullName,
  snooze,
}: {
  itemKey: string;
  fullName: string;
  snooze: ReturnType<typeof useSnooze>;
}) {
  return (
    <Button
      variant="ghost"
      small
      accessibleName={fullName}
      busy={snooze.isPending && snooze.variables === itemKey}
      onClick={() => {
        snooze.mutate(itemKey);
      }}
    >
      <Clock3 aria-hidden="true" className="size-icon-sm" />
      <Localized id="attention-later" />
    </Button>
  );
}

/** One seller or buyer: his name, the amount and the days; a click opens his history. */
function PartyLine({
  row,
  side,
  onOpen,
  snooze,
}: {
  row: AttentionPartyRow;
  side: 'seller' | 'buyer';
  onOpen: (destination: Destination) => void;
  snooze: ReturnType<typeof useSnooze>;
}) {
  const language = useLanguage();
  const { l10n } = useLocalization();
  const name = displayName(row.party, language);
  return (
    <li className="flex items-center gap-2">
      <button
        type="button"
        data-nav-item
        onClick={() => {
          onOpen({ kind: 'party-statement', side, party: row.party.account_id });
        }}
        className="flex min-h-target min-w-0 flex-1 items-center gap-3 rounded-2 px-2 text-start transition-calm hover:bg-surface-sunken"
      >
        <span className="type-body min-w-0 flex-1">
          <ScriptText>{name}</ScriptText>
        </span>
        {row.days === null ? null : (
          <Localized id="attention-row-days" vars={{ days: row.days }} elems={RUNNING_NUMBER}>
            <span className="type-body shrink-0 text-ink-muted" />
          </Localized>
        )}
        <span className="shrink-0">
          <Num paisa={row.amount_paisa} />
        </span>
      </button>
      <LaterButton
        itemKey={row.item_key}
        fullName={l10n.getString('attention-later-label', { name })}
        snooze={snooze}
      />
    </li>
  );
}

/** What each group of people says when it has nobody. */
const EMPTY_IDS = { seller: 'attention-sellers-none', buyer: 'attention-buyers-none' } as const;

/** The sellers or the buyers: the top five, the group's total and the way to the whole list. */
function PartyGroup({
  group,
  side,
  icon,
  titleId,
  all,
  onOpen,
  snooze,
}: {
  group: AttentionPartyGroup;
  side: 'seller' | 'buyer';
  icon: LucideIcon;
  titleId: string;
  /** Where "See all" goes. */
  all: Destination;
  onOpen: (destination: Destination) => void;
  snooze: ReturnType<typeof useSnooze>;
}) {
  const keys = useRovingKeys();
  const { l10n } = useLocalization();
  return (
    <Section
      icon={icon}
      titleId={titleId}
      total={
        <Localized
          id="attention-section-total"
          elems={{ money: <Num inherit paisa={group.total_paisa} /> }}
        >
          <span className="type-body text-ink" />
        </Localized>
      }
    >
      {group.rows.length === 0 ? (
        <Localized id={EMPTY_IDS[side]}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <ul onKeyDown={keys} className="flex flex-col gap-1">
          {group.rows.map((row) => (
            <PartyLine key={row.item_key} row={row} side={side} onOpen={onOpen} snooze={snooze} />
          ))}
        </ul>
      )}
      <div>
        <Button
          accessibleName={l10n.getString('attention-see-all-label', {
            section: l10n.getString(titleId),
          })}
          onClick={() => {
            onOpen(all);
          }}
        >
          <Localized id="attention-see-all" />
          <Localized
            id="attention-see-all-count"
            vars={{ count: group.count }}
            elems={RUNNING_NUMBER}
          >
            <span className="type-body text-ink-muted" />
          </Localized>
        </Button>
      </div>
    </Section>
  );
}

/** A payable from the sales still to pay (head 02): a button opens ادائیگی with that account chosen. */
function PayableLine({
  row,
  onOpen,
  snooze,
}: {
  row: AttentionPayableRow;
  onOpen: (destination: Destination) => void;
  snooze: ReturnType<typeof useSnooze>;
}) {
  const language = useLanguage();
  const { l10n } = useLocalization();
  const name = displayName(row, language);
  return (
    <li className="flex flex-wrap items-center gap-2 px-2" data-attention-payable={row.account_id}>
      <span className="type-body min-w-0 flex-1">
        <ScriptText>{name}</ScriptText>
      </span>
      <Num paisa={row.amount_paisa} />
      <Button
        small
        accessibleName={l10n.getString('attention-pay-label', { name })}
        onClick={() => {
          onOpen({
            kind: 'payment',
            party: {
              id: row.account_id,
              code: row.code,
              name_ur: row.name_ur,
              name_en: row.name_en,
            },
          });
        }}
      >
        <HandCoins aria-hidden="true" className="size-icon-sm" />
        <Localized id="attention-pay" />
      </Button>
      <LaterButton
        itemKey={row.item_key}
        fullName={l10n.getString('attention-later-label', { name })}
        snooze={snooze}
      />
    </li>
  );
}

export function AttentionPanelBody({ panel, onOpen }: AttentionPanelBodyProps) {
  const snooze = useSnooze();
  const { l10n } = useLocalization();
  const nothingElse = panel.payables.length === 0 && panel.general.length === 0;
  return (
    <div className="@container flex min-w-0 flex-col gap-3">
      {snooze.isError ? <ErrorNotice error={uiErrorOf(snooze.error)} /> : null}
      <div className="grid grid-cols-1 gap-3 @form3:grid-cols-2">
        <PartyGroup
          group={panel.sellers}
          side="seller"
          icon={HandCoins}
          titleId="attention-section-sellers"
          all={{ kind: 'seller-dues', side: 'shop_owes' }}
          onOpen={onOpen}
          snooze={snooze}
        />
        <PartyGroup
          group={panel.buyers}
          side="buyer"
          icon={Users}
          titleId="attention-section-buyers"
          all={{ kind: 'buyer-dues' }}
          onOpen={onOpen}
          snooze={snooze}
        />
      </div>
      <Section icon={Info} titleId="attention-section-general">
        {panel.payables.length === 0 ? null : (
          <ul className="flex flex-col gap-1">
            {panel.payables.map((row) => (
              <PayableLine key={row.item_key} row={row} onOpen={onOpen} snooze={snooze} />
            ))}
          </ul>
        )}
        {panel.general.length === 0 ? null : (
          <AttentionLines
            items={panel.general}
            onOpen={onOpen}
            action={(item) => (
              <LaterButton
                itemKey={item.item_key}
                fullName={l10n.getString('attention-later-reminder')}
                snooze={snooze}
              />
            )}
          />
        )}
        {nothingElse ? (
          <Localized id="attention-general-none">
            <p className="type-body text-ink-muted" />
          </Localized>
        ) : null}
      </Section>
      {panel.hidden_count === 0 ? null : (
        <Localized
          id="attention-hidden-note"
          vars={{ count: panel.hidden_count }}
          elems={RUNNING_NUMBER}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
    </div>
  );
}
