// The print box (PRD FR-12.12, FR-7.36, FR-7.57), opened with F9 or its button on a saved بکری: the
// print language (اردو or English, the app language to start), the documents, one buyer or all
// buyers, the copies, and printer or PDF. The documents ticked here are the list that Ctrl+Enter
// prints after a save, so they are remembered (`settings_set_printing`).
//
// Printing works on Windows only (plan 14 contract 8). Elsewhere the backend answers
// `ERR_PRINT_WINDOWS_ONLY` and the "Preview" shows the same documents in the app, laid out exactly
// as they would print. The preview is always there, because a bill is worth a look before it is
// printed. The language switch of the box changes only the print, never the app (PRD FR-7.36).

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { PrintDoc } from '../../../bindings/PrintDoc';
import type { PrintOutcome } from '../../../bindings/PrintOutcome';
import type { PrintTo } from '../../../bindings/PrintTo';
import type { PrintingSettings } from '../../../bindings/PrintingSettings';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import { DOC_NAME_ID, PRINT_DOCS, stripIsolates } from '../../../shared/print';
import {
  Button,
  Checkbox,
  Code,
  ErrorNotice,
  ErrorState,
  Field,
  FormDialog,
  LoadingState,
  QtyInput,
  SegmentedControl,
  Select,
  fieldError,
  type SegmentedOption,
} from '../../../shared/ui';
import { usePrintDocs } from '../../reports';
import { usePrintingSettings, useSetPrintingSettings } from '../../settings';
import { useBikriDetail } from '../hooks/useBikriReads';
import { buyersOf } from './buyers';
import { PrintPreview } from './PrintPreview';

const LANGUAGE_OPTIONS: readonly SegmentedOption<Language>[] = [
  { value: 'ur', lang: 'ur', label: <Localized id="common-language-ur" /> },
  { value: 'en', lang: 'en', label: <Localized id="common-language-en" /> },
];

const TO_OPTIONS: readonly SegmentedOption<PrintTo>[] = [
  { value: 'printer', label: <Localized id="print-box-to-printer" /> },
  { value: 'pdf', label: <Localized id="print-box-to-pdf" /> },
];

/** The Select cannot hold an empty value, so "all buyers" is this value in the list. */
const ALL_BUYERS = 'all';

export interface PrintTarget {
  /** The saved بکری. */
  id: number;
  /** Its number, for the title. */
  number: number;
}

interface PrintBoxProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: PrintTarget;
}

/** "Seller bill and Buyer slips": the printed documents, joined the way the language joins a list. */
function useDocNames(): (docs: readonly PrintDoc[], language: Language) => string {
  const { l10n } = useLocalization();
  return (docs, language) =>
    new Intl.ListFormat(language, { style: 'long', type: 'conjunction' }).format(
      docs.map((doc) => stripIsolates(l10n.getString(DOC_NAME_ID[doc]))),
    );
}

