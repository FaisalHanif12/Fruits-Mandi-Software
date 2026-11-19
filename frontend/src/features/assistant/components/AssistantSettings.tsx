// The Settings group "سوال پوچھیں" (PRD FR-7.85): on or off, the OpenRouter key, the model, the
// connection test, today's count of the 20 and the question log. The key is only ever in the paste
// field until it is sent: the field is masked, the text lives in this component's state, it is sent
// with the Owner's password, and after that only the last 4 characters are shown (NFR-SEC-15). It is
// not put into a store, the query cache, a log or a test. The daily limit of 20 is fixed, not a
// setting. Four states for the log: loading, an error with a retry, nothing yet, and the rows.

import { Localized, useLocalization, type ReactLocalization } from '@fluent/react';
import { History, MessageCircleQuestion } from 'lucide-react';
import { useState } from 'react';
import type { AssistantFailure } from '../../../bindings/AssistantFailure';
import type { AssistantLogRow } from '../../../bindings/AssistantLogRow';
import type { AssistantStatus } from '../../../bindings/AssistantStatus';
import { uiErrorOf } from '../../../shared/api';
import { utcToLocalDisplay } from '../../../shared/format';
import { Pager } from '../../../shared/grid';
import {
  Button,
  Card,
  Checkbox,
  EmptyState,
  ErrorNotice,
  ErrorState,
  Field,
  LoadingState,
  Notice,
  PasswordInput,
  RUNNING_NUMBER,
  Table,
  TextInput,
  fieldError,
  showToast,
  type TableColumn,
} from '../../../shared/ui';
import {
  useAssistantLog,
  useAssistantRunner,
  useAssistantStatus,
  useRemoveAssistantKey,
  useSaveAssistantKey,
  useSetAssistant,
  useTestAssistant,
} from '../hooks';
import { useAssistantStore } from '../store';
import { shownDuration, type DurationUnit } from '../timing';
import { QUICK_CHIPS } from './QuickChips';
import { failureMessageId } from './ReplyBody';

/** The words of a failure, with the HTTP status after them when there was one ("… (404)"). */
function failureText(failure: AssistantFailure, l10n: ReactLocalization): string {
  const words = l10n.getString(failureMessageId(failure.kind));
  return failure.http_status === null ? words : `${words} (${String(failure.http_status)})`;
}

/** What a row of the log shows as the question: a chip's name in words, else the text as typed. */
function questionText(row: AssistantLogRow, l10n: ReactLocalization): string {
  if (row.source !== 'chip') return row.question;
  const chip = QUICK_CHIPS.find((found) => found.chip === row.question);
  return chip === undefined ? row.question : l10n.getString(chip.messageId);
}

function kindMessageId(source: AssistantLogRow['source']): string {
  switch (source) {
    case 'chip':
      return 'assistant-log-kind-quick';
    case 'model':
      return 'assistant-log-kind-model';
    case 'continue':
      return 'assistant-log-kind-pick';
    case 'rerun':
      return 'assistant-log-kind-rerun';
    case 'test':
      return 'assistant-log-kind-test';
  }
}

/** Which of the key's three states the Owner is in: just looking, replacing it, or removing it. */
type KeyMode = 'view' | 'replace' | 'remove';

