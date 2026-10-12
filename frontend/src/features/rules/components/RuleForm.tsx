// The form of one commission rule (plan 09, PRD 11.1, 11.2): side, charge (only the charges the
// side has), party, item or item group, method (only the methods the charge allows), the value as
// text with its unit, the dates, active and a note. The screen offers only what PRD 11.2 allows;
// the backend refuses anything else and is the final word.
//
// - Everything is text in this screen's state, so a language switch loses nothing (F39). The value
//   and the dates go as typed; the backend parses them and says what is wrong (M8).
// - After a save that overlaps another rule at the same level, the form says so and stays until the
//   user presses Done: the newer rule wins and the older one is not changed (FR-11.5).
// - Keys: F10 or Ctrl+S saves, Enter on the last field saves, Esc goes back (asking first).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { TriangleAlert } from 'lucide-react';
import { useEffect, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { uiErrorOf } from '../../../shared/api';
import { useFocusFirstField } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  Card,
  Checkbox,
  ConfirmDialog,
  DateInput,
  ErrorNotice,
  Field,
  fieldError,
  FreeTextInput,
  Picker,
  Select,
  showToast,
  TextInput,
} from '../../../shared/ui';
import { chargesOf, methodsOf, SIDES } from '../allowed';
import { newRuleDraft, ruleDraftOf, sameRuleDraft, withCharge, withSide } from '../draft';
import { useSaveRule, useSetRuleActive } from '../hooks';
import type { ItemGroup, RuleDraft, RuleOverlap, RuleRow, RuleSaved } from '../types';
import { OverlapItem } from './OverlapItem';

interface RuleFormProps {
  /** The rule being edited, or null for a new one. */
  rule: RuleRow | null;
  /** Today as `YYYY-MM-DD`, the start date of a new rule. */
  today: string;
}

const GROUPS: readonly ItemGroup[] = ['fruit', 'vegetable'];
const NO_GROUP = 'none';
const FIELD_PATHS = [
  'side',
  'charge_type',
  'account_id',
  'item_id',
  'item_group',
  'method',
  'value',
  'from_date',
  'to_date',
  'note',
];

