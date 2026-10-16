// One partner in the setup list (PRD 17.1): his name, a button for his details, and the box for
// his share as a percent. The share is the text as typed; the backend reads it and adds the
// shares up (CLAUDE.md rule 1). The column headings above the list are the visible labels, so
// the share box carries the partner's name for a screen reader.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import { displayName, otherScriptLang, otherScriptName, useLanguage } from '../../../shared/i18n';
import { Button, Field, PercentInput } from '../../../shared/ui';
import type { PartnerRow } from '../types';

interface ShareRowProps {
  partner: PartnerRow;
  /** The share as text, or null for an inactive partner (he has no share box). */
  share: string | null;
  onShareChange: (text: string) => void;
  /** What the backend said about this row, from fieldError(). */
  error?: ReactNode;
  onDetails: () => void;
}

export function ShareRow({ partner, share, onShareChange, error, onDetails }: ShareRowProps) {
  const language = useLanguage();
  const { l10n } = useLocalization();
  const name = displayName(partner, language);
  const nameForAria = l10n.getString('partners-share-of', { name });
  return (
    <li className="grid grid-cols-3 items-start gap-4 border-t border-border py-3">
      <div className="col-span-2 flex min-w-0 flex-wrap items-center justify-between gap-3">
        <div className="flex min-w-0 flex-col">
          <span className="type-label truncate text-ink" lang={language}>
            {name}
          </span>
          <span className="type-body truncate text-ink-muted" lang={otherScriptLang(language)}>
            {otherScriptName(partner, language)}
          </span>
        </div>
        <div className="flex items-center gap-3">
          {partner.is_active ? null : (
            <Localized id="partners-badge-inactive">
              <span className="type-body rounded-1 bg-surface-sunken px-2 text-ink-muted" />
            </Localized>
          )}
          <Button onClick={onDetails}>
            <Localized id="partners-edit" />
            <span className="sr-only">{l10n.getString('partners-edit-of', { name })}</span>
          </Button>
        </div>
      </div>
      {share === null ? null : (
        <Field label={nameForAria} hideLabel error={error}>
          {(control) => (
            <PercentInput {...control} value={share} onChange={onShareChange} label={nameForAria} />
          )}
        </Field>
      )}
    </li>
  );
}
