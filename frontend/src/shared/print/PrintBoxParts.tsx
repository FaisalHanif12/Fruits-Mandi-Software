// The parts every print box has (PRD FR-7.36, plans 14 and 15): the print language and where the
// print goes, the notice that says what was printed, and the names of the printed documents. The
// sale book's box and the voucher's box are built from these, so the two look and work alike
// (Code Standards F12). The language switch of a box changes only the print, never the app.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import type { Language } from '../../bindings/Language';
import type { PrintDoc } from '../../bindings/PrintDoc';
import type { PrintOutcome } from '../../bindings/PrintOutcome';
import type { PrintTo } from '../../bindings/PrintTo';
import { Code } from '../ui/Code';
import { Field } from '../ui/Field';
import { SegmentedControl, type SegmentedOption } from '../ui/SegmentedControl';
import { DOC_NAME_ID } from './docs';
import { stripIsolates } from './footer';

const LANGUAGE_OPTIONS: readonly SegmentedOption<Language>[] = [
  { value: 'ur', lang: 'ur', label: <Localized id="common-language-ur" /> },
  { value: 'en', lang: 'en', label: <Localized id="common-language-en" /> },
];

const TO_OPTIONS: readonly SegmentedOption<PrintTo>[] = [
  { value: 'printer', label: <Localized id="print-box-to-printer" /> },
  { value: 'pdf', label: <Localized id="print-box-to-pdf" /> },
];

interface PrintLanguageToProps {
  language: Language;
  onLanguage: (language: Language) => void;
  to: PrintTo;
  onTo: (to: PrintTo) => void;
}

/** Two columns, so the whole box fits a 1024 x 600 window with little scrolling. */
export function PrintLanguageTo({ language, onLanguage, to, onTo }: PrintLanguageToProps) {
  const { l10n } = useLocalization();
  return (
    <div className="grid grid-cols-2 gap-4">
      <Field label={<Localized id="print-box-language" />}>
        {() => (
          <div>
            <SegmentedControl
              options={LANGUAGE_OPTIONS}
              value={language}
              groupLabel={l10n.getString('print-box-language')}
              onChange={onLanguage}
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
              onChange={onTo}
            />
          </div>
        )}
      </Field>
    </div>
  );
}

/** "Seller bill and Buyer slips": the printed documents, joined the way the language joins a list. */
export function useDocNames(): (docs: readonly PrintDoc[], language: Language) => string {
  const { l10n } = useLocalization();
  return (docs, language) =>
    new Intl.ListFormat(language, { style: 'long', type: 'conjunction' }).format(
      docs.map((doc) => stripIsolates(l10n.getString(DOC_NAME_ID[doc]))),
    );
}

interface PrintOutcomeNoticeProps {
  outcome: PrintOutcome;
  language: Language;
}

/** What the last print did: nothing to print, sent to the printer, or the PDF files written. */
export function PrintOutcomeNotice({ outcome, language }: PrintOutcomeNoticeProps): ReactNode {
  const docNames = useDocNames();
  return (
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
  );
}
