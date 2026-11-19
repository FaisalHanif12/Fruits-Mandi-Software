// "Edit history" (plan 44, PRD FR-7.71): every version of one entry, newest first as the backend
// sends them: when, who, the reason, and for each field the old value against the new one. It is
// read from the audit log; nothing here can change it. A field is named by its path, exactly as the
// backend names it, like the audit page does.

import { Localized, useLocalization } from '@fluent/react';
import { History } from 'lucide-react';
import { isoToDisplayDate, useGrouping } from '../format';
import { useLanguage } from '../i18n';
import type { EditEntity } from '../../bindings/EditEntity';
import type { EditVersion } from '../../bindings/EditVersion';
import { useEditHistory } from '../edit/hooks';
import { whenWhoMessage } from '../edit/sign';
import { auditValueText } from '../edit/value-text';
import { uiErrorOf } from '../api';
import { Button } from './Button';
import { EmptyState } from './EmptyState';
import { ErrorState } from './ErrorState';
import { FormDialog } from './FormDialog';
import { LoadingState } from './LoadingState';
import { stampTimeText } from './EditStampLine';
import { Table, type TableColumn } from './Table';

interface EditHistoryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  entity: EditEntity;
  id: number;
}

/** The message of an action of the audit log, or the "other action" word. */
function actionId(action: string): string {
  return action === 'create' || action === 'edit' || action === 'cancel'
    ? `edit-history-action-${action}`
    : 'edit-history-action-other';
}

function Version({ version }: { version: EditVersion }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const ctx = { l10n, language, grouping };
  // The sign given when the day was reopened follows the name (PRD FR-7.71, v1.36).
  const when = whenWhoMessage('edit-history-version', version, {
    date: isoToDisplayDate(version.local_date),
    time: stampTimeText(version.local_time, l10n),
    by: version.by,
  });
  const columns: readonly TableColumn<EditVersion['changes'][number]>[] = [
    {
      id: 'field',
      header: l10n.getString('edit-history-field'),
      cell: (change) => <bdi dir="ltr">{change.path}</bdi>,
    },
    {
      id: 'old',
      header: l10n.getString('edit-history-old'),
      cell: (change) => auditValueText(change.old, ctx),
    },
    {
      id: 'new',
      header: l10n.getString('edit-history-new'),
      cell: (change) => auditValueText(change.new, ctx),
    },
  ];
  return (
    <section className="flex flex-col gap-2" data-edit-version="">
      <h3 className="type-body font-semibold text-ink">
        <Localized id={when.id} vars={when.vars}>
          <span />
        </Localized>{' '}
        <Localized id={actionId(version.action)}>
          <span className="font-normal" />
        </Localized>
      </h3>
      {version.reason === null || version.reason === '' ? null : (
        <Localized id="edit-stamp-reason" vars={{ reason: version.reason }}>
          <p className="type-body text-ink" />
        </Localized>
      )}
      {version.changes.length === 0 ? null : (
        <Table
          label={l10n.getString('edit-history-title')}
          columns={columns}
          rows={version.changes}
          rowKey={(change) => change.path}
        />
      )}
      {version.changes_truncated ? (
        <Localized id="edit-history-truncated">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
    </section>
  );
}

export function EditHistoryDialog({ open, onOpenChange, entity, id }: EditHistoryDialogProps) {
  const history = useEditHistory(entity, id, open);
  let body;
  if (history.isPending) {
    body = <LoadingState />;
  } else if (history.isError) {
    body = (
      <ErrorState
        title={<Localized id="edit-history-failed" />}
        error={uiErrorOf(history.error)}
        onRetry={() => {
          void history.refetch();
        }}
      />
    );
  } else if (history.data.versions.length === 0) {
    body = (
      <EmptyState
        icon={History}
        title={<Localized id="edit-history-empty" />}
        help={<Localized id="edit-history-empty-help" />}
      />
    );
  } else {
    body = (
      <div className="flex flex-col gap-4">
        {history.data.versions.map((version) => (
          <Version key={version.audit_id} version={version} />
        ))}
      </div>
    );
  }
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="edit-history-title" />}
    >
      <div className="flex flex-col gap-4">
        {body}
        <div className="flex justify-end">
          <Button
            onClick={() => {
              onOpenChange(false);
            }}
          >
            <Localized id="edit-history-close" />
          </Button>
        </div>
      </div>
    </FormDialog>
  );
}