function KeySection({ status }: { status: AssistantStatus }) {
  const { l10n } = useLocalization();
  const save = useSaveAssistantKey();
  const remove = useRemoveAssistantKey();
  const [mode, setMode] = useState<KeyMode>('view');
  // The pasted key and the password live here, in this state, until they are sent.
  const [key, setKey] = useState('');
  const [password, setPassword] = useState('');
  const asking = status.key_saved ? mode !== 'view' : true;
  const removing = mode === 'remove';
  const error =
    removing && remove.error !== null
      ? uiErrorOf(remove.error)
      : save.error === null
        ? null
        : uiErrorOf(save.error);

  const finish = () => {
    setKey('');
    setPassword('');
    setMode('view');
    save.reset();
    remove.reset();
  };
  const submit = () => {
    if (removing) {
      remove.mutate(
        { password },
        {
          onSuccess: () => {
            finish();
            showToast('success', l10n.getString('settings-saved'));
          },
        },
      );
      return;
    }
    save.mutate(
      { key, password },
      {
        onSuccess: () => {
          finish();
          showToast('success', l10n.getString('settings-saved'));
        },
      },
    );
  };
  const busy = save.isPending || remove.isPending;

  return (
    <div className="flex flex-col gap-3">
      <h3 className="type-label text-ink">
        <Localized id="assistant-settings-key" />
      </h3>
      {status.key_saved ? (
        <div className="flex flex-wrap items-center gap-3">
          <Localized
            id="assistant-settings-key-saved"
            vars={{ last4: status.key_last4 ?? '' }}
            elems={RUNNING_NUMBER}
          >
            <p className="type-body text-ink" />
          </Localized>
          {mode === 'view' ? (
            <>
              <Button
                onClick={() => {
                  setMode('replace');
                }}
              >
                <Localized id="assistant-settings-key-replace" />
              </Button>
              <Button
                variant="danger"
                onClick={() => {
                  setMode('remove');
                }}
              >
                <Localized id="assistant-settings-key-remove" />
              </Button>
            </>
          ) : null}
        </div>
      ) : (
        <Localized id="assistant-settings-key-none">
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
      {asking ? (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            submit();
          }}
          className="flex max-w-dialog flex-col gap-3"
        >
          {error === null ? null : <ErrorNotice error={error} fieldsShown={['key', 'password']} />}
          {removing ? null : (
            <Field
              label={<Localized id="assistant-settings-key-paste" />}
              error={fieldError(error, 'key')}
            >
              {(control) => (
                <PasswordInput
                  {...control}
                  value={key}
                  onChange={setKey}
                  autoComplete="new-password"
                />
              )}
            </Field>
          )}
          <Field
            label={<Localized id="assistant-settings-key-password" />}
            hint={<Localized id="assistant-settings-key-password-help" />}
            error={fieldError(error, 'password')}
          >
            {(control) => <PasswordInput {...control} value={password} onChange={setPassword} />}
          </Field>
          <div className="flex flex-wrap gap-3">
            <Button
              variant={removing ? 'danger' : 'primary'}
              busy={busy}
              onClick={submit}
              disabled={password === '' || (!removing && key.trim() === '')}
            >
              <Localized
                id={
                  removing
                    ? 'assistant-settings-key-confirm-remove'
                    : status.key_saved
                      ? 'assistant-settings-key-confirm-replace'
                      : 'assistant-settings-key-save'
                }
              />
            </Button>
            {status.key_saved ? (
              <Button onClick={finish}>
                <Localized id="assistant-settings-key-cancel" />
              </Button>
            ) : null}
          </div>
        </form>
      ) : null}
    </div>
  );
}

function ModelSection({ status }: { status: AssistantStatus }) {
  const set = useSetAssistant();
  const [model, setModel] = useState(status.model);
  const error = set.error === null ? null : uiErrorOf(set.error);
  return (
    <div className="flex max-w-dialog flex-col gap-3">
      <Field
        label={<Localized id="assistant-settings-model" />}
        hint={<Localized id="assistant-settings-model-help" />}
        error={fieldError(error, 'model')}
      >
        {(control) => <TextInput {...control} value={model} onChange={setModel} lang="en" />}
      </Field>
      <div>
        <Button
          busy={set.isPending}
          disabled={model.trim() === '' || model.trim() === status.model}
          onClick={() => {
            set.mutate({ enabled: status.enabled, model: model.trim() });
          }}
        >
          <Localized id="assistant-settings-model-save" />
        </Button>
      </div>
    </div>
  );
}

