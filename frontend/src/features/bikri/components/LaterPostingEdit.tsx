// The edit of a later sale (PRD FR-12.18, FR-12.17, plan 16): the lines of that one posting. The lot
// and the day are kept and shown read-only; only the lines can be changed, added or removed. It is
// the form of a later sale (LinesForm) with the posting loaded from the backend, the changed cells
// marked, the list "What you changed", and the reason asked for when the day is not today.

import { Localized, useLocalization } from '@fluent/react';
import { useMemo } from 'react';
import type { BikriEdited } from '../../../bindings/BikriEdited';
import type { BikriPreview } from '../../../bindings/BikriPreview';
import type { EditableBikri } from '../../../bindings/EditableBikri';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Card, Code, Fact, Num } from '../../../shared/ui';
import { useBikriDraft } from '../draft-store';
import type { Baseline } from '../edit-changes';
import { useEditDiff } from '../hooks/useEditDiff';
import type { EditReason } from '../hooks/useEditReason';
import { changedCellWords } from '../edit-view';
import { editSource } from '../preview-source';
import type { SaveMutation } from '../hooks/useSaveFlow';
import { LOT } from '../types';
import { ClosedDayNotice } from './ClosedDayNotice';
import { EditChangesCard } from './EditChangesCard';
import { LinesForm } from './LinesForm';

interface LaterPostingEditProps {
  editable: EditableBikri;
  baseline: Baseline;
  /** The backend's numbers for the entry as loaded. */
  before: BikriPreview | null;
  mutation: SaveMutation<BikriEdited>;
  reason: EditReason;
  needsReason: boolean;
  done: boolean;
  onSaved: () => void;
  onBack: () => void;
}

export function LaterPostingEdit({
  editable,
  baseline,
  before,
  mutation,
  reason,
  needsReason,
  done,
  onSaved,
  onBack,
}: LaterPostingEditProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const stores = useBikriDraft();
  const lotRow = stores.lots((state) => state.rows)[0];
  const lotKey = lotRow?.rowId ?? '';
  const storedLot = editable.stored[0] ?? null;

  const diff = useEditDiff(stores, baseline);
  const say = useMemo(
    () => (id: string, vars?: Record<string, string>) => l10n.getString(id, vars),
    [l10n],
  );
  const changed = useMemo(
    () => changedCellWords(diff.saleCells, 'line', say, language),
    [diff, say, language],
  );
  const source = useMemo(() => editSource(editable.entry_id), [editable.entry_id]);
  const seller = lotRow?.picked?.[LOT.seller];
  const item = lotRow?.picked?.[LOT.item];
  const packing = lotRow?.picked?.[LOT.packing];

  return (
    <LinesForm
      lotKey={lotKey}
      source={source}
      mutation={mutation}
      toastId="bikri-edit-saved-toast"
      onSaved={onSaved}
      title={
        <Localized id="bikri-edit-title" vars={{ number: String(editable.number) }}>
          <span />
        </Localized>
      }
      help={
        <Localized
          id="bikri-edit-help-later"
          vars={{ date: isoToDisplayDate(editable.entry_date) }}
        >
          <span />
        </Localized>
      }
      notice={editable.editable ? null : <ClosedDayNotice date={editable.entry_date} />}
      dayField={
        <dl>
          <Fact label={<Localized id="bikri-later-date" />}>
            <Code>{isoToDisplayDate(editable.entry_date)}</Code>
          </Fact>
        </dl>
      }
      cellChanged={changed}
      dirty={done ? false : !diff.unchanged}
      saveId="bikri-edit-save"
      canSave={editable.editable}
      reason={{ store: reason, needed: needsReason, day: editable.entry_date }}
      onBack={onBack}
      below={(after) => (
        <EditChangesCard
          items={diff.items}
          before={before?.lots[0]?.totals ?? null}
          after={after}
        />
      )}
    >
      <Card compact title={<Localized id="bikri-later-lot-title" />}>
        <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-5">
          {seller === undefined ? null : (
            <Fact label={<Localized id="glossary-beopari" />}>{displayName(seller, language)}</Fact>
          )}
          {item === undefined ? null : (
            <Fact label={<Localized id="glossary-qism-maal" />}>{displayName(item, language)}</Fact>
          )}
          {packing === undefined ? null : (
            <Fact label={<Localized id="glossary-packing" />}>
              {displayName(packing, language)}
            </Fact>
          )}
          {storedLot === null ? null : (
            <Fact label={<Localized id="bikri-col-qty-received" />}>
              <Num>{storedLot.qty_received}</Num>
            </Fact>
          )}
        </dl>
      </Card>
    </LinesForm>
  );
}
