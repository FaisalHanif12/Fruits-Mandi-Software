// The partners of a trip (or, later, a garden) with each one's % and where his money comes from
// (PRD FR-12.29): the shares first, then the source of each partner, in that order. One row for
// every active partner, in the order the partners were created. A partner with no share is not
// in it: his row is dimmed and says so, and the source choice waits.
//
// The shares and sources are text and choices in the screen's state (Code Standards F39). The
// running total is the backend's (partners_check_shares): this component adds nothing up
// (CLAUDE.md rule 1). Problems come from the backend by field path `partners[i].share` and
// `partners[i].source`, where `i` is the row's place in the list.
//
// Shared by the new trip form and, in D4R, the new garden form.

import { Localized, useLocalization } from '@fluent/react';
import type { MoneySource } from '../../bindings/MoneySource';
import type { RecordRef } from '../../bindings/RecordRef';
import type { UiError } from '../../bindings/UiError';
import { displayName, otherScriptLang, otherScriptName, useLanguage } from '../i18n';
import { Field } from './Field';
import { fieldError } from './field-error';
import { PercentInput } from './PercentInput';
import { SegmentedControl } from './SegmentedControl';

export interface VentureShareRow {
  partner: RecordRef;
  /** The % as typed, for example "40" or "33.33". Empty or 0 means he is not in it. */
  share: string;
  source: MoneySource;
}

interface PartnerSharesEditorProps {
  rows: readonly VentureShareRow[];
  onShareChange: (index: number, text: string) => void;
  onSourceChange: (index: number, source: MoneySource) => void;
  /** What the backend refused, with the field paths of the rows. */
  error: UiError | null;
}

/** True when the typed share says "not in it": empty, or only zeros. A reading of text, no sum. */
export function isOutOfVenture(share: string): boolean {
  return /^\s*0*([.,]0*)?\s*$/.test(share);
}

export function PartnerSharesEditor({
  rows,
  onShareChange,
  onSourceChange,
  error,
}: PartnerSharesEditorProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  return (
    <ul aria-label={l10n.getString('ventures-shares-label')} className="flex flex-col">
      {rows.map((row, index) => {
        const name = displayName(row.partner, language);
        const out = isOutOfVenture(row.share);
        const shareLabel = l10n.getString('ventures-share-of', { name });
        const sourceLabel = l10n.getString('ventures-source-of', { name });
        return (
          <li
            key={row.partner.id}
            className={`grid grid-cols-3 items-start gap-3 border-t border-border py-3 ${out ? 'text-ink-muted' : 'text-ink'}`}
          >
            <div className="flex min-w-0 flex-col">
              <span className="type-label truncate" lang={language}>
                {name}
              </span>
              <span className="type-body truncate text-ink-muted" lang={otherScriptLang(language)}>
                {otherScriptName(row.partner, language)}
              </span>
            </div>
            <Field
              label={shareLabel}
              hideLabel
              error={fieldError(error, `partners[${String(index)}].share`)}
            >
              {(control) => (
                <PercentInput
                  {...control}
                  value={row.share}
                  label={shareLabel}
                  onChange={(text) => {
                    onShareChange(index, text);
                  }}
                />
              )}
            </Field>
            <Field
              label={sourceLabel}
              hideLabel
              error={fieldError(error, `partners[${String(index)}].source`)}
            >
              {() =>
                out ? (
                  <Localized id="ventures-not-in">
                    <p className="type-body" />
                  </Localized>
                ) : (
                  <div>
                    <SegmentedControl<MoneySource>
                      groupLabel={sourceLabel}
                      value={row.source}
                      onChange={(source) => {
                        onSourceChange(index, source);
                      }}
                      options={[
                        { value: 'shop', label: <Localized id="ventures-source-shop" /> },
                        { value: 'pocket', label: <Localized id="ventures-source-pocket" /> },
                      ]}
                    />
                  </div>
                )
              }
            </Field>
          </li>
        );
      })}
    </ul>
  );
}