/** The connection test needs a saved key, not the on switch (the lead's ruling). */
function TestSection({ keySaved }: { keySaved: boolean }) {
  const { l10n } = useLocalization();
  const test = useTestAssistant();
  const result = test.data;
  const error = test.error === null ? null : uiErrorOf(test.error);
  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-3">
        <Button
          busy={test.isPending}
          disabled={!keySaved}
          onClick={() => {
            test.mutate();
          }}
        >
          <Localized id="assistant-settings-test" />
        </Button>
        <Localized
          id={keySaved ? 'assistant-settings-test-help' : 'assistant-settings-test-needs-key'}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      {test.isPending ? (
        <Localized id="assistant-settings-test-working">
          <p role="status" className="type-body text-ink-muted" />
        </Localized>
      ) : null}
      {result === undefined ? null : result.worked ? (
        <Notice tone="info">
          <Localized
            id="assistant-settings-test-ok"
            vars={{ ms: String(result.millis) }}
            elems={RUNNING_NUMBER}
          >
            <span />
          </Localized>
        </Notice>
      ) : (
        <Notice tone="error">
          {l10n.getString('assistant-settings-test-failed', {
            error: result.failure === null ? '' : failureText(result.failure, l10n),
          })}
        </Notice>
      )}
      {error === null ? null : <ErrorNotice error={error} />}
    </div>
  );
}

/**
 * How long a row took: the whole run with the model's share and the app's own (plan 51 B2). A chip
 * has only the whole run; a row from before the times were kept has none.
 */
function TookCell({ row }: { row: AssistantLogRow }) {
  const { l10n } = useLocalization();
  if (row.total_millis === null) {
    return <Localized id="assistant-log-took-none" />;
  }
  const total = shownDuration(row.total_millis);
  const unitText = (unit: DurationUnit) => l10n.getString(`assistant-unit-${unit}`);
  if (row.model_millis === null || row.tool_millis === null) {
    return (
      <Localized
        id="assistant-log-took-total"
        vars={{ total: total.value, total_unit: unitText(total.unit) }}
        elems={RUNNING_NUMBER}
      >
        <span />
      </Localized>
    );
  }
  const model = shownDuration(row.model_millis);
  const tools = shownDuration(row.tool_millis);
  return (
    <Localized
      id="assistant-log-took"
      vars={{
        total: total.value,
        total_unit: unitText(total.unit),
        model: model.value,
        model_unit: unitText(model.unit),
        tools: tools.value,
        tools_unit: unitText(tools.unit),
      }}
      elems={RUNNING_NUMBER}
    >
      <span />
    </Localized>
  );
}

