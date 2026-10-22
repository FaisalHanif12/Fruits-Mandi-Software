// Backup settings (PRD FR-20.2 to FR-20.4, FR-20.6, Section 21 "Backup"): the folder of the local
// copies, the USB copy, the cloud account and the closing time. The cloud keys are secrets: they are
// never shown, only whether they are stored; a new one replaces the old, and "remove" deletes it.
// Everything is text, as typed (CLAUDE.md rule 1); the backend checks all of it and shows every
// problem at its own field. Unsaved changes are guarded when the user leaves (FR-7.9).

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Card,
  Checkbox,
  Code,
  ErrorNotice,
  ErrorState,
  Field,
  LoadingState,
  PasswordInput,
  TextInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import type { UiError } from '../../../bindings/UiError';
import { useBackupSettings, useSaveBackupSettings } from '../hooks';
import type { BackupSettings } from '../types';

/** The fields this form draws, so a problem on any of them shows next to its field. */
const SETTINGS_FIELDS = [
  'local_dir',
  'usb_folder',
  'cloud_endpoint',
  'cloud_bucket',
  'cloud_region',
  'cloud_prefix',
  'closing_time',
  'cloud_access_key_id',
  'cloud_secret_access_key',
];

/** The settings that are typed as text. */
type TextKey =
  | 'local_dir'
  | 'usb_folder'
  | 'cloud_endpoint'
  | 'cloud_bucket'
  | 'cloud_region'
  | 'cloud_prefix'
  | 'closing_time';

/** What the Owner typed for a cloud key, and whether he asked to remove the stored one. */
interface KeyEdit {
  text: string;
  remove: boolean;
}
const NO_EDIT: KeyEdit = { text: '', remove: false };

/** null keeps the stored key, an empty text removes it (the backend's contract). */
function keyToSend(edit: KeyEdit): string | null {
  if (edit.remove) return '';
  return edit.text === '' ? null : edit.text;
}

