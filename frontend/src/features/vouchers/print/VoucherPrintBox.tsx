// The print box of a saved voucher (PRD FR-15.2, FR-7.36, FR-7.57), opened with F9 or the "Print"
// button of the voucher screen: the print language (the app language to start), where it goes
// (printer or PDF), and the copies. The receipt slip is the one document. Its tick is the "print
// after save" choice for vouchers that `Ctrl+Enter` follows, so it is remembered
// (`settings_set_printing`), together with the sale book's documents it does not touch.
//
// Printing works on Windows only (plan 14 contract 8). Elsewhere the backend answers
// `ERR_PRINT_WINDOWS_ONLY` and the "Preview" shows the same slip in the app, laid out exactly as it
// would print. The language switch of the box changes only the print, never the app.

import { Localized } from '@fluent/react';
import { useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { PrintOutcome } from '../../../bindings/PrintOutcome';
import type { PrintTo } from '../../../bindings/PrintTo';
import type { PrintingSettings } from '../../../bindings/PrintingSettings';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import { DOC_NAME_ID, PrintLanguageTo, PrintOutcomeNotice } from '../../../shared/print';
import {
  Button,
  Checkbox,
  ErrorNotice,
  ErrorState,
  Field,
  FormDialog,
  LoadingState,
  QtyInput,
  fieldError,
} from '../../../shared/ui';
import { usePrintVoucher } from '../../reports';
import { usePrintingSettings, useSetPrintingSettings } from '../../settings';
import { VoucherPrintView } from './VoucherPrintView';

export interface VoucherPrintTarget {
  /** The saved voucher. */
  id: number;
  /** Its number in its series, as the screen shows it ("R-12"), for the title. */
  numberText: string;
}

interface VoucherPrintBoxProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: VoucherPrintTarget;
}

function PrintBoxForm({
  target,
  saved,
  onClose,
}: {
  target: VoucherPrintTarget;
  saved: PrintingSettings;
  onClose: () => void;
}) {
  const appLanguage = useLanguage();
  const saveSettings = useSetPrintingSettings();
  const print = usePrintVoucher();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [afterSave, setAfterSave] = useState<boolean | null>(null);
  const [copies, setCopies] = useState('');
  const [to, setTo] = useState<PrintTo>('printer');
  const [view, setView] = useState<'form' | 'preview'>('form');
  const [outcome, setOutcome] = useState<PrintOutcome | null>(null);
  const [error, setError] = useState<UiError | null>(null);
  const flight = useSingleFlight();

  const ticked = afterSave ?? saved.print_after_save.includes('receipt_slip');

  /** Remembers the tick as the "print after save" choice; the other documents stay as they were. */
  const remember = async (): Promise<void> => {
    if (ticked === saved.print_after_save.includes('receipt_slip')) return;
    const others = saved.print_after_save.filter((doc) => doc !== 'receipt_slip');
    await saveSettings.mutateAsync({
      paper_seller_bill: saved.paper_seller_bill,
      paper_buyer_slips: saved.paper_buyer_slips,
      paper_pukhta_slip: saved.paper_pukhta_slip,
      paper_receipt_slip: saved.paper_receipt_slip,
      printer: saved.printer,
      copies: String(saved.copies),
      urdu_digits: saved.urdu_digits,
      print_after_save: ticked ? [...others, 'receipt_slip'] : others,
    });
  };

  const run = (): Promise<boolean> =>
    flight.run(async () => {
      setError(null);
      setOutcome(null);
      try {
        await remember();
        const result = await print.mutateAsync({
          voucher_id: target.id,
          language,
          copies: copies.trim() === '' ? null : copies,
          to,
        });
        setOutcome(result);
        return true;
      } catch (failure: unknown) {
        setError(uiErrorOf(failure));
        return false;
      }
    }, false);
  const submitNow = () => {
    void run();
  };
  const navigation = useFormNavigation(submitNow);

  if (view === 'preview') {
    return (
      <div className="flex flex-col gap-3">
        <Localized id="print-preview-title">
          <h3 className="type-section" />
        </Localized>
        <Localized id="print-preview-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        <VoucherPrintView
          key={language}
          mode="preview"
          input={{ voucher_id: target.id, language }}
        />
        <div className="flex flex-wrap justify-end gap-2">
          <Button
            onClick={() => {
              setView('form');
            }}
          >
            <Localized id="common-back" />
          </Button>
        </div>
      </div>
    );
  }

  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      <Localized id="vouchers-print-help" vars={{ key: SHORTCUTS.saveAndNext.label }}>
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={['copies']} />}
      {outcome === null ? null : <PrintOutcomeNotice outcome={outcome} language={language} />}
      <PrintLanguageTo
        language={language}
        onLanguage={(next) => {
          setLanguage(next);
          setOutcome(null);
        }}
        to={to}
        onTo={setTo}
      />
      <fieldset className="flex flex-col">
        <Localized id="print-box-docs">
          <legend className="type-label" />
        </Localized>
        <Checkbox
          checked={ticked}
          onChange={setAfterSave}
          label={<Localized id={DOC_NAME_ID.receipt_slip} />}
        />
      </fieldset>
      <Field
        label={<Localized id="print-box-copies" />}
        hint={<Localized id="print-box-copies-hint" />}
        error={fieldError(error, 'copies')}
      >
        {(control) => (
          <QtyInput
            {...control}
            value={copies}
            placeholder={String(saved.copies)}
            onChange={setCopies}
          />
        )}
      </Field>
      <div className="flex flex-wrap justify-end gap-2">
        <Button onClick={onClose}>
          <Localized id="print-box-close" />
        </Button>
        <Button
          onClick={() => {
            setView('preview');
          }}
        >
          <Localized id="print-box-preview" />
        </Button>
        <Button variant="primary" busy={flight.busy || print.isPending} onClick={submitNow}>
          <Localized id={to === 'pdf' ? 'print-box-save-pdf' : 'print-box-print'} />
        </Button>
      </div>
    </form>
  );
}

/** Reads the Printing settings (the tick from last time), then shows the box. */
function PrintBoxBody({ target, onClose }: { target: VoucherPrintTarget; onClose: () => void }) {
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
  return <PrintBoxForm target={target} saved={settings.data} onClose={onClose} />;
}

export function VoucherPrintBox({ open, onOpenChange, target }: VoucherPrintBoxProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="vouchers-print-title" vars={{ number: target.numberText }} />}
    >
      {/* The body is made when the box opens, so every opening starts from the saved settings. */}
      {open ? (
        <PrintBoxBody
          target={target}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
