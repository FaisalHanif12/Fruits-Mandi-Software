// Printing (PRD Section 21, A21): the paper of each document, the printer, how many copies, the
// Urdu digits on Urdu prints, and the documents that Ctrl+Enter prints after a save (PRD FR-7.57).
// Copies are typed text: the backend checks 1 to 99 and names the field `copies` when it is wrong.
// The four states: the settings load, they fail with a retry, the printer list is empty (it is
// only known on Windows, and the default printer is then used) or failed, and the data.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import type { Paper } from '../../../bindings/Paper';
import type { PrintDoc } from '../../../bindings/PrintDoc';
import type { PrintingSettings } from '../../../bindings/PrintingSettings';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import { DOC_NAME_ID, PAPERS, PRINT_DOCS } from '../../../shared/print';
import {
  Button,
  Checkbox,
  ErrorNotice,
  ErrorState,
  Field,
  LinkButton,
  LoadingState,
  QtyInput,
  Select,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { usePrinters } from '../../reports';
import { usePrintingSettings, useSetPrintingSettings } from '../hooks';

/** The Select cannot hold an empty value, so "the default printer" is this value in the list. */
const DEFAULT_PRINTER = '*';

const FIELDS = ['copies', 'printer', 'print_after_save'];

interface FormValues {
  paper_seller_bill: Paper;
  paper_buyer_slips: Paper;
  paper_pukhta_slip: Paper;
  paper_receipt_slip: Paper;
  printer: string;
  copies: string;
  urdu_digits: boolean;
  print_after_save: PrintDoc[];
}

function valuesOf(saved: PrintingSettings): FormValues {
  return {
    paper_seller_bill: saved.paper_seller_bill,
    paper_buyer_slips: saved.paper_buyer_slips,
    paper_pukhta_slip: saved.paper_pukhta_slip,
    paper_receipt_slip: saved.paper_receipt_slip,
    printer: saved.printer,
    copies: String(saved.copies),
    urdu_digits: saved.urdu_digits,
    print_after_save: saved.print_after_save,
  };
}

function sameValues(a: FormValues, b: FormValues): boolean {
  return (
    a.paper_seller_bill === b.paper_seller_bill &&
    a.paper_buyer_slips === b.paper_buyer_slips &&
    a.paper_pukhta_slip === b.paper_pukhta_slip &&
    a.paper_receipt_slip === b.paper_receipt_slip &&
    a.printer === b.printer &&
    a.copies === b.copies &&
    a.urdu_digits === b.urdu_digits &&
    a.print_after_save.join() === b.print_after_save.join()
  );
}

const PAPER_FIELDS = [
  { field: 'paper_seller_bill', doc: 'seller_bill' },
  { field: 'paper_buyer_slips', doc: 'buyer_slips' },
  { field: 'paper_pukhta_slip', doc: 'pukhta_slip' },
  { field: 'paper_receipt_slip', doc: 'receipt_slip' },
] as const;

function PrintingForm({ saved }: { saved: PrintingSettings }) {
  const { l10n } = useLocalization();
  const printers = usePrinters();
  const save = useSetPrintingSettings();
  const savedValues = valuesOf(saved);
  const [values, setValues] = useState<FormValues>(savedValues);
  const setField =
    <K extends keyof FormValues>(field: K) =>
    (value: FormValues[K]) => {
      setValues((current) => ({ ...current, [field]: value }));
    };
  const error = save.error === null ? null : uiErrorOf(save.error);
  const dirty = !sameValues(values, savedValues);

  const submitOnce = (): Promise<boolean> =>
    save
      .mutateAsync(values)
      .then(() => {
        showToast('success', l10n.getString('settings-saved'));
        return true;
      })
      .catch(() => false);
  // One save at a time: a second F10 or Enter while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submit = (): Promise<boolean> => flight.run(submitOnce, false);
  const submitNow = () => {
    void submit();
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);

  // The printer in Settings may be missing from the list (offline now, or no list on this
  // computer). It stays in the list, so opening this tab never changes it by itself.
  const names = printers.data ?? [];
  const known = values.printer === '' || names.includes(values.printer);
  const printerOptions = [
    {
      value: DEFAULT_PRINTER,
      label: <Localized id="print-settings-printer-default" />,
    },
    ...(known ? [] : [{ value: values.printer, label: values.printer }]),
    ...names.map((name) => ({ value: name, label: name })),
  ];

  const toggleDoc = (doc: PrintDoc, on: boolean) => {
    setField('print_after_save')(
      PRINT_DOCS.filter((candidate) =>
        candidate === doc ? on : values.print_after_save.includes(candidate),
      ),
    );
  };

  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex max-w-form flex-col gap-5"
    >
      <LeaveGuard dirty={dirty} onSave={submit} />
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELDS} />}

      <section className="flex flex-col gap-3">
        <Localized id="print-settings-paper">
          <h2 className="type-section" />
        </Localized>
        <Localized id="print-settings-paper-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        {/* Two columns in a narrow window and four in a wide one, so the tab stays short. */}
        <div className="grid grid-cols-2 gap-4 normal:grid-cols-4">
          {PAPER_FIELDS.map(({ field, doc }) => (
            <Field key={field} label={<Localized id={DOC_NAME_ID[doc]} />}>
              {(control) => (
                <Select<Paper>
                  {...control}
                  value={values[field]}
                  onChange={setField(field)}
                  options={PAPERS.map((paper) => ({
                    value: paper,
                    label: <Localized id={`print-paper-${paper}`} />,
                  }))}
                />
              )}
            </Field>
          ))}
        </div>
      </section>

      <section className="grid grid-cols-3 gap-4">
        <div className="col-span-2">
          <Field
            label={<Localized id="print-settings-printer" />}
            error={fieldError(error, 'printer')}
            hint={
              printers.isPending ? (
                <Localized id="common-loading" />
              ) : printers.isError ? (
                <span className="flex flex-wrap items-center gap-2">
                  <Localized id="print-settings-printers-failed" />
                  <LinkButton
                    onClick={() => {
                      void printers.refetch();
                    }}
                  >
                    <Localized id="common-retry" />
                  </LinkButton>
                </span>
              ) : names.length === 0 ? (
                <Localized id="print-settings-printer-none" />
              ) : undefined
            }
          >
            {(control) => (
              <Select<string>
                {...control}
                value={values.printer === '' ? DEFAULT_PRINTER : values.printer}
                onChange={(next) => {
                  setField('printer')(next === DEFAULT_PRINTER ? '' : next);
                }}
                options={printerOptions}
              />
            )}
          </Field>
        </div>
        <Field
          label={<Localized id="print-settings-copies" />}
          hint={<Localized id="print-settings-copies-hint" />}
          error={fieldError(error, 'copies')}
        >
          {(control) => (
            <QtyInput {...control} value={values.copies} onChange={setField('copies')} />
          )}
        </Field>
      </section>

      <section className="flex flex-col gap-1">
        <Checkbox
          checked={values.urdu_digits}
          onChange={setField('urdu_digits')}
          label={<Localized id="print-settings-urdu-digits" />}
        />
        <Localized id="print-settings-urdu-digits-hint">
          <p className="type-body text-ink-muted" />
        </Localized>
      </section>

      <section className="flex flex-col gap-1" aria-labelledby="print-after-save-heading">
        <Localized id="print-settings-after-save">
          <h2 id="print-after-save-heading" className="type-section" />
        </Localized>
        <Localized id="print-settings-after-save-hint" vars={{ key: SHORTCUTS.saveAndNext.label }}>
          <p className="type-body text-ink-muted" />
        </Localized>
        {PRINT_DOCS.map((doc) => (
          <Checkbox
            key={doc}
            checked={values.print_after_save.includes(doc)}
            onChange={(on) => {
              toggleDoc(doc, on);
            }}
            label={<Localized id={DOC_NAME_ID[doc]} />}
          />
        ))}
        {fieldError(error, 'print_after_save')}
      </section>

      <div>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || save.isPending}
          disabled={!dirty}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
    </form>
  );
}

export function PrintingTab() {
  const settings = usePrintingSettings();
  if (settings.isPending) return <LoadingState />;
  if (settings.isError) {
    return (
      <ErrorState
        title={<Localized id="print-settings-load-failed" />}
        error={uiErrorOf(settings.error)}
        onRetry={() => {
          void settings.refetch();
        }}
      />
    );
  }
  // The form starts again from the settings when they change (the answer of a save).
  return <PrintingForm key={JSON.stringify(settings.data)} saved={settings.data} />;
}
