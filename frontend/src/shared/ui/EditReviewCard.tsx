// "What you changed" and "Its effect", before the save (plan 44, PRD FR-7.70): the list of changes
// as sentences (the old value next to the new one) and the effect on the balances of the parties,
// on cash and on the partners. Everything comes from the backend's review, worked out by the same
// function as the save (Code Standards B5). The screen shows it; it works out nothing (CLAUDE.md
// rule 1). When the partner figures are behind the partner code, a box takes their place.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import type { EditReview } from '../../bindings/EditReview';
import type { UiError } from '../../bindings/UiError';
import {
  changeValueText,
  fieldText,
  isWorkedOut,
  placeText,
  type ValueContext,
} from '../edit/value-text';
import type { EditChange } from '../../bindings/EditChange';
import { formatPaisa, useGrouping } from '../format';
import { displayName, useLanguage } from '../i18n';
import { Balance } from './Balance';
import { Card } from './Card';
import { PartnerLockedBox } from './PartnerLockedBox';
import { Num } from './Num';

/** One list of changes as sentences: the old value next to the new one. */
function ChangeList({ changes, ctx }: { changes: readonly EditChange[]; ctx: ValueContext }) {
  return (
    <ul className="flex flex-col gap-1">
      {changes.map((change, index) => (
        <li
          key={`${change.place.kind}-${String(change.place.lot_no)}-${String(change.place.line_no)}-${change.field}-${String(index)}`}
          className="type-body text-ink"
        >
          <Localized
            id="edit-review-change"
            vars={{
              where: placeText(change.place, ctx),
              field: fieldText(change.field, ctx),
              before: changeValueText(change.before, ctx),
              after: changeValueText(change.after, ctx),
            }}
          >
            <span />
          </Localized>
        </li>
      ))}
    </ul>
  );
}

interface EffectItem {
  key: string;
  name: string;
  before: ReactNode;
  after: ReactNode;
}

/**
 * The effect on each account or partner: his name, then what he was and what he is now. It is a list
 * and not a table because it stands in the narrow card at the side of a form, where three columns of
 * figures would be cut off; the figures wrap under each other instead (NFR-USE-15). Each figure
 * keeps its own left-to-right order and its place on the physical right (NFR-USE-19).
 */
function EffectList({ label, items }: { label: string; items: readonly EffectItem[] }) {
  return (
    <ul aria-label={label} className="flex flex-col divide-y divide-border">
      {items.map((item) => (
        <li key={item.key} className="flex flex-col gap-1 py-2">
          <span className="type-body font-semibold text-ink">{item.name}</span>
          <span className="flex flex-wrap items-baseline gap-x-2">
            <Localized id="edit-review-effect-before">
              <span className="type-body text-ink-muted" />
            </Localized>
            <span className="inline-block min-w-col-qty text-end">{item.before}</span>
            <Localized id="edit-review-effect-after">
              <span className="ms-3 type-body text-ink-muted" />
            </Localized>
            <span className="inline-block min-w-col-qty text-end">{item.after}</span>
          </span>
        </li>
      ))}
    </ul>
  );
}

interface EditReviewCardProps {
  /** The backend's review of the edit as it stands, or undefined before the first answer. */
  review: EditReview | undefined;
  /** The review is on its way (the user is still typing). */
  pending?: boolean;
  /** The review could not be made. The save is still possible; the backend checks again. */
  error?: UiError | null;
}

export function EditReviewCard({ review, pending = false, error = null }: EditReviewCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const ctx = { l10n, language, grouping };

  // What the user typed comes first; the figures the app worked out again follow (PRD FR-7.70).
  const typed = review?.changes.filter((change) => !isWorkedOut(change.field)) ?? [];
  const worked = review?.changes.filter((change) => isWorkedOut(change.field)) ?? [];
  const effect = review?.effect;
  const hasEffect =
    effect !== undefined &&
    (effect.parties.length > 0 ||
      effect.cash !== null ||
      effect.partners.length > 0 ||
      effect.partners_locked);

  return (
    <Card compact title={<Localized id="edit-review-title" />}>
      <div className="flex flex-col gap-3" data-edit-review="">
        {error !== null ? (
          <Localized id="edit-review-failed">
            <p className="type-body text-due" role="alert" />
          </Localized>
        ) : review === undefined ? (
          pending ? (
            <Localized id="edit-review-loading">
              <p className="type-body text-ink-muted" />
            </Localized>
          ) : null
        ) : review.changes.length === 0 ? (
          <Localized id="edit-review-none">
            <p className="type-body text-ink-muted" />
          </Localized>
        ) : (
          <>
            <ChangeList changes={typed} ctx={ctx} />
            {worked.length === 0 ? null : (
              <>
                <Localized id="edit-review-worked-title">
                  <h3 className="type-body font-semibold text-ink" />
                </Localized>
                <ChangeList changes={worked} ctx={ctx} />
              </>
            )}
          </>
        )}
        {review !== undefined && review.changes.length > 0 ? (
          <section className="flex flex-col gap-2" data-edit-effect="">
            <Localized id="edit-review-effect-title">
              <h3 className="type-body font-semibold text-ink" />
            </Localized>
            {!hasEffect ? (
              <Localized id="edit-review-effect-none">
                <p className="type-body text-ink-muted" />
              </Localized>
            ) : null}
            {effect !== undefined && effect.parties.length > 0 ? (
              <EffectList
                label={l10n.getString('edit-review-effect-account')}
                items={effect.parties.map((row) => ({
                  key: String(row.account.id),
                  name: displayName(row.account, language),
                  before: <Balance balance={row.before} />,
                  after: <Balance balance={row.after} />,
                }))}
              />
            ) : null}
            {effect?.cash === null || effect === undefined ? null : (
              <Localized
                id="edit-review-effect-cash"
                vars={{
                  before: formatPaisa(effect.cash.before_paisa, { grouping }),
                  after: formatPaisa(effect.cash.after_paisa, { grouping }),
                }}
              >
                <p className="type-body text-ink" />
              </Localized>
            )}
            {effect !== undefined && effect.partners.length > 0 ? (
              <EffectList
                label={l10n.getString('edit-review-effect-partner')}
                items={effect.partners.map((row) => ({
                  key: String(row.partner.id),
                  name: displayName(row.partner, language),
                  before: <Num paisa={row.before_paisa} />,
                  after: <Num paisa={row.after_paisa} />,
                }))}
              />
            ) : null}
            {effect?.partners_locked === true ? <PartnerLockedBox /> : null}
          </section>
        ) : null}
      </div>
    </Card>
  );
}
