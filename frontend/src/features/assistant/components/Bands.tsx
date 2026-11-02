// The two bands of an answer (PRD FR-7.74, FR-7.86, FR-7.88). An estimate is amber, with the words
// "اندازہ، یقینی نہیں" and, for each method, the months it used and the figures it gave. Advice is calm,
// not a warning colour: "مشورہ، فیصلہ آپ کا", above the answer. Both carry an icon and words, so colour
// is never the only signal (NFR-USE-12). Every figure is the backend's; an estimate has no second
// place, so it never carries ✓ (FR-7.86).

import { Localized, useLocalization } from '@fluent/react';
import { Compass, Lightbulb, TriangleAlert } from 'lucide-react';
import type { AnswerEstimate } from '../../../bindings/AnswerEstimate';
import type { AnswerMonth } from '../../../bindings/AnswerMonth';
import type { EstimateMethod } from '../../../bindings/EstimateMethod';
import type { EstimateMethodKind } from '../../../bindings/EstimateMethodKind';
import { RUNNING_NUMBER } from '../../../shared/ui';
import { useListContext } from '../../analytics';
import { cellText, monthText } from '../../reports';
import { Figure } from './Figure';

/** The band that stands above the answer when it is advice (PRD FR-7.88). */
export function AdviceBand() {
  return (
    <div
      role="note"
      className="flex items-center gap-2 rounded-2 border border-border bg-primary-soft px-3 py-2 text-primary"
    >
      <Lightbulb aria-hidden="true" className="size-icon-md shrink-0" />
      <Localized id="assistant-band-advice">
        <span className="type-label" />
      </Localized>
    </div>
  );
}

/**
 * The band of general advice (PRD FR-7.88, v1.19): the books had too little for the question, so the
 * assistant gives short, practical mandi advice that rests on no figure of the books. It is calm like the
 * advice band, with its own words and icon, so the Owner never takes it for advice from his own books.
 */
export function GeneralAdviceBand() {
  return (
    <div
      role="note"
      data-band="general-advice"
      className="flex items-center gap-2 rounded-2 border border-border bg-primary-soft px-3 py-2 text-primary"
    >
      <Compass aria-hidden="true" className="size-icon-md shrink-0" />
      <div className="flex flex-col">
        <Localized id="assistant-band-general">
          <span className="type-label" />
        </Localized>
        <Localized id="assistant-band-general-note">
          <span className="type-body text-ink-muted" />
        </Localized>
      </div>
    </div>
  );
}

function methodMessageId(kind: EstimateMethodKind): string {
  switch (kind) {
    case 'what_if_pace':
      return 'assistant-estimate-what-if-any';
    case 'same_pace':
      return 'assistant-estimate-same-pace';
    case 'same_growth':
      return 'assistant-estimate-same-growth';
    case 'same_season':
      return 'assistant-estimate-same-season';
  }
}

function MethodView({
  method,
  hasTarget,
}: {
  method: EstimateMethod;
  /** A target was asked for: each method says the month it is reached, or that it is not. */
  hasTarget: boolean;
}) {
  const context = useListContext();
  const text = (month: AnswerMonth) => monthText(context.text, month.year, month.month);
  return (
    <li className="flex flex-col gap-1">
      {method.kind === 'what_if_pace' && method.factor !== null ? (
        // The scenario's words come from the factor the backend sent, never from the model (FR-7.76).
        <Localized
          id="assistant-estimate-what-if"
          vars={{ factor: cellText(method.factor, context) }}
          elems={RUNNING_NUMBER}
        >
          <p className="type-label text-ink" />
        </Localized>
      ) : (
        <p className="type-label text-ink">{context.text(methodMessageId(method.kind))}</p>
      )}
      <Localized
        id="assistant-estimate-months"
        vars={{
          count: String(method.months_used),
          from: text(method.used_from),
          to: text(method.used_to),
        }}
        elems={RUNNING_NUMBER}
      >
        <p className="type-body text-ink-muted" />
      </Localized>
      <p className="flex flex-wrap items-center gap-x-4 gap-y-1 type-body text-ink">
        {method.per_month === null ? null : (
          <span className="inline-flex items-center gap-2">
            {context.text('assistant-estimate-per-month')}
            <Figure cell={method.per_month} dir={null} checked={false} working={null} />
          </span>
        )}
        {method.step === null ? null : (
          <span className="inline-flex items-center gap-2">
            {context.text('assistant-estimate-step')}
            <Figure cell={method.step} dir={method.step_dir} checked={false} working={null} />
          </span>
        )}
        <span className="inline-flex items-center gap-2">
          {context.text('assistant-estimate-total')}
          <Figure cell={method.total} dir={null} checked={false} working={method.working} />
        </span>
      </p>
      {hasTarget ? (
        method.target_month === null ? (
          <p className="type-body text-ink">{context.text('assistant-estimate-target-not')}</p>
        ) : (
          <p className="type-body text-ink">
            {context.text('assistant-estimate-target-reached', {
              month: text(method.target_month),
            })}
          </p>
        )
      ) : null}
    </li>
  );
}

/** The amber band of an estimate (PRD FR-7.86): the words, the months ahead, and each method. */
export function EstimateBand({ estimate }: { estimate: AnswerEstimate | null }) {
  const { l10n } = useLocalization();
  const caption =
    estimate === null || l10n.getBundle(estimate.measure_id) === null
      ? null
      : l10n.getString(estimate.measure_id);
  return (
    <section
      aria-label={l10n.getString('assistant-band-estimate')}
      className="flex flex-col gap-2 rounded-2 border border-warn-border bg-warn-soft p-3"
    >
      <div className="flex items-center gap-2 text-warn">
        <TriangleAlert aria-hidden="true" className="size-icon-md shrink-0" />
        <Localized id="assistant-band-estimate">
          <span className="type-label" />
        </Localized>
      </div>
      {estimate === null ? null : <EstimateDetails estimate={estimate} caption={caption} />}
    </section>
  );
}

function EstimateDetails({
  estimate,
  caption,
}: {
  estimate: AnswerEstimate;
  caption: string | null;
}) {
  const context = useListContext();
  const text = (month: AnswerMonth) => monthText(context.text, month.year, month.month);
  return (
    <>
      {caption === null ? null : <p className="type-body text-ink">{caption}</p>}
      {estimate.target === null ? null : (
        <p className="inline-flex items-center gap-2 type-body text-ink">
          {context.text('assistant-estimate-target')}
          <Figure cell={estimate.target} dir={null} checked={false} working={null} />
        </p>
      )}
      {estimate.methods.length === 0 ? (
        <Localized
          id="assistant-estimate-few-months"
          vars={{ count: String(estimate.full_months) }}
          elems={RUNNING_NUMBER}
        >
          <p className="type-body text-ink" />
        </Localized>
      ) : (
        <>
          <p className="type-body text-ink-muted">
            {context.text('assistant-estimate-ahead', {
              from: text(estimate.first),
              to: text(estimate.last),
            })}
          </p>
          <ul className="flex list-none flex-col gap-3">
            {estimate.methods.map((method) => (
              <MethodView key={method.kind} method={method} hasTarget={estimate.target !== null} />
            ))}
          </ul>
        </>
      )}
    </>
  );
}
