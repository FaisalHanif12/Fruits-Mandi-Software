// The link fields of a separate khata or of a person who is lent money (PRD FR-16.6, FR-16.13, plan 76 points
// 5 and 10): what the money is linked to (nobody, the shop, or one partner) and, for one partner, which. The
// kinds a form offers come from the screen. `locked` shows the link in words and cannot be changed: a link
// is fixed while the account's balance is not 0. A refusal of the backend shows at its field (`link`,
// `link.partner_id`). Nothing here works out any money (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Field, Select, fieldError } from '../../../shared/ui';
import { usePartnerList } from '../../partners';
import { linkKindId, type LinkChoiceKind, type LinkDraft } from '../link-draft';

interface LinkChoiceProps {
  draft: LinkDraft;
  onChange: (next: LinkDraft) => void;
  error: UiError | null;
  /** The kinds offered, in order. */
  kinds: readonly LinkChoiceKind[];
  /** The label of the kind field. */
  labelId: string;
  /** What the link means, under the field. */
  helpId?: string;
  /** The link cannot change now: the fields show it and say why. */
  locked?: boolean;
}

export function LinkChoice({
  draft,
  onChange,
  error,
  kinds,
  labelId,
  helpId,
  locked = false,
}: LinkChoiceProps) {
  const language = useLanguage();
  const partners = usePartnerList();
  return (
    <>
      <Field
        label={<Localized id={labelId} />}
        hint={
          locked ? (
            <Localized id="cashbook-link-locked" />
          ) : helpId === undefined ? undefined : (
            <Localized id={helpId} />
          )
        }
        error={fieldError(error, 'link')}
      >
        {(props) => (
          <Select<LinkChoiceKind>
            {...props}
            readOnly={props.readOnly === true || locked}
            value={draft.kind}
            onChange={(kind) => {
              onChange({ kind, partnerId: kind === 'partner' ? draft.partnerId : '' });
            }}
            options={kinds.map((kind) => ({
              value: kind,
              label: <Localized id={linkKindId(kind)} />,
            }))}
          />
        )}
      </Field>
      {draft.kind === 'partner' ? (
        <PartnerField
          draft={draft}
          onChange={onChange}
          error={error}
          locked={locked}
          partners={partners}
          language={language}
        />
      ) : null}
    </>
  );
}

interface PartnerFieldProps {
  draft: LinkDraft;
  onChange: (next: LinkDraft) => void;
  error: UiError | null;
  locked: boolean;
  partners: ReturnType<typeof usePartnerList>;
  language: ReturnType<typeof useLanguage>;
}

function PartnerField({
  draft,
  onChange,
  error,
  locked,
  partners,
  language,
}: PartnerFieldProps): ReactNode {
  return (
    <Field
      label={<Localized id="cashbook-link-partner" />}
      error={
        fieldError(error, 'link.partner_id') ??
        (partners.isError ? <Localized id="cashbook-partners-failed" /> : undefined)
      }
    >
      {(props) => (
        <Select
          {...props}
          readOnly={props.readOnly === true || locked || partners.isPending}
          value={draft.partnerId === '' ? undefined : draft.partnerId}
          onChange={(partnerId) => {
            onChange({ kind: 'partner', partnerId });
          }}
          options={(partners.data ?? [])
            .filter((partner) => partner.is_active)
            .map((partner) => ({
              value: String(partner.id),
              label: displayName(partner, language),
            }))}
        />
      )}
    </Field>
  );
}
