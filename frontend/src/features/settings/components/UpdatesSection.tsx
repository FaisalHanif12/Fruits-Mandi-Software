// Updates (PRD NFR-INS-3, NFR-INS-4, NFR-SEC-11): the app looks for a newer version when this screen
// opens, shows what is new, and installs only when the Owner presses yes in the confirm box. Never in
// the middle of work: while there are unsaved entries (`drafts`) the button is off and says why, and
// the backend refuses too (ERR_UPDATE_DURING_WORK). The backend makes a full backup first and refuses
// a package that is not signed by us. On Windows the installer closes the app, so the answer may never
// arrive: the screen says the app will close and carries on if it does.
//
// The confirm box starts on "Back", so a stray Enter never starts an update. `confirmed` goes to the
// backend as true only from that box.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { UpdateCheck } from '../../../bindings/UpdateCheck';
import { uiErrorOf } from '../../../shared/api';
import { utcToLocalDisplay } from '../../../shared/format';
import { useLanguage } from '../../../shared/i18n';
import { Button, Card, ConfirmDialog, ErrorNotice, LoadingState, Notice } from '../../../shared/ui';
import { useInstallUpdate, useUpdateCheck } from '../maintenance-hooks';
import { ReleaseNotes } from './ReleaseNotes';

/** The notes in the app language; the other language, then the plain notes, when it has none. */
function notesFor(data: UpdateCheck, language: Language): { text: string; language: Language } {
  const own = language === 'ur' ? data.notes_ur : data.notes_en;
  const otherLanguage: Language = language === 'ur' ? 'en' : 'ur';
  const other = language === 'ur' ? data.notes_en : data.notes_ur;
  if (own.trim() !== '') return { text: own, language };
  if (other.trim() !== '') return { text: other, language: otherLanguage };
  return { text: data.notes, language };
}

export function UpdatesSection() {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const check = useUpdateCheck();
  const install = useInstallUpdate();
  const [asking, setAsking] = useState(false);
  const installError = install.error === null ? null : uiErrorOf(install.error);
  const data = check.data;

  const lookAgain = () => {
    install.reset();
    void check.refetch();
  };
  const update = (version: string) => {
    install.mutate(
      { expected_version: version, confirmed: true },
      {
        onError: (failure) => {
          // A different version is on offer now: ask again, so the Owner reads what is new first.
          // The update commands answer with codes of their own (WireCode), so they are compared as text.
          const codes: readonly string[] = uiErrorOf(failure).errors.map((item) => item.code);
          if (codes.includes('ERR_UPDATE_CHANGED')) void check.refetch();
        },
      },
    );
  };

  let body;
  if (data === undefined && check.isError) {
    body = (
      <div className="flex flex-col gap-3">
        <ErrorNotice error={uiErrorOf(check.error)} />
        <div>
          <Button onClick={lookAgain}>
            <Localized id="settings-update-check-again" />
          </Button>
        </div>
      </div>
    );
  } else if (data === undefined) {
    body = (
      <div role="status" aria-busy="true">
        <LoadingState rows={2} />
        <Localized id="settings-update-checking">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
    );
  } else {
    const version = data.available ? data.version : null;
    const notes = notesFor(data, language);
    body = (
      <div className="flex flex-col gap-4">
        <Localized id="settings-app-version" vars={{ version: data.current_version }}>
          <p className="type-body" />
        </Localized>
        {version === null ? (
          <Localized id="settings-update-up-to-date">
            <p className="type-control" role="status" />
          </Localized>
        ) : (
          <Card title={<Localized id="settings-update-available" vars={{ version }} />}>
            {data.date === null ? null : (
              <Localized id="settings-update-date" vars={{ date: utcToLocalDisplay(data.date) }}>
                <p className="type-body text-ink-muted" />
              </Localized>
            )}
            <Localized id="settings-update-notes">
              <h3 className="type-control" />
            </Localized>
            {notes.text.trim() === '' ? (
              <Localized id="settings-update-no-notes">
                <p className="type-body text-ink-muted" />
              </Localized>
            ) : (
              <ReleaseNotes text={notes.text} language={notes.language} />
            )}
            {data.drafts > 0 ? (
              <Notice tone="warn">
                <Localized id="settings-update-has-drafts" vars={{ drafts: String(data.drafts) }} />
              </Notice>
            ) : null}
            <div className="flex flex-wrap gap-2">
              <Button
                variant="primary"
                disabled={data.drafts > 0 || install.isPending || install.isSuccess}
                onClick={() => {
                  setAsking(true);
                }}
              >
                <Localized id="settings-update-install" />
              </Button>
            </div>
            <ConfirmDialog
              open={asking}
              onOpenChange={setAsking}
              title={<Localized id="settings-update-confirm-title" vars={{ version }} />}
              description={<Localized id="settings-update-confirm-body" />}
              confirmLabel={<Localized id="settings-update-confirm-yes" />}
              initialFocus="back"
              onConfirm={() => {
                update(version);
              }}
            />
          </Card>
        )}
        <div>
          <Button busy={check.isFetching} disabled={install.isPending} onClick={lookAgain}>
            <Localized id="settings-update-check-again" />
          </Button>
        </div>
      </div>
    );
  }

  return (
    <section className="flex flex-col gap-3" aria-label={l10n.getString('settings-app-title')}>
      <Localized id="settings-app-title">
        <h2 className="type-section" />
      </Localized>
      <Localized id="settings-update-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {install.isPending ? (
        <Notice tone="info">
          <Localized id="settings-update-installing" />
        </Notice>
      ) : null}
      {install.data === undefined ? null : (
        <Notice tone="info">
          <Localized
            id="settings-update-done"
            vars={{ version: install.data.version, file: install.data.backup_file ?? '' }}
          />
        </Notice>
      )}
      {installError === null ? null : <ErrorNotice error={installError} />}
      {body}
    </section>
  );
}