function PrintBoxForm({
  target,
  saved,
  onClose,
}: {
  target: PrintTarget;
  saved: PrintingSettings;
  onClose: () => void;
}) {
  const { l10n } = useLocalization();
  const appLanguage = useLanguage();
  const detail = useBikriDetail(target.id);
  const saveSettings = useSetPrintingSettings();
  const print = usePrintDocs();
  const docNames = useDocNames();

  const [language, setLanguage] = useState<Language>(appLanguage);
  // Until the settings arrive the list is unknown; the first render of the form waits for them.
  const [chosen, setChosen] = useState<PrintDoc[] | null>(null);
  const [onlyBuyer, setOnlyBuyer] = useState<string>(ALL_BUYERS);
  const [copies, setCopies] = useState('');
  const [to, setTo] = useState<PrintTo>('printer');
  const [view, setView] = useState<'form' | 'preview'>('form');
  const [outcome, setOutcome] = useState<PrintOutcome | null>(null);
  const [error, setError] = useState<UiError | null>(null);
  const flight = useSingleFlight();

  const docs = chosen ?? saved.print_after_save;
  const buyers = detail.data === undefined ? [] : buyersOf(detail.data);
  const buyerId = onlyBuyer === ALL_BUYERS ? null : Number(onlyBuyer);
  const onlyBuyerId = docs.includes('buyer_slips') ? buyerId : null;

  const toggle = (doc: PrintDoc, on: boolean) => {
    setChosen(
      PRINT_DOCS.filter((candidate) => (candidate === doc ? on : docs.includes(candidate))),
    );
    setOutcome(null);
  };

  /** Remembers the ticked documents as the "print after save" list (PRD FR-7.57). */
  const remember = async (): Promise<void> => {
    if (docs.join() === saved.print_after_save.join()) return;
    await saveSettings.mutateAsync({
      paper_seller_bill: saved.paper_seller_bill,
      paper_buyer_slips: saved.paper_buyer_slips,
      paper_pukhta_slip: saved.paper_pukhta_slip,
      printer: saved.printer,
      copies: String(saved.copies),
      urdu_digits: saved.urdu_digits,
      print_after_save: docs,
    });
  };

  const run = (): Promise<boolean> =>
    flight.run(async () => {
      setError(null);
      setOutcome(null);
      try {
        await remember();
        const result = await print.mutateAsync({
          bikri_id: target.id,
          docs,
          language,
          copies: copies.trim() === '' ? null : copies,
          to,
          only_buyer_id: onlyBuyerId,
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
        <PrintPreview
          bikriId={target.id}
          docs={docs}
          language={language}
          onlyBuyerId={onlyBuyerId}
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
      <Localized id="print-box-help" vars={{ key: SHORTCUTS.saveAndNext.label }}>
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={['copies']} />}
      {outcome === null ? null : (
        <div role="status" className="type-body flex flex-col gap-1 text-money-in">
          {outcome.printed.length === 0 ? (
            <Localized id="print-box-nothing-printed" />
          ) : outcome.pdf_paths === null ? (
            <Localized
              id="print-box-done-printed"
              vars={{ docs: docNames(outcome.printed, language) }}
            />
          ) : (
            <>
              <Localized id="print-box-done-pdf" />
              {outcome.pdf_paths.map((path) => (
                <Code key={path}>{path}</Code>
              ))}
            </>
          )}
        </div>
      )}
      {/* Two columns, so the whole box fits a 1024 x 600 window with little scrolling. */}
      <div className="grid grid-cols-2 gap-4">
        <Field label={<Localized id="print-box-language" />}>
          {() => (
            <div>
              <SegmentedControl
                options={LANGUAGE_OPTIONS}
                value={language}
                groupLabel={l10n.getString('print-box-language')}
                onChange={(next) => {
                  setLanguage(next);
                  setOutcome(null);
                }}
              />
            </div>
          )}
        </Field>
        <Field label={<Localized id="print-box-to" />}>
          {() => (
            <div>
              <SegmentedControl
                options={TO_OPTIONS}
                value={to}
                groupLabel={l10n.getString('print-box-to')}
                onChange={setTo}
              />
            </div>
          )}
        </Field>
      </div>
      <fieldset className="flex flex-col">
        <Localized id="print-box-docs">
          <legend className="type-label" />
        </Localized>
        <div className="flex flex-wrap gap-x-6">
          {PRINT_DOCS.map((doc) => (
            <Checkbox
              key={doc}
              checked={docs.includes(doc)}
              onChange={(on) => {
                toggle(doc, on);
              }}
              label={<Localized id={DOC_NAME_ID[doc]} />}
            />
          ))}
        </div>
        {docs.length === 0 ? (
          <Localized id="print-box-none-ticked">
            <p role="alert" className="type-body text-due" />
          </Localized>
        ) : null}
      </fieldset>
      <div className="grid grid-cols-2 gap-4">
        {docs.includes('buyer_slips') && buyers.length > 1 ? (
          <Field label={<Localized id="print-box-only-buyer" />}>
            {(control) => (
              <Select<string>
                {...control}
                value={onlyBuyer}
                onChange={setOnlyBuyer}
                options={[
                  { value: ALL_BUYERS, label: <Localized id="print-box-all-buyers" /> },
                  ...buyers.map((buyer) => ({
                    value: String(buyer.id),
                    label: displayName(buyer, appLanguage),
                  })),
                ]}
              />
            )}
          </Field>
        ) : null}
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
      </div>
      <div className="flex flex-wrap justify-end gap-2">
        <Button onClick={onClose}>
          <Localized id="print-box-close" />
        </Button>
        <Button
          disabled={docs.length === 0}
          onClick={() => {
            setView('preview');
          }}
        >
          <Localized id="print-box-preview" />
        </Button>
        <Button
          variant="primary"
          busy={flight.busy || print.isPending}
          disabled={docs.length === 0}
          onClick={submitNow}
        >
          <Localized id={to === 'pdf' ? 'print-box-save-pdf' : 'print-box-print'} />
        </Button>
      </div>
    </form>
  );
}

/** Reads the Printing settings (the documents ticked last time), then shows the box. */
function PrintBoxBody({ target, onClose }: { target: PrintTarget; onClose: () => void }) {
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

export function PrintBox({ open, onOpenChange, target }: PrintBoxProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="print-box-title" vars={{ number: String(target.number) }} />}
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
