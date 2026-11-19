// Changing the date to sell of goods that are not sold yet (PRD FR-12.39, FR-12.40): a trip purchase
// or a garden's goods arrived entry. One box with one date; the backend checks that the day is not
// before the day the goods came and answers with the new day. A refusal stays inside the box and
// keeps what was typed. The words and the save are the caller's, so the same box serves both
// kinds; it knows nothing about money or stock.

import { Localized } from '@fluent/react';
import { useState, type ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { Button } from './Button';
import { DateInput } from './DateInput';
import { ErrorNotice } from './ErrorNotice';
import { Field } from './Field';
import { fieldError } from './field-error';
import { FormDialog } from './FormDialog';
import { SHORTCUTS } from '../keyboard';

interface SellDateDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  description?: ReactNode;
  /** The day now, as `DD-MM-YYYY`. The box starts with it, so it is visible and can be changed. */
  current: string;
  /** The save is running. */
  busy: boolean;
  /** What the backend refused: it points at the field `sell_date` or the whole box. */
  error: UiError | null;
  /** The new day, as typed. */
  onSave: (sellDate: string) => void;
}

export function SellDateDialog({
  open,
  onOpenChange,
  title,
  description,
  current,
  busy,
  error,
  onSave,
}: SellDateDialogProps) {
  // The day typed. Null until the user types: the box then shows the day the goods have now.
  const [typed, setTyped] = useState<string | null>(null);
  const value = typed ?? current;
  const close = () => {
    setTyped(null);
    onOpenChange(false);
  };
  return (
    <FormDialog
      open={open}
      onOpenChange={(next) => {
        if (!next) close();
        else onOpenChange(true);
      }}
      title={title}
      description={description}
    >
      <form
        className="flex flex-col gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          onSave(value.trim());
        }}
      >
        <Field label={<Localized id="glossary-sell-date" />} error={fieldError(error, 'sell_date')}>
          {(control) => (
            <DateInput
              {...control}
              value={value}
              onChange={(day) => {
                setTyped(day);
              }}
            />
          )}
        </Field>
        {error === null ? null : <ErrorNotice error={error} fieldsShown={['sell_date']} />}
        <div className="mt-2 flex flex-wrap justify-end gap-2">
          <Button shortcut={SHORTCUTS.back} onClick={close}>
            <Localized id="common-back" />
          </Button>
          <Button
            variant="primary"
            busy={busy}
            onClick={() => {
              onSave(value.trim());
            }}
          >
            <Localized id="common-save" />
          </Button>
        </div>
      </form>
    </FormDialog>
  );
}
