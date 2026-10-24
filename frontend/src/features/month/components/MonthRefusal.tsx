// What the backend refused about a month, in words (PRD FR-17.6, FR-21.2, NFR-USE-13). The month
// screen has words of its own for five refusals, because the general words of the same codes belong
// to the day screen: "ERR_DAY_NOT_CLOSED" there says "nothing to reopen", here it says the days of
// the month are not all closed yet and where they are closed up to. The others (a month in the wrong
// form, a missing reason, an entry that does not balance) use the general words. A refusal of the
// reason box is shown under that box by the dialog, so it is left out here (`fieldsShown`).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import type { UiError } from '../../../bindings/UiError';
import {
  FALLBACK_ERROR_MESSAGE_ID,
  errorMessageId,
  errorsOutsideFields,
  messageVars,
} from '../../../shared/api';
import { isoToDisplayDate, useGrouping } from '../../../shared/format';
import { Code, LinkButton } from '../../../shared/ui';
import { monthErrorId, monthName } from '../text';

interface MonthRefusalProps {
  error: UiError;
  /** The month the refusal is about, `YYYY-MM`: its name fills the words. */
  month: string;
  /** Field paths shown next to their own boxes. */
  fieldsShown?: readonly string[];
}

export function MonthRefusal({ error, month, fieldsShown = [] }: MonthRefusalProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const navigate = useNavigate();
  const items = errorsOutsideFields(error, fieldsShown);
  const empty = error.errors.length === 0;
  if (items.length === 0 && !empty) return null;
  const name = monthName((id) => l10n.getString(id), month);
  return (
    <div role="alert" className="flex flex-col gap-2 text-due">
      {empty ? (
        <Localized id={FALLBACK_ERROR_MESSAGE_ID}>
          <p className="type-body" />
        </Localized>
      ) : null}
      {items.map((item, index) => {
        const wanted = monthErrorId(item.code) ?? errorMessageId(item.code);
        const id = l10n.getBundle(wanted) === null ? FALLBACK_ERROR_MESSAGE_ID : wanted;
        const vars: Record<string, string> = { ...messageVars(item.values, grouping), month: name };
        // The day the days are closed up to is a stored date: written as DD-MM-YYYY like every date.
        const closedUpTo = item.values['closed_up_to'];
        if (typeof closedUpTo === 'string') vars['closed_up_to'] = isoToDisplayDate(closedUpTo);
        return (
          <div key={`${item.code}-${String(index)}`} className="flex flex-col gap-1">
            <Localized id={id} vars={vars}>
              <p className="type-body" />
            </Localized>
            {item.code === 'ERR_DAY_NOT_CLOSED' ? (
              <LinkButton
                onClick={() => {
                  void navigate({ to: '/day' });
                }}
              >
                <Localized id="month-open-day-close" />
              </LinkButton>
            ) : null}
          </div>
        );
      })}
      {error.ref === '' ? null : (
        <Localized id="err-reference" vars={{ ref: error.ref }} elems={{ code: <Code /> }}>
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
    </div>
  );
}
