// One audit entry opened (PRD FR-21.3): who did it, when, what it was, the reason, and every field that
// changed with its old and new value, side by side. A secret (a password, a key) is never shown: the
// backend sends `hidden` and the screen says only that it changed. Money is the backend's amount
// through the list's formatter. Esc or the button goes back to the list.

import { Localized, useLocalization } from '@fluent/react';
import { ArrowLeft } from 'lucide-react';
import { useMemo, type ReactNode } from 'react';
import type { AuditRow } from '../../../bindings/AuditRow';
import { ListGrid } from '../../../shared/grid';
import { Button, Card, Code, Num } from '../../../shared/ui';
import { actionWord, entityWord } from '../words';
import { CHANGE_COLUMNS, whenText, type ChangeRow } from '../columns';

interface AuditDetailProps {
  row: AuditRow;
  onBack: () => void;
}

function DetailFact({ labelId, children }: { labelId: string; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-1">
      <dt className="type-body text-ink-muted">
        <Localized id={labelId} />
      </dt>
      <dd className="type-body">{children}</dd>
    </div>
  );
}

export function AuditDetail({ row, onBack }: AuditDetailProps) {
  const { l10n } = useLocalization();
  const text = (id: string) => l10n.getString(id);
  const changes = useMemo<ChangeRow[]>(
    () => row.changes.map((change, index) => ({ key: String(index), change })),
    [row],
  );
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Localized id="audit-detail-title" vars={{ number: String(row.id) }}>
          <h3 className="type-section" />
        </Localized>
        <Button onClick={onBack}>
          <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
          <Localized id="audit-detail-back" />
        </Button>
      </div>
      <Card compact>
        <dl className="grid grid-cols-3 gap-x-6 gap-y-3">
          <DetailFact labelId="audit-col-when">
            <Num>{whenText(row)}</Num>
          </DetailFact>
          <DetailFact labelId="audit-col-user">
            {row.user?.display_name ?? text('audit-system-user')}
          </DetailFact>
          <DetailFact labelId="audit-col-action">{actionWord(row.action, text)}</DetailFact>
          <DetailFact labelId="audit-col-entity">{entityWord(row.entity, text)}</DetailFact>
          {row.entity_id === null ? null : (
            <DetailFact labelId="audit-col-record">
              <Code>{String(row.entity_id)}</Code>
            </DetailFact>
          )}
          {row.reason === null ? null : (
            <DetailFact labelId="audit-col-reason">{row.reason}</DetailFact>
          )}
        </dl>
      </Card>
      {changes.length === 0 ? (
        <Localized id="audit-detail-none">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <ListGrid<ChangeRow>
          columns={CHANGE_COLUMNS}
          rows={changes}
          getRowId={(item) => item.key}
          labelId="audit-detail-grid-label"
          onOpen={() => undefined}
          heightMode="auto"
        />
      )}
      {row.changes_truncated ? (
        <Localized id="audit-detail-truncated" vars={{ shown: String(row.changes.length) }}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
    </div>
  );
}
