// The form of one master, for every kind (plan 06, PRD 10.3, FR-7.24): the code (with the next
// one suggested), the name in two scripts, the fields that kind has, and the facts the backend
// works out. One form, configured per kind (configs.ts).
//
// - Everything is text in this screen's state, so a language switch loses nothing (F39).
// - The names go through NamePair; a word made by the spelling rules alone is checked before the
//   save (useSpellingFlow, PRD FR-7.66).
// - Keys: F10 or Ctrl+S saves, Enter on the last field saves, Esc goes back (asking first when
//   there are unsaved changes), ↑ ↓ ← → move between fields (shared/keyboard).
// - A master is never deleted. It is made inactive, after a confirm (PRD E21).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useMemo, useRef, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { pickedOf } from '../../../shared/accounts';
import { uiErrorOf } from '../../../shared/api';
import {
  focusFirstEmptyField,
  useFocusFirstField,
  useSingleFlight,
  useSpellingFlow,
} from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  Checkbox,
  CodeInput,
  ConfirmDialog,
  ErrorNotice,
  Field,
  NamePair,
  Picker,
  SaveChangesPrompt,
  Select,
  TextInput,
  fieldError,
  showToast,
  type PickerValue,
} from '../../../shared/ui';
import type { SearchHit } from '../../../shared/search';
import type { FieldSpec, MasterConfig } from '../configs';
import { labelOf, newValues, sameValues, valuesOf, type FormValues } from '../draft';
import { useSaveMaster, useSetMasterActive } from '../hooks';
import { useMasterFocus } from '../store';
import type { MasterRecord } from '../types';

/**
 * The form opened above another screen by the "add" button of a picker (plan 32, R8). It is the
 * same form; only the way in and out is different. What was typed in the picker starts the name
 * (or the mark text of a مارکہ).
 */
export interface EmbeddedMaster {
  /** What was typed in the picker. */
  text: string;
  /** The form closed: saved (true) or left (false). */
  onDone: (saved: boolean) => void;
}

interface MasterFormProps {
  config: MasterConfig;
  /** The master being edited, or null for a new one. */
  record: MasterRecord | null;
  /** The suggested code for a new master. */
  suggestedCode: string;
  /** Set when the form is opened above another screen instead of being a screen of its own. */
  embedded?: EmbeddedMaster | undefined;
}

const URDU_LETTER = /[؀-ۿ]/;

/** The path the backend names a field by when it refuses it. */
function fieldPath(spec: FieldSpec): string {
  switch (spec.control.type) {
    case 'picker':
      return `${spec.key}_id`;
    case 'account':
      return `${spec.key}_account_id`;
    default:
      return spec.key;
  }
}

function toPickerValue(hit: Pick<SearchHit, 'id' | 'code' | 'name_ur' | 'name_en'>): PickerValue {
  return pickedOf(hit);
}