export function SettingsCard() {
  const { l10n } = useLocalization();
  const view = useBackupSettings();
  const save = useSaveBackupSettings();
  // What the Owner changed, or null while the form shows what is stored. A save drops it again.
  const [edits, setEdits] = useState<BackupSettings | null>(null);
  const [keyId, setKeyId] = useState<KeyEdit>(NO_EDIT);
  const [secret, setSecret] = useState<KeyEdit>(NO_EDIT);
  const saved = view.data?.settings;
  const draft = edits ?? saved;

  const error = save.error === null ? null : uiErrorOf(save.error);
  const flight = useSingleFlight();
  const dirty =
    draft !== undefined &&
    saved !== undefined &&
    (JSON.stringify(draft) !== JSON.stringify(saved) ||
      keyId.text !== '' ||
      keyId.remove ||
      secret.text !== '' ||
      secret.remove);

  const submitOnce = (): Promise<boolean> => {
    if (draft === undefined) return Promise.resolve(false);
    return save
      .mutateAsync({
        settings: draft,
        cloud_access_key_id: keyToSend(keyId),
        cloud_secret_access_key: keyToSend(secret),
      })
      .then(() => {
        setEdits(null);
        setKeyId(NO_EDIT);
        setSecret(NO_EDIT);
        showToast('success', l10n.getString('backup-settings-saved'));
        return true;
      })
      .catch(() => false);
  };
  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const submit = (): Promise<boolean> => flight.run(submitOnce, false);
  const submitNow = () => {
    void submit();
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);

  const change = <K extends keyof BackupSettings>(key: K, value: BackupSettings[K]) => {
    if (draft !== undefined) setEdits({ ...draft, [key]: value });
  };
  const text = (key: TextKey, labelId: string, hintId?: string, lang?: string) =>
    draft === undefined ? null : (
      <Field
        label={<Localized id={labelId} />}
        hint={hintId === undefined ? undefined : <Localized id={hintId} />}
        error={fieldError(error, key)}
      >
        {(control) => (
          <TextInput
            {...control}
            lang={lang}
            value={draft[key]}
            onChange={(next) => {
              change(key, next);
            }}
          />
        )}
      </Field>
    );

  let body;
  if (view.isPending) {
    body = <LoadingState rows={3} />;
  } else if (view.isError || draft === undefined) {
    body = (
      <ErrorState
        title={<Localized id="backup-settings-failed" />}
        error={uiErrorOf(view.error)}
        onRetry={() => {
          void view.refetch();
        }}
      />
    );
  } else {
    const data = view.data;
    body = (
      <form
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex max-w-form flex-col gap-4"
      >
        <LeaveGuard dirty={dirty} onSave={submit} />
        {error === null ? null : <ErrorNotice error={error} fieldsShown={SETTINGS_FIELDS} />}
        <Field
          label={<Localized id="backup-set-local-dir" />}
          hint={
            <Localized
              id="backup-set-local-dir-hint"
              vars={{ folder: data.local_dir_effective }}
              elems={{ code: <Code /> }}
            />
          }
          error={fieldError(error, 'local_dir')}
        >
          {(control) => (
            <TextInput
              {...control}
              lang="en"
              value={draft.local_dir}
              onChange={(next) => {
                change('local_dir', next);
              }}
            />
          )}
        </Field>
        {text('closing_time', 'backup-set-closing-time', 'backup-set-closing-time-hint', 'en')}
        <Checkbox
          checked={draft.usb_enabled}
          onChange={(next) => {
            change('usb_enabled', next);
          }}
          label={<Localized id="backup-set-usb-on" />}
        />
        {draft.usb_enabled
          ? text('usb_folder', 'backup-set-usb-folder', 'backup-set-usb-folder-hint', 'en')
          : null}
        <Checkbox
          checked={draft.cloud_enabled}
          onChange={(next) => {
            change('cloud_enabled', next);
          }}
          label={<Localized id="backup-set-cloud-on" />}
        />
        {draft.cloud_enabled ? (
          <>
            {text('cloud_endpoint', 'backup-set-endpoint', 'backup-set-endpoint-hint', 'en')}
            {text('cloud_bucket', 'backup-set-bucket', undefined, 'en')}
            {text('cloud_region', 'backup-set-region', 'backup-set-region-hint', 'en')}
            {text('cloud_prefix', 'backup-set-prefix', 'backup-set-prefix-hint', 'en')}
            <SecretField
              labelId="backup-set-key-id"
              fieldKey="cloud_access_key_id"
              stored={data.cloud_access_key_set}
              edit={keyId}
              onChange={setKeyId}
              error={error}
              plain
            />
            <SecretField
              labelId="backup-set-secret"
              fieldKey="cloud_secret_access_key"
              stored={data.cloud_secret_set}
              edit={secret}
              onChange={setSecret}
              error={error}
            />
          </>
        ) : null}
        <div>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={flight.busy || save.isPending}
            disabled={!dirty}
            onClick={submitNow}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </form>
    );
  }
  return <Card title={<Localized id="backup-set-title" />}>{body}</Card>;
}

interface SecretFieldProps {
  labelId: string;
  /** The field the backend names when it refuses this key. */
  fieldKey: 'cloud_access_key_id' | 'cloud_secret_access_key';
  /** The backend has this key stored. The key itself is never sent to the screen. */
  stored: boolean;
  edit: KeyEdit;
  onChange: (edit: KeyEdit) => void;
  error: UiError | null;
  /** The key id is not a secret, so it is drawn as plain text. */
  plain?: boolean;
}

function SecretField({
  labelId,
  fieldKey,
  stored,
  edit,
  onChange,
  error,
  plain = false,
}: SecretFieldProps) {
  const hint = edit.remove
    ? 'backup-set-key-will-remove'
    : stored
      ? 'backup-set-key-stored'
      : 'backup-set-key-none';
  return (
    <div className="flex flex-col gap-2">
      <Field
        label={<Localized id={labelId} />}
        hint={<Localized id={hint} />}
        error={fieldError(error, fieldKey)}
      >
        {(control) =>
          plain ? (
            <TextInput
              {...control}
              lang="en"
              value={edit.text}
              onChange={(next) => {
                onChange({ text: next, remove: false });
              }}
            />
          ) : (
            <PasswordInput
              {...control}
              value={edit.text}
              onChange={(next) => {
                onChange({ text: next, remove: false });
              }}
              autoComplete="new-password"
            />
          )
        }
      </Field>
      {stored && !edit.remove ? (
        <div>
          <Button
            variant="danger"
            onClick={() => {
              onChange({ text: '', remove: true });
            }}
          >
            <Localized id="backup-set-key-remove" />
          </Button>
        </div>
      ) : null}
    </div>
  );
}
