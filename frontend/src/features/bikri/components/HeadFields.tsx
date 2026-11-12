// The head of the sale book (PRD 12.2, v1.34 H1, H4; plan 71): the date and the بکری number, in the
// top row of the page beside the name of the screen and its buttons. Nothing else is in the header
// any more: the truck, the زمیندار, the اڈا and the fields behind "More" are gone, and their space
// goes to the lots and the sale lines.
//
// - The date is the first field. Enter on it goes on to the first lot's seller (`onDone`), through
//   the same form keys as every form (shared/keyboard, F27).
// - A new بکری has no number yet: it is given when it is saved and never reused (FR-12.2), so the
//   number shows the word for "new". A posting open for edit shows its number, and its date is
//   kept (PRD FR-7.70, FR-12.19), so that field is read only.
// - The bill date is the entry date (H3), so it is not asked. Old saved بکری keep their truck, اڈا
//   and the rest in the database; a posting open for edit sends them back as they came, unseen
//   and unchanged (`headerInput`), and the saved view shows them when they are there.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { useFormNavigation } from '../../../shared/keyboard';
import { Code, DateInput, Field, fieldError } from '../../../shared/ui';
import { useBikriDraft } from '../draft-store';

interface HeadFieldsProps {
  /** The problems of the whole draft, so the date shows the words of its own. */
  error: UiError | null;
  /** The number of the saved بکری being edited, or null on a new one (it has none yet). */
  number: number | null;
  /** A posting open for edit keeps its day (PRD FR-12.19). */
  editing?: boolean;
  /** Enter on the date: the screen moves on to the lots. */
  onDone: () => void;
}

export function HeadFields({ error, number, editing = false, onDone }: HeadFieldsProps) {
  const stores = useBikriDraft();
  const entryDate = stores.header((state) => state.entryDate);
  const navigation = useFormNavigation(onDone);
  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-wrap items-center gap-x-4 gap-y-1"
    >
      <Field
        inline
        label={<Localized id="bikri-header-date" />}
        error={fieldError(error, 'header.entry_date')}
        readOnly={editing}
      >
        {(control) => (
          <div className="w-date shrink-0">
            <DateInput
              {...control}
              value={entryDate}
              onChange={(next) => {
                stores.header.getState().patch({ entryDate: next });
              }}
            />
          </div>
        )}
      </Field>
      <span className="inline-flex items-baseline gap-2 type-body">
        <Localized id="bikri-head-number">
          <span className="type-label text-ink" />
        </Localized>
        {number === null ? (
          <Localized id="bikri-head-number-new">
            <span className="text-ink-muted" />
          </Localized>
        ) : (
          <Code>{String(number)}</Code>
        )}
      </span>
    </form>
  );
}
