// The print box of the collection list (اگراہی فہرست, PRD FR-15.6, FR-7.36), opened from the
// collection screen: the print language (the app language to start), where it goes (printer or PDF)
// and the copies. The day and the filters are the screen's own, so the paper is exactly the list the
// user sees. The paper size comes from the Printing settings.
//
// Printing works on Windows only (plan 14 contract 8). Elsewhere the backend answers
// `ERR_PRINT_WINDOWS_ONLY` and the "Preview" shows the same list in the app, laid out exactly as it
// would print. The language switch of the box changes only the print, never the app.

import { Localized } from '@fluent/react';
import { useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { PrintOutcome } from '../../../bindings/PrintOutcome';
import type { PrintTo } from '../../../bindings/PrintTo';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { useFormNavigation } from '../../../shared/keyboard';
import { PrintLanguageTo, PrintOutcomeNotice } from '../../../shared/print';
import { Button, ErrorNotice, Field, FormDialog, QtyInput, fieldError } from '../../../shared/ui';
import { usePrintCollection } from '../../reports';
import { usePrintingSettings } from '../../settings';
import { CollectionPrintView } from './CollectionPrintView';

/** The list to print: the day and the filters of the collection screen. */
export interface CollectionPrintTarget {
  /** The day as the screen typed it (`DD-MM-YYYY`). */
  date: string;
  collectorStaffId: number | null;
  cityId: number | null;
}

interface CollectionPrintBoxProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: CollectionPrintTarget;
}

function PrintBoxForm({ target, onClose }: { target: CollectionPrintTarget; onClose: () => void }) {
  const appLanguage = useLanguage();
  const print = usePrintCollection();
  // Only the copies hint comes from the settings; the box works while they load.
  const settings = usePrintingSettings();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [copies, setCopies] = useState('');
  const [to, setTo] = useState<PrintTo>('printer');
  const [view, setView] = useState<'form' | 'preview'>('form');
  const [outcome, setOutcome] = useState<PrintOutcome | null>(null);
  const [error, setError] = useState<UiError | null>(null);
  const flight = useSingleFlight();

  const run = (): Promise<boolean> =>
    flight.run(async () => {
      setError(null);
      setOutcome(null);
      try {
        const result = await print.mutateAsync({
          date: target.date,
          collector_staff_id: target.collectorStaffId,
          city_id: target.cityId,
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
        <CollectionPrintView
          key={language}
          mode="preview"
          input={{
            date: target.date,
            collector_staff_id: target.collectorStaffId,
            city_id: target.cityId,
            language,
          }}
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
      <Localized id="vouchers-collection-print-help" vars={{ date: target.date }}>
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
      <Field
        label={<Localized id="print-box-copies" />}
        hint={<Localized id="print-box-copies-hint" />}
        error={fieldError(error, 'copies')}
      >
        {(control) => (
          <QtyInput
            {...control}
            value={copies}
            placeholder={settings.data === undefined ? '' : String(settings.data.copies)}
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

export function CollectionPrintBox({ open, onOpenChange, target }: CollectionPrintBoxProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="vouchers-collection-print-title" />}
    >
      {/* The body is made when the box opens, so every opening starts clean. */}
      {open ? (
        <PrintBoxForm
          target={target}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
