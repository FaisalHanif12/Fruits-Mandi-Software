// The quick questions (PRD FR-7.80): a row of round chips (plan 51) that answer the common questions at once from the
// app alone, with no internet and no model. They are not counted. The last one, "کسی پارٹی کا کھاتہ",
// opens a party picker inside the panel (the shared `AccountPicker`, which searches the raw text),
// and the party chosen is sent with the chip.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { QuickChip } from '../../../bindings/QuickChip';
import { AccountPicker, Button, Field } from '../../../shared/ui';
import type { AnswerName } from '../../../bindings/AnswerName';
import { RoundChip } from './RoundChip';

/** The chips in the order of PRD FR-7.80, each with the Fluent message of its words. */
export const QUICK_CHIPS: readonly { chip: QuickChip; messageId: string }[] = [
  { chip: 'today_sales', messageId: 'assistant-chip-sales' },
  { chip: 'today_commission', messageId: 'assistant-chip-commission' },
  { chip: 'cash_in_hand', messageId: 'assistant-chip-cash' },
  { chip: 'buyers_dues', messageId: 'assistant-chip-dues' },
  { chip: 'total_dues', messageId: 'assistant-chip-total-dues' },
  { chip: 'dead_amount', messageId: 'assistant-chip-dead' },
  { chip: 'sellers_payable', messageId: 'assistant-chip-payable' },
  { chip: 'advances_open', messageId: 'assistant-chip-advances' },
  { chip: 'stock_left', messageId: 'assistant-chip-stock' },
  { chip: 'party_ledger', messageId: 'assistant-chip-party' },
];

interface QuickChipsProps {
  /** A quick question to send; the party is for the party chip only. */
  onQuick: (chip: QuickChip, party: { id: number; name: AnswerName } | null) => void;
  /** A question is being worked on: the chips wait. */
  busy: boolean;
  /** The party list opens above its field: the chips are in the footer of the window (plan 51). */
  pickerAbove?: boolean;
}

export function QuickChips({ onQuick, busy, pickerAbove = false }: QuickChipsProps) {
  const { l10n } = useLocalization();
  const [choosing, setChoosing] = useState(false);
  if (choosing) {
    return (
      <div className="flex flex-wrap items-end gap-3">
        <div className="min-w-0 max-w-container-field flex-1">
          <Field
            label={<Localized id="assistant-party-pick-label" />}
            hint={<Localized id="assistant-party-pick-hint" />}
          >
            {(control) => (
              <AccountPicker
                {...control}
                scope="any"
                value={null}
                showCard={false}
                canAdd={false}
                listAbove={pickerAbove}
                onChange={(hit) => {
                  if (hit === null) return;
                  setChoosing(false);
                  onQuick('party_ledger', {
                    id: hit.id,
                    name: {
                      name_ur: hit.name_ur,
                      name_en: hit.name_en,
                      tag: hit.tag ?? null,
                      code: hit.code,
                    },
                  });
                }}
              />
            )}
          </Field>
        </div>
        <Button
          round
          onClick={() => {
            setChoosing(false);
          }}
        >
          <Localized id="assistant-party-cancel" />
        </Button>
      </div>
    );
  }
  return (
    <ul
      aria-label={l10n.getString('assistant-chips-label')}
      className="flex list-none flex-wrap gap-2"
    >
      {QUICK_CHIPS.map(({ chip, messageId }) => (
        <li key={chip}>
          <RoundChip
            disabled={busy}
            onClick={() => {
              if (chip === 'party_ledger') setChoosing(true);
              else onQuick(chip, null);
            }}
          >
            <Localized id={messageId} />
          </RoundChip>
        </li>
      ))}
    </ul>
  );
}