/** The question log, page by page, newest first; each row can be read again on today's books. */
function LogSection() {
  const { l10n } = useLocalization();
  const log = useAssistantLog();
  const runner = useAssistantRunner();
  if (log.isPending) return <LoadingState />;
  if (log.isError) {
    return (
      <ErrorState
        title={<Localized id="assistant-log-failed" />}
        error={uiErrorOf(log.error)}
        onRetry={() => {
          void log.refetch();
        }}
      />
    );
  }
  const page = log.data?.pages[0];
  const rows = page?.rows ?? [];
  if (rows.length === 0) {
    return (
      <EmptyState
        icon={History}
        title={<Localized id="assistant-log-empty" />}
        help={<Localized id="assistant-log-empty-help" />}
      />
    );
  }
  const columns: TableColumn<AssistantLogRow>[] = [
    {
      id: 'time',
      header: l10n.getString('assistant-log-col-time'),
      kind: 'number',
      size: 'datetime',
      cell: (row) => utcToLocalDisplay(row.asked_at),
    },
    {
      id: 'question',
      header: l10n.getString('assistant-log-col-question'),
      cell: (row) => <bdi dir="auto">{questionText(row, l10n)}</bdi>,
    },
    {
      id: 'kind',
      header: l10n.getString('assistant-log-col-kind'),
      cell: (row) => l10n.getString(kindMessageId(row.source)),
    },
    {
      id: 'outcome',
      header: l10n.getString('assistant-log-col-outcome'),
      cell: (row) => l10n.getString(`assistant-outcome-${row.outcome.replaceAll('_', '-')}`),
    },
    {
      id: 'tools',
      header: l10n.getString('assistant-log-col-tools'),
      cell: (row) =>
        row.tools.length === 0 ? (
          l10n.getString('assistant-log-no-tools')
        ) : (
          <bdi dir="ltr">
            {row.tools
              .map((tool) => (tool.source === null ? tool.tool : `${tool.tool} ${tool.source}`))
              .join(', ')}
          </bdi>
        ),
    },
    {
      id: 'took',
      header: l10n.getString('assistant-log-col-took'),
      cell: (row) => <TookCell row={row} />,
    },
    {
      id: 'counted',
      header: l10n.getString('assistant-log-col-counted'),
      kind: 'centre',
      cell: (row) =>
        l10n.getString(row.counted ? 'assistant-log-counted-yes' : 'assistant-log-counted-no'),
    },
    {
      id: 'again',
      header: l10n.getString('assistant-log-open'),
      kind: 'action',
      cell: (row) =>
        row.can_rerun ? (
          <Button
            accessibleName={l10n.getString('assistant-log-open-row', {
              question: questionText(row, l10n),
            })}
            onClick={() => {
              // The answer shows in the panel, which rises to show it (PRD FR-7.84).
              useAssistantStore.getState().openDock();
              runner.rerun(row.id, questionText(row, l10n));
            }}
          >
            <Localized id="assistant-log-open" />
          </Button>
        ) : null,
    },
  ];
  return (
    <div className="flex flex-col gap-3">
      <Table
        label={l10n.getString('assistant-log-title')}
        columns={columns}
        rows={rows}
        rowKey={(row) => String(row.id)}
      />
      <Pager pager={log.pager} />
    </div>
  );
}

export function AssistantSettings() {
  const { l10n } = useLocalization();
  const status = useAssistantStatus();
  const set = useSetAssistant();
  if (status.isPending) return <LoadingState />;
  if (status.isError) {
    return (
      <ErrorState
        title={<Localized id="assistant-settings-failed" />}
        error={uiErrorOf(status.error)}
        onRetry={() => {
          void status.refetch();
        }}
      />
    );
  }
  const data = status.data;
  const setError = set.error === null ? null : uiErrorOf(set.error);
  return (
    <div className="flex flex-col gap-4">
      <Card
        title={
          <span className="inline-flex items-center gap-2">
            <MessageCircleQuestion aria-hidden="true" className="size-icon-md text-primary" />
            <Localized id="assistant-settings-title" />
          </span>
        }
      >
        <Localized id="assistant-settings-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        {setError === null ? null : <ErrorNotice error={setError} />}
        <div className="flex flex-col gap-1">
          <Checkbox
            checked={data.enabled}
            onChange={(enabled) => {
              set.mutate({ enabled, model: data.model });
            }}
            label={<Localized id="assistant-settings-enabled" />}
          />
          {data.key_saved ? null : (
            <Localized id="assistant-settings-enabled-needs-key">
              <p className="type-body text-ink-muted" />
            </Localized>
          )}
        </div>
        <KeySection status={data} />
        <ModelSection key={data.model} status={data} />
        <TestSection keySaved={data.key_saved} />
        <div className="flex flex-col gap-1">
          <Localized
            id="assistant-settings-count"
            vars={{ used: String(data.count_today), limit: String(data.limit) }}
            elems={RUNNING_NUMBER}
          >
            <p
              aria-label={l10n.getString('assistant-count-label')}
              className="type-label text-ink"
            />
          </Localized>
          <Localized id="assistant-settings-limit-fixed">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
      </Card>
      <Card title={<Localized id="assistant-log-title" />}>
        <Localized id="assistant-log-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        <LogSection />
      </Card>
    </div>
  );
}
