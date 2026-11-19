// Changing the split of goods that are not all sold (PRD FR-12.41): a trip purchase or a garden's
// goods arrived entry. How many of the entry's نگ are for another shop, and the name of that shop.
// The rest are for our shop. One box; the backend checks the part (not more than the entry has,
// not below what is already sold) and answers with the new parts. A refusal stays inside the box
// and keeps what was typed. The words and the save are the caller's, so the same box serves both
// kinds; it knows nothing about money or stock, and works out no quantity: the part left for our
// shop comes from the backend after the save.

import { Localized } from '@fluent/react';
import { useState, type ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { SHORTCUTS } from '../keyboard';
import { Button } from './Button';
import { ErrorNotice } from './ErrorNotice';
import { Field } from './Field';
import { fieldError } from './field-error';
import { FormDialog } from './FormDialog';
import { FreeTextInput } from './FreeTextInput';
import { QtyInput } from './QtyInput';

/** The fields of the box, by the field paths the backend's problems use. */
const SPLIT_FIELDS = ['other_qty', 'shop_name'];

interface SplitDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  description?: ReactNode;
  /** How many of the entry's نگ in all, for the hint under the field. */
  qty: number;
  /** The part for another shop now. The box starts with it, so it is visible and can be changed. */
  currentOtherQty: number;
  /** The name of the other shop now, or null. */
  currentShop: string | null;
  /** The save is running. */
  busy: boolean;
  /** What the backend refused: it points at `other_qty`, `shop_name` or the whole box. */
  error: UiError | null;
  /** The new part and the shop's name, as typed. */
  onSave: (otherQty: string, shopName: string) => void;
}

export function SplitDialog({
  open,
  onOpenChange,
  title,
  description,
  qty,
  currentOtherQty,
  currentShop,
  busy,
  error,
  onSave,
}: SplitDialogProps) {
  // The text typed. Null until the user types: the box then shows what the entry has now.
  const [typedQty, setTypedQty] = useState<string | null>(null);
  const [typedShop, setTypedShop] = useState<string | null>(null);
  const otherQty = typedQty ?? String(currentOtherQty);
  const shop = typedShop ?? currentShop ?? '';
  const close = () => {
    setTypedQty(null);
    setTypedShop(null);
    onOpenChange(false);
  };
  const save = () => {
    onSave(otherQty.trim(), shop.trim());
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
          save();
        }}
      >
        <Field
          label={<Localized id="split-other-qty" />}
          hint={<Localized id="split-other-qty-hint" vars={{ qty: String(qty) }} />}
          error={fieldError(error, 'other_qty')}
        >
          {(control) => (
            <QtyInput
              {...control}
              value={otherQty}
              onChange={(text) => {
                setTypedQty(text);
              }}
            />
          )}
        </Field>
        <Field label={<Localized id="split-shop" />} error={fieldError(error, 'shop_name')}>
          {(control) => (
            <FreeTextInput
              {...control}
              value={shop}
              onChange={(text) => {
                setTypedShop(text);
              }}
            />
          )}
        </Field>
        {error === null ? null : <ErrorNotice error={error} fieldsShown={SPLIT_FIELDS} />}
        <div className="mt-2 flex flex-wrap justify-end gap-2">
          <Button shortcut={SHORTCUTS.back} onClick={close}>
            <Localized id="common-back" />
          </Button>
          <Button variant="primary" busy={busy} onClick={save}>
            <Localized id="common-save" />
          </Button>
        </div>
      </form>
    </FormDialog>
  );
}
