// Adds a city or an اڈا from the account form without leaving it (PRD FR-7.64, FR-10.8): `F4` on
// the city or اڈا picker opens this small form over the screen, with what was typed as the name.
// What the user already typed in the account form stays as it is, because the form is only
// covered. The saved record goes into the picker it was opened from.
//
// The names go through NamePair and the spelling check before the save (FR-7.66). Keys: F10
// saves, Esc closes.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight, useSpellingFlow } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  CodeInput,
  ErrorNotice,
  Field,
  FormDialog,
  NamePair,
  Picker,
  fieldError,
  showToast,
  type NamePairValue,
  type PickerValue,
} from '../../../shared/ui';
import { useNextCode, useSaveMaster } from '../hooks';
import type { MasterRecord } from '../types';

export type QuickMasterKind = 'city' | 'adda';

interface QuickAddMasterProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  kind: QuickMasterKind;
  /** What was typed in the picker. It starts the name. */
  typed: string;
  /** The saved record, to put in the picker. */
  onSaved: (record: PickerValue) => void;
}

const URDU_LETTER = /[؀-ۿ]/;
const FIELD_PATHS = ['code', 'name_ur', 'name_en', 'city_id'];

function QuickAddForm({
  kind,
  typed,
  onSaved,
  onClose,
}: Omit<QuickAddMasterProps, 'open' | 'onOpenChange'> & { onClose: () => void }) {
  const { l10n } = useLocalization();
  const text = typed.trim();
  const [code, setCode] = useState<string | null>(null);
  const [names, setNames] = useState<NamePairValue>(() =>
    text === ''
      ? { urdu: '', english: '' }
      : URDU_LETTER.test(text)
        ? { urdu: text, english: '' }
        : { urdu: '', english: text },
  );
  const [city, setCity] = useState<PickerValue | null>(null);
  const save = useSaveMaster(kind);
  const nextCode = useNextCode(kind, true);
  const spelling = useSpellingFlow();
  const error = save.error === null ? null : uiErrorOf(save.error);
  const codeShown = code ?? nextCode.data ?? '';

  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = async () => {
    try {
      const saved = await spelling.guard(names, setNames, (confirmedWords) =>
        save.mutateAsync({
          draft: {
            id: null,
            version: null,
            code: codeShown,
            names,
            text: {},
            refs: kind === 'adda' ? { city } : {},
            lists: {},
          },
          confirmedWords,
        }),
      );
      if (saved === undefined) return;
      showToast('success', l10n.getString('masters-saved'));
      onSaved(pickerValueOf(saved));
      onClose();
    } catch {
      // The refusal is shown in the form by its fields.
    }
  };
  const submitNow = () => {
    void flight.run(submit, undefined);
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);

  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
        <NamePair
          value={names}
          onChange={setNames}
          fillOnOpen
          urduLabel={<Localized id="masters-field-name-ur" />}
          englishLabel={<Localized id="masters-field-name-en" />}
          urduError={fieldError(error, 'name_ur')}
          englishError={fieldError(error, 'name_en')}
        />
        <Field label={<Localized id="masters-field-code" />} error={fieldError(error, 'code')}>
          {(props) => <CodeInput {...props} value={codeShown} onChange={setCode} />}
        </Field>
        {kind === 'adda' ? (
          <Field label={<Localized id="masters-adda-city" />} error={fieldError(error, 'city_id')}>
            {(props) => (
              <Picker
                {...props}
                scope="city"
                value={city}
                onChange={(hit) => {
                  setCity(
                    hit === null
                      ? null
                      : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                  );
                }}
              />
            )}
          </Field>
        ) : null}
      </div>
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || save.isPending}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
      {spelling.dialog}
    </form>
  );
}

function pickerValueOf(record: MasterRecord): PickerValue {
  return {
    id: record.id,
    code: record.code,
    name_ur: record.names?.name_ur ?? '',
    name_en: record.names?.name_en ?? '',
  };
}

export function QuickAddMaster({ open, onOpenChange, ...form }: QuickAddMasterProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id={form.kind === 'city' ? 'masters-city-new' : 'masters-adda-new'} />}
    >
      {open ? (
        <QuickAddForm
          {...form}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
