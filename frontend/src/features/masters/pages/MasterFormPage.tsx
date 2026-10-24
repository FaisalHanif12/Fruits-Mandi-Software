// The add and edit screen of any master (route /index/<kind>/new and /index/<kind>/<id>). It reads
// what the form needs (the master for an edit, the next code for a new one) and shows the four
// states: loading, error with a retry, "not found", and the form (PRD FR-7.46, Code Standards F8).

import { Localized } from '@fluent/react';
import { useNavigate, useParams } from '@tanstack/react-router';
import { SearchX } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { Button, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { MasterForm } from '../components/MasterForm';
import { configOfPath } from '../configs';
import { useMasterRecord, useNextCode } from '../hooks';
import { UnknownMasterPage } from './UnknownMasterPage';

export function MasterFormPage() {
  const params = useParams({ strict: false });
  const config = configOfPath(params.kind ?? '');
  if (config === undefined) return <UnknownMasterPage />;
  return (
    <MasterFormLoader key={`${config.kind}-${params.id ?? 'new'}`} config={config} id={params.id} />
  );
}

function MasterFormLoader({
  config,
  id,
}: {
  config: NonNullable<ReturnType<typeof configOfPath>>;
  id: string | undefined;
}) {
  const navigate = useNavigate();
  const isNew = id === undefined;
  const list = useMasterRecord(config.kind, id);
  const nextCode = useNextCode(config.kind, isNew && config.hasCode);

  const back = () => {
    void navigate({ to: '/index/$kind', params: { kind: config.path } });
  };

  const query = isNew ? nextCode : list;
  if (isNew && !config.hasCode) {
    return <MasterForm config={config} record={null} suggestedCode="" />;
  }
  if (query.isPending) return <LoadingState />;
  if (query.isError) {
    return (
      <ErrorState
        title={<Localized id="masters-load-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          void query.refetch();
        }}
      />
    );
  }
  if (isNew) {
    return <MasterForm config={config} record={null} suggestedCode={nextCode.data ?? ''} />;
  }
  const record = list.record;
  if (record === undefined) {
    return (
      <EmptyState
        icon={SearchX}
        title={<Localized id="masters-not-found-title" />}
        help={<Localized id="masters-not-found-help" />}
        action={
          <Button variant="primary" onClick={back}>
            <Localized id="common-back" />
          </Button>
        }
      />
    );
  }
  return <MasterForm config={config} record={record} suggestedCode={record.code} />;
}
