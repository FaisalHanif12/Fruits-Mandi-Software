// The support file (NFR-OBS-4): one button collects the log, the app version and the check results
// into a text file the client can send us. The Owner chooses where it goes in the system's save box;
// closing the box without choosing does nothing. The file has no password, key or balance (the
// backend writes it). A path the backend refuses (ERR_OUT_OF_RANGE on `path`, with no range to show)
// gets its own words here.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { FALLBACK_ERROR_MESSAGE_ID, chooseSavePath, uiErrorOf } from '../../../shared/api';
import { Button, ErrorNotice, Notice } from '../../../shared/ui';
import { useSaveSupportInfo } from '../maintenance-hooks';

const PATH_FIELD = 'path';

/** The file name the save box starts with: mandi-support-20261023.txt (the computer's date). */
export function supportFileName(today: Date): string {
  const two = (part: number) => String(part).padStart(2, '0');
  const stamp = `${String(today.getFullYear())}${two(today.getMonth() + 1)}${two(today.getDate())}`;
  return `mandi-support-${stamp}.txt`;
}

export function SupportSection() {
  const { l10n } = useLocalization();
  const save = useSaveSupportInfo();
  const [choosing, setChoosing] = useState(false);
  const [boxFailed, setBoxFailed] = useState(false);
  const error = save.error === null ? null : uiErrorOf(save.error);
  const pathRefused = error?.errors.some((item) => item.field === PATH_FIELD) ?? false;

  const choose = () => {
    setBoxFailed(false);
    save.reset();
    setChoosing(true);
    chooseSavePath({
      defaultName: supportFileName(new Date()),
      extension: 'txt',
      filterName: l10n.getString('settings-support-file-type'),
    })
      .then((path) => {
        if (path !== null) save.mutate({ path });
      })
      .catch(() => {
        setBoxFailed(true);
      })
      .finally(() => {
        setChoosing(false);
      });
  };

  return (
    <section className="flex flex-col gap-3" aria-label={l10n.getString('settings-support-title')}>
      <Localized id="settings-support-title">
        <h2 className="type-section" />
      </Localized>
      <Localized id="settings-support-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {pathRefused ? (
        <Notice tone="error">
          <Localized id="settings-support-path-refused" />
        </Notice>
      ) : null}
      {error === null ? null : <ErrorNotice error={error} fieldsShown={[PATH_FIELD]} />}
      {boxFailed ? (
        <Notice tone="error">
          <Localized id={FALLBACK_ERROR_MESSAGE_ID} />
        </Notice>
      ) : null}
      {save.data === undefined ? null : (
        <Notice tone="info">
          <Localized
            id="settings-support-saved"
            vars={{ path: save.data.path, lines: String(save.data.log_lines) }}
          />
        </Notice>
      )}
      <div>
        <Button busy={choosing || save.isPending} onClick={choose}>
          <Localized id="settings-support-save" />
        </Button>
      </div>
    </section>
  );
}
