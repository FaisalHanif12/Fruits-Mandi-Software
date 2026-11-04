// Step 1 of the restore: where is the copy (PRD FR-20.11). The cloud, with the same fields as
// Settings, Backup, or a USB drive or a file. The two cloud keys are masked as they are typed and
// can never be shown again (NFR-SEC-7). The backend looks for the copies; if it refuses, each
// message shows next to the field it concerns (PRD FR-7.6), and nothing is changed on this computer.

import { Localized, useLocalization } from '@fluent/react';
import { FolderOpen, Search } from 'lucide-react';
import { useState } from 'react';
import { chooseOpenPath } from '../../../shared/api';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import { useFocusFirstField } from '../../../shared/hooks';
import {
  Button,
  Card,
  ErrorNotice,
  Field,
  FormGrid,
  FormSpan,
  PasswordInput,
  SegmentedControl,
  TextInput,
  fieldError,
  type SegmentedOption,
} from '../../../shared/ui';
import { CLOUD_PATHS, FILE_PATH, FOLDER_PATH, WHERE_PATHS } from '../field-paths';
import { findCopies } from '../flow';
import { useRestoreStore } from '../store';
import type { CloudFields, RestoreWhere } from '../types';

/** The cloud fields that cannot be left empty. The folder and the region have a usual value. */
const REQUIRED_CLOUD: readonly (keyof CloudFields)[] = ['endpoint', 'bucket', 'key_id', 'secret'];

const CHOICES: readonly RestoreWhere[] = ['cloud', 'folder', 'file'];

/** The cloud address must be https (the backend says the same). A hint here saves a round trip. */
const HTTPS_START = /^https:\/\//i;

export function WhereStep() {
  const { l10n } = useLocalization();
  const where = useRestoreStore((state) => state.where);
  const cloud = useRestoreStore((state) => state.cloud);
  const path = useRestoreStore((state) => state.path);
  const error = useRestoreStore((state) => state.error);
  const busy = useRestoreStore((state) => state.busy);
  const setWhere = useRestoreStore((state) => state.setWhere);
  const setCloudField = useRestoreStore((state) => state.setCloudField);
  const setPath = useRestoreStore((state) => state.setPath);
  const setChoice = useRestoreStore((state) => state.setChoice);
  // Has he tried to go on? Only then do empty fields show a hint (the wizard does the same).
  const [tried, setTried] = useState(false);
  const fields = useFocusFirstField<HTMLDivElement>(where);

  const endpointBad = cloud.endpoint.trim() !== '' && !HTTPS_START.test(cloud.endpoint.trim());
  const missing =
    where === 'cloud'
      ? REQUIRED_CLOUD.some((field) => cloud[field].trim() === '') || endpointBad
      : path.trim() === '';
  const find = () => {
    if (busy) return;
    if (missing) {
      setTried(true);
      return;
    }
    void findCopies();
  };
  const navigation = useFormNavigation(find);
  const empty = (text: string) =>
    tried && text.trim() === '' ? <Localized id="err-required" /> : undefined;

  const options: readonly SegmentedOption<RestoreWhere>[] = CHOICES.map((value) => ({
    value,
    label: <Localized id={`restore-where-${value}`} />,
  }));

  const cloudField = (
    field: keyof CloudFields,
    labelId: string,
    options: { hintId?: string; secret?: boolean } = {},
  ) => (
    <Field
      label={<Localized id={labelId} />}
      hint={options.hintId === undefined ? undefined : <Localized id={options.hintId} />}
      error={
        fieldError(error, CLOUD_PATHS[field]) ??
        (REQUIRED_CLOUD.includes(field) ? empty(cloud[field]) : undefined) ??
        (field === 'endpoint' && tried && endpointBad ? (
          <Localized id="restore-where-endpoint-https" />
        ) : undefined)
      }
    >
      {(control) =>
        options.secret === true ? (
          <PasswordInput
            {...control}
            value={cloud[field]}
            onChange={(text) => {
              setCloudField(field, text);
            }}
            autoComplete="new-password"
            canReveal={false}
          />
        ) : (
          <TextInput
            {...control}
            lang="en"
            dir="ltr"
            value={cloud[field]}
            onChange={(text) => {
              setCloudField(field, text);
            }}
          />
        )
      }
    </Field>
  );

  const choose = (directory: boolean) => {
    void chooseOpenPath({
      directory,
      extension: 'mbk',
      filterName: l10n.getString('restore-where-file-type'),
    }).then(
      (chosen) => {
        if (chosen !== null) setPath(chosen);
      },
      // The box could not open: he can still type the path.
      () => undefined,
    );
  };

  return (
    <Card>
      <div ref={fields} {...navigation} className="flex flex-col gap-4">
        <div className="flex flex-col gap-1">
          <Localized id="restore-where-title">
            <h2 className="type-section" />
          </Localized>
          <Localized id="restore-where-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        <div>
          <SegmentedControl<RestoreWhere>
            options={options}
            value={where}
            onChange={setWhere}
            groupLabel={l10n.getString('restore-where-group')}
            disabled={busy}
          />
        </div>
        {error === null ? null : <ErrorNotice error={error} fieldsShown={WHERE_PATHS} />}
        {where === 'cloud' ? (
          <FormGrid columns={3}>
            <FormSpan span="all">
              {cloudField('endpoint', 'backup-set-endpoint', {
                hintId: 'backup-set-endpoint-hint',
              })}
            </FormSpan>
            {cloudField('bucket', 'backup-set-bucket')}
            {cloudField('prefix', 'backup-set-prefix', { hintId: 'backup-set-prefix-hint' })}
            {cloudField('key_id', 'backup-set-key-id', { secret: true })}
            {cloudField('secret', 'backup-set-secret', { secret: true })}
            {cloudField('region', 'backup-set-region', { hintId: 'backup-set-region-hint' })}
          </FormGrid>
        ) : (
          <div className="flex flex-col gap-3">
            <Field
              label={<Localized id={`restore-where-${where}-path`} />}
              hint={<Localized id={`restore-where-${where}-path-hint`} />}
              error={fieldError(error, where === 'folder' ? FOLDER_PATH : FILE_PATH) ?? empty(path)}
            >
              {(control) => (
                <TextInput {...control} lang="en" dir="ltr" value={path} onChange={setPath} />
              )}
            </Field>
            <div>
              <Button
                onClick={() => {
                  choose(where === 'folder');
                }}
              >
                <FolderOpen aria-hidden="true" className="size-icon-md" />
                <Localized id={`restore-where-choose-${where}`} />
              </Button>
            </div>
          </div>
        )}
        {busy ? (
          <Localized id="restore-where-finding">
            <p role="status" className="type-body text-ink-muted" />
          </Localized>
        ) : null}
        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            shortcut={SHORTCUTS.back}
            onClick={() => {
              setChoice('start');
            }}
          >
            <Localized id="common-back" />
          </Button>
          <Button variant="primary" shortcut={SHORTCUTS.confirm} busy={busy} onClick={find}>
            <Search aria-hidden="true" className="size-icon-md" />
            <Localized id="restore-where-find" />
          </Button>
        </div>
      </div>
    </Card>
  );
}