export function RuleForm({ rule, today }: RuleFormProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const isNew = rule === null;
  const [initial] = useState(() => (rule === null ? newRuleDraft(today) : ruleDraftOf(rule)));
  const [values, setValues] = useState<RuleDraft>(initial);
  const [baseline, setBaseline] = useState<RuleDraft>(initial);
  const [leaving, setLeaving] = useState(false);
  const [overlaps, setOverlaps] = useState<RuleOverlap[]>([]);
  const [confirmInactive, setConfirmInactive] = useState(false);
  const save = useSaveRule();
  const setActive = useSetRuleActive();
  const containerRef = useFocusFirstField<HTMLFormElement>();
  const error =
    save.error !== null
      ? uiErrorOf(save.error)
      : setActive.error !== null
        ? uiErrorOf(setActive.error)
        : null;
  const dirty = !leaving && overlaps.length === 0 && !sameRuleDraft(values, baseline);

  useEffect(() => {
    if (leaving) void navigate({ to: '/index/rules' });
  }, [leaving, navigate]);

  // The saved rule and its overlap warnings, or null when the save was refused (the form shows why).
  const persist = async (): Promise<RuleSaved | null> => {
    try {
      const saved = await save.mutateAsync(values);
      setBaseline(ruleDraftOf(saved.row));
      showToast('success', l10n.getString('rules-saved'));
      // An overlap is told before the user leaves (FR-11.5).
      setOverlaps(saved.overlaps);
      return saved;
    } catch {
      return null;
    }
  };
  const finish = () => {
    setLeaving(true);
  };
  const saveAndClose = () => {
    void persist().then((saved) => {
      if (saved !== null && saved.overlaps.length === 0) finish();
    });
  };
  // With an overlap on screen, Enter and F10 mean "done": the rule is already saved.
  const submit = overlaps.length > 0 ? finish : saveAndClose;
  const navigation = useFormNavigation(submit);
  useShortcut(SHORTCUTS.save, submit);
  const goBack = () => {
    void navigate({ to: '/index/rules' });
  };
  useEscape('leave', goBack);

  const changeActive = (isActive: boolean) => {
    if (rule === null) return;
    setActive.mutate(
      { id: rule.id, isActive },
      {
        onSuccess: (saved) => {
          showToast(
            'success',
            l10n.getString(isActive ? 'rules-made-active' : 'rules-made-inactive'),
          );
          // Switching a rule on can overlap another rule: the warning shows until Done (FR-11.5).
          if (saved.overlaps.length > 0) setOverlaps(saved.overlaps);
          else setLeaving(true);
        },
      },
    );
  };

  const change = (patch: Partial<RuleDraft>) => {
    setValues((current) => ({ ...current, ...patch }));
  };
  const pickerValue = (
    hit: { id: number; code: string; name_ur: string; name_en: string } | null,
  ) =>
    hit === null
      ? null
      : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en };

  const methods = methodsOf(values.side, values.charge_type);

  return (
    <div className="flex max-w-form flex-col gap-5">
      <Localized id={isNew ? 'rules-new-title' : 'rules-edit-title'}>
        <h1 className="type-title" />
      </Localized>
      <form
        ref={containerRef}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-4"
      >
        <LeaveGuard dirty={dirty} onSave={async () => (await persist()) !== null} />
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
        {overlaps.length === 0 ? null : (
          <section
            role="status"
            className="flex flex-col gap-2 rounded-2 border border-warn-border bg-warn-soft p-3 text-warn"
          >
            <h2 className="flex items-center gap-2 type-body font-semibold">
              <TriangleAlert aria-hidden="true" className="size-icon-md shrink-0" />
              <Localized id="rules-overlap-title" />
            </h2>
            <Localized id="rules-overlap-help">
              <p className="type-body" />
            </Localized>
            <ul className="flex flex-col gap-1 type-body text-ink">
              {overlaps.map((overlap) => (
                <li key={overlap.id}>
                  <OverlapItem overlap={overlap} />
                </li>
              ))}
            </ul>
          </section>
        )}
        <Card>
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
            <Field label={<Localized id="rules-field-side" />} error={fieldError(error, 'side')}>
              {(props) => (
                <Select
                  {...props}
                  value={values.side}
                  onChange={(side) => {
                    setValues((current) => withSide(current, side));
                  }}
                  options={SIDES.map((option) => ({
                    value: option,
                    label: <Localized id={`rules-side-${option}`} />,
                  }))}
                />
              )}
            </Field>
            <Field
              label={<Localized id="rules-field-charge" />}
              error={fieldError(error, 'charge_type')}
            >
              {(props) => (
                <Select
                  {...props}
                  value={values.charge_type}
                  onChange={(charge) => {
                    setValues((current) => withCharge(current, charge));
                  }}
                  options={chargesOf(values.side).map((option) => ({
                    value: option,
                    label: <Localized id={`rules-charge-${option}`} />,
                  }))}
                />
              )}
            </Field>
            <Field
              label={<Localized id="rules-field-party" />}
              hint={<Localized id="rules-field-party-hint" />}
              error={fieldError(error, 'account_id')}
            >
              {(props) => (
                <AccountPicker
                  {...props}
                  scope={values.side}
                  value={values.account}
                  showCard={false}
                  onChange={(hit) => {
                    // A party is more specific than a group: choosing one clears the group.
                    change({
                      account: pickerValue(hit),
                      ...(hit === null ? {} : { item_group: '' }),
                    });
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="rules-field-item" />}
              hint={<Localized id="rules-field-item-hint" />}
              error={fieldError(error, 'item_id')}
            >
              {(props) => (
                <Picker
                  {...props}
                  scope="item"
                  value={values.item}
                  onChange={(hit) => {
                    // An item is more specific than a group: choosing one clears the group.
                    change({ item: pickerValue(hit), ...(hit === null ? {} : { item_group: '' }) });
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="rules-field-group" />}
              hint={<Localized id="rules-field-group-hint" />}
              error={fieldError(error, 'item_group')}
              readOnly={values.item !== null || values.account !== null}
            >
              {(props) => (
                <Select
                  {...props}
                  value={values.item_group === '' ? NO_GROUP : values.item_group}
                  onChange={(next) => {
                    change({ item_group: next === NO_GROUP ? '' : next });
                  }}
                  options={[
                    { value: NO_GROUP, label: <Localized id="rules-group-none" /> },
                    ...GROUPS.map((group) => ({
                      value: group,
                      label: <Localized id={`rules-group-${group}`} />,
                    })),
                  ]}
                />
              )}
            </Field>
          </div>
        </Card>
        <Card>
          <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
            <Field
              label={<Localized id="rules-field-method" />}
              error={fieldError(error, 'method')}
            >
              {(props) => (
                <Select
                  {...props}
                  value={values.method}
                  onChange={(method) => {
                    change({ method });
                  }}
                  options={methods.map((option) => ({
                    value: option,
                    label: <Localized id={`rules-method-${option}`} />,
                  }))}
                />
              )}
            </Field>
            <Field
              label={<Localized id="rules-field-value" />}
              hint={<Localized id={`rules-value-hint-${values.method}`} />}
              error={fieldError(error, 'value')}
            >
              {(props) => (
                <TextInput
                  {...props}
                  value={values.value}
                  onChange={(value) => {
                    change({ value });
                  }}
                  lang="en"
                />
              )}
            </Field>
            <Field
              label={<Localized id="rules-field-from" />}
              error={fieldError(error, 'from_date')}
            >
              {(props) => (
                <DateInput
                  {...props}
                  value={values.from_date}
                  onChange={(from_date) => {
                    change({ from_date });
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="rules-field-to" />}
              hint={<Localized id="rules-field-to-hint" />}
              error={fieldError(error, 'to_date')}
            >
              {(props) => (
                <DateInput
                  {...props}
                  value={values.to_date}
                  onChange={(to_date) => {
                    change({ to_date });
                  }}
                />
              )}
            </Field>
          </div>
          <Field label={<Localized id="rules-field-note" />} error={fieldError(error, 'note')}>
            {(props) => (
              <FreeTextInput
                {...props}
                value={values.note}
                onChange={(note) => {
                  change({ note });
                }}
              />
            )}
          </Field>
          {isNew ? null : (
            <Checkbox
              checked={values.is_active}
              onChange={(is_active) => {
                change({ is_active });
              }}
              label={<Localized id="rules-field-active" />}
            />
          )}
        </Card>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap gap-2">
            {rule === null ? null : rule.is_active ? (
              <Button
                variant="danger"
                busy={setActive.isPending}
                onClick={() => {
                  setConfirmInactive(true);
                }}
              >
                <Localized id="rules-make-inactive" />
              </Button>
            ) : (
              <Button
                busy={setActive.isPending}
                onClick={() => {
                  changeActive(true);
                }}
              >
                <Localized id="rules-make-active" />
              </Button>
            )}
          </div>
          <div className="flex flex-wrap gap-2">
            <Button shortcut={SHORTCUTS.back} onClick={goBack}>
              <Localized id="common-back" />
            </Button>
            {overlaps.length > 0 ? (
              <Button variant="primary" onClick={finish}>
                <Localized id="rules-overlap-done" />
              </Button>
            ) : (
              <Button
                variant="primary"
                shortcut={SHORTCUTS.save}
                busy={save.isPending}
                onClick={saveAndClose}
              >
                <Localized id="layout-save-save" />
              </Button>
            )}
          </div>
        </div>
      </form>
      {rule === null ? null : (
        <ConfirmDialog
          open={confirmInactive}
          onOpenChange={setConfirmInactive}
          title={<Localized id="rules-inactive-title" />}
          description={<Localized id="rules-inactive-description" />}
          confirmLabel={<Localized id="rules-make-inactive" />}
          tone="danger"
          onConfirm={() => {
            changeActive(false);
          }}
        />
      )}
    </div>
  );
}