export function MasterForm({ config, record, suggestedCode, embedded }: MasterFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const navigate = useNavigate();
  const typedText = embedded?.text.trim() ?? '';
  const initial = useMemo(() => {
    if (record !== null) return valuesOf(config, record);
    const fresh = newValues(config, suggestedCode);
    if (typedText === '') return fresh;
    // What was typed in the picker starts the name, or the mark text of a مارکہ, which has no name.
    if (config.names === null) {
      return config.kind === 'marka'
        ? { ...fresh, text: { ...fresh.text, mark_text: typedText } }
        : fresh;
    }
    return {
      ...fresh,
      names: URDU_LETTER.test(typedText)
        ? { urdu: typedText, english: '' }
        : { urdu: '', english: typedText },
    };
  }, [config, record, suggestedCode, typedText]);
  const [values, setValues] = useState<FormValues>(initial);
  // What was last saved (or opened), to know whether anything changed.
  const [baseline, setBaseline] = useState<FormValues>(initial);
  const [leaving, setLeaving] = useState(false);
  const [confirmInactive, setConfirmInactive] = useState(false);
  // The question "save changes?" of Esc above another screen (the route guard cannot ask it there).
  const [askLeave, setAskLeave] = useState(false);
  const save = useSaveMaster(config.kind);
  const setActive = useSetMasterActive(config.kind);
  const remember = useMasterFocus((state) => state.remember);
  const containerRef = useFocusFirstField<HTMLFormElement>();
  const nameRef = useRef<HTMLFormElement>(null);

  const error =
    save.error !== null
      ? uiErrorOf(save.error)
      : setActive.error !== null
        ? uiErrorOf(setActive.error)
        : null;
  const dirty = !leaving && !sameValues(values, baseline);
  const listPath = `/index/${config.path}`;
  const isNew = record === null;

  const focusNames = () => {
    nameRef.current?.querySelector<HTMLElement>('[role="combobox"], input')?.focus();
  };
  const spelling = useSpellingFlow({ onFix: focusNames });
  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();

  // The host's way out, kept in a ref: the form leaves once, whichever object the host passes.
  // Above another screen the cursor goes to the first box still empty (the code is already filled).
  const isEmbedded = embedded !== undefined;
  useEffect(() => {
    if (isEmbedded) focusFirstEmptyField(nameRef.current);
  }, [isEmbedded]);
  const doneRef = useRef(embedded?.onDone);
  useEffect(() => {
    doneRef.current = embedded?.onDone;
  });
  // Go back to the list once a save or a change of state is finished and the form is clean.
  useEffect(() => {
    if (!leaving) return;
    if (isEmbedded) doneRef.current?.(true);
    else void navigate({ to: listPath });
  }, [leaving, isEmbedded, listPath, navigate]);

  const persistOnce = async (): Promise<boolean> => {
    try {
      const saved = await spelling.guard(
        values.names,
        (names) => {
          setValues((current) => ({ ...current, names }));
        },
        (confirmedWords) => save.mutateAsync({ draft: values, confirmedWords }),
      );
      if (saved === undefined) return false;
      remember(config.kind, saved.id);
      setBaseline(valuesOf(config, saved));
      showToast('success', l10n.getString('masters-saved'));
      return true;
    } catch {
      return false;
    }
  };
  const persist = (): Promise<boolean> => flight.run(persistOnce, false);
  const saveAndClose = () => {
    void persist().then((saved) => {
      if (saved) setLeaving(true);
    });
  };
  const navigation = useFormNavigation(saveAndClose);
  useShortcut(SHORTCUTS.save, saveAndClose);

  const goBack = () => {
    if (embedded === undefined) {
      void navigate({ to: listPath });
    } else if (dirty) {
      setAskLeave(true);
    } else {
      embedded.onDone(false);
    }
  };
  useEscape('leave', goBack, !askLeave);

  const changeActive = (isActive: boolean) => {
    if (record === null) return;
    setActive.mutate(
      { id: record.id, isActive },
      {
        onSuccess: (changed) => {
          remember(config.kind, changed.id);
          showToast(
            'success',
            l10n.getString(isActive ? 'masters-made-active' : 'masters-made-inactive'),
          );
          setLeaving(true);
        },
      },
    );
  };

  const setText = (key: string) => (text: string) => {
    setValues((current) => ({ ...current, text: { ...current.text, [key]: text } }));
  };
  const setRef =
    (key: string) => (hit: Pick<SearchHit, 'id' | 'code' | 'name_ur' | 'name_en'> | null) => {
      setValues((current) => ({
        ...current,
        refs: { ...current.refs, [key]: hit === null ? null : toPickerValue(hit) },
      }));
    };

  const fieldsShown = ['code', 'name_ur', 'name_en', ...config.fields.map(fieldPath)];

  const toggleChoice = (key: string, choice: string, on: boolean) => {
    setValues((current) => {
      const ticked = (current.lists[key] ?? []).filter((item) => item !== choice);
      return { ...current, lists: { ...current.lists, [key]: on ? [...ticked, choice] : ticked } };
    });
  };

  const control = (spec: FieldSpec) => {
    const kind = spec.control;
    if (kind.type === 'multi') {
      const ticked = values.lists[spec.key] ?? [];
      return (
        <fieldset key={spec.key} className="flex min-w-0 flex-col gap-1 normal:col-span-2">
          <Localized id={spec.labelId}>
            <legend className="type-label text-ink" />
          </Localized>
          {spec.hintId === undefined ? null : (
            <Localized id={spec.hintId}>
              <p className="type-body text-ink-muted" />
            </Localized>
          )}
          <div className="grid grid-cols-1 gap-x-5 normal:grid-cols-3">
            {kind.options.map((option) => (
              <Checkbox
                key={option.value}
                checked={ticked.includes(option.value)}
                onChange={(on) => {
                  toggleChoice(spec.key, option.value, on);
                }}
                label={<Localized id={option.labelId} />}
              />
            ))}
          </div>
        </fieldset>
      );
    }
    return (
      <Field
        key={spec.key}
        label={<Localized id={spec.labelId} />}
        hint={spec.hintId === undefined ? undefined : <Localized id={spec.hintId} />}
        error={fieldError(error, fieldPath(spec))}
      >
        {(props) => {
          switch (kind.type) {
            case 'text':
              return (
                <TextInput
                  {...props}
                  value={values.text[spec.key] ?? ''}
                  onChange={setText(spec.key)}
                />
              );
            case 'latin':
              return (
                <TextInput
                  {...props}
                  value={values.text[spec.key] ?? ''}
                  onChange={setText(spec.key)}
                  lang="en"
                />
              );
            case 'select':
              return (
                <Select
                  {...props}
                  value={values.text[spec.key]}
                  onChange={setText(spec.key)}
                  options={kind.options.map((option) => ({
                    value: option.value,
                    label: <Localized id={option.labelId} />,
                  }))}
                />
              );
            case 'picker':
              return (
                <Picker
                  {...props}
                  scope={kind.scope}
                  value={values.refs[spec.key] ?? null}
                  onChange={setRef(spec.key)}
                />
              );
            case 'account':
              return (
                <AccountPicker
                  {...props}
                  scope={kind.scope}
                  value={values.refs[spec.key] ?? null}
                  onChange={setRef(spec.key)}
                />
              );
          }
        }}
      </Field>
    );
  };

  const heading = isNew ? (
    <Localized id={config.newId}>
      <h1 className="type-title" />
    </Localized>
  ) : (
    <Localized id="masters-edit-title" vars={{ name: labelOf(record, language) }}>
      <h1 className="type-title" />
    </Localized>
  );

  return (
    <div className="flex max-w-form flex-col gap-5">
      <div className="flex flex-wrap items-center gap-3">
        {embedded === undefined ? heading : null}
        {record !== null && !record.is_active ? (
          <Localized id="masters-status-inactive">
            <span className="type-chip rounded-1 bg-surface-sunken px-2 text-ink-muted" />
          </Localized>
        ) : null}
      </div>
      {record !== null && !record.is_active ? (
        <Localized id="masters-inactive-note">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
      <form
        ref={(element) => {
          containerRef.current = element;
          nameRef.current = element;
        }}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-4"
      >
        {embedded === undefined ? <LeaveGuard dirty={dirty} onSave={persist} /> : null}
        {error === null ? null : <ErrorNotice error={error} fieldsShown={fieldsShown} />}
        {config.hasCode ? (
          <div className="max-w-field">
            <Field
              label={<Localized id="masters-field-code" />}
              hint={isNew ? <Localized id="masters-field-code-hint" /> : undefined}
              error={fieldError(error, 'code')}
            >
              {(props) => (
                <CodeInput
                  {...props}
                  value={values.code}
                  onChange={(code) => {
                    setValues((current) => ({ ...current, code }));
                  }}
                />
              )}
            </Field>
          </div>
        ) : null}
        {config.names === null ? null : (
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
            <NamePair
              value={values.names}
              onChange={(names) => {
                setValues((current) => ({ ...current, names }));
              }}
              urduLabel={<Localized id={config.names.urduLabelId} />}
              englishLabel={<Localized id={config.names.englishLabelId} />}
              urduError={fieldError(error, 'name_ur')}
              englishError={fieldError(error, 'name_en')}
              checkSpelling={config.names.spelling}
              fillOnOpen={embedded !== undefined}
              // What the pair writes on open is the form's starting point, not a change of the user's.
              onFilledOnOpen={(names) => {
                setValues((current) => ({ ...current, names }));
                setBaseline((current) => ({ ...current, names }));
              }}
            />
          </div>
        )}
        {config.fields.length === 0 && config.shown.length === 0 ? null : (
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
            {config.fields.map(control)}
            {config.shown.map((spec) => {
              const text = record?.shown[spec.key] ?? '';
              return (
                <Field
                  key={spec.key}
                  label={<Localized id={spec.labelId} />}
                  hint={text === '' ? <Localized id={spec.emptyHintId} /> : undefined}
                  readOnly
                >
                  {(props) => (
                    <TextInput {...props} value={text} onChange={() => undefined} lang="en" />
                  )}
                </Field>
              );
            })}
          </div>
        )}
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap gap-2">
            {record === null ? null : record.is_active ? (
              <Button
                variant="danger"
                busy={setActive.isPending}
                onClick={() => {
                  setConfirmInactive(true);
                }}
              >
                <Localized id="masters-make-inactive" />
              </Button>
            ) : (
              <Button
                busy={setActive.isPending}
                onClick={() => {
                  changeActive(true);
                }}
              >
                <Localized id="masters-make-active" />
              </Button>
            )}
          </div>
          <div className="flex flex-wrap gap-2">
            <Button shortcut={SHORTCUTS.back} onClick={goBack}>
              <Localized id="common-back" />
            </Button>
            <Button
              variant="primary"
              shortcut={SHORTCUTS.save}
              busy={flight.busy || save.isPending}
              onClick={saveAndClose}
            >
              <Localized id="layout-save-save" />
            </Button>
          </div>
        </div>
      </form>
      {spelling.dialog}
      {embedded === undefined ? null : (
        <SaveChangesPrompt
          open={askLeave}
          onOpenChange={setAskLeave}
          onSave={() => {
            setAskLeave(false);
            saveAndClose();
          }}
          onLeave={() => {
            setAskLeave(false);
            embedded.onDone(false);
          }}
        />
      )}
      {record === null ? null : (
        <ConfirmDialog
          open={confirmInactive}
          onOpenChange={setConfirmInactive}
          title={<Localized id="masters-inactive-title" />}
          description={
            <Localized
              id="masters-inactive-description"
              vars={{ name: labelOf(record, language) }}
            />
          }
          confirmLabel={<Localized id="masters-make-inactive" />}
          tone="danger"
          onConfirm={() => {
            changeActive(false);
          }}
        />
      )}
    </div>
  );
}
