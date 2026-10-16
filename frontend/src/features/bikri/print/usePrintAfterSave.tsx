// `Ctrl+Enter` on the sale book: save, print the documents ticked in "print after save", and open
// a new بکری (PRD FR-7.57). The save hands the saved بکری to `afterSave` (after-save.ts), which
// calls the function registered here. A print that fails never undoes the save: the بکری is
// already in the books, so the failure is a message, and the bill can be printed again with F9.
//
// The documents come from the Printing settings of the user (in the cache the settings screen and
// the print box keep fresh). The language is the app language: the print box is the place to
// choose another one (PRD FR-7.36).

import { useQueryClient } from '@tanstack/react-query';
import { Localized, useLocalization } from '@fluent/react';
import { useEffect } from 'react';
import type { BikriSaved } from '../../../bindings/BikriSaved';
import { FALLBACK_ERROR_MESSAGE_ID, uiErrorOf } from '../../../shared/api';
import { useLatest } from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { showToast, useErrorText } from '../../../shared/ui';
import { reportsPrint } from '../../reports';
import { PRINTING_KEY, settingsGetPrinting } from '../../settings';
import { registerPrintAfterSave } from '../after-save';

/** Registers the Ctrl+Enter print while the sale book is on screen. */
export function usePrintAfterSave(): void {
  const queryClient = useQueryClient();
  const language = useLanguage();
  const errorText = useErrorText();
  const { l10n } = useLocalization();
  const latest = useLatest({ language, errorText, l10n });

  useEffect(
    () =>
      registerPrintAfterSave((saved: BikriSaved) => {
        void (async () => {
          const { language: printLanguage, errorText: say, l10n: words } = latest();
          try {
            const settings = await queryClient.query({
              queryKey: PRINTING_KEY,
              queryFn: settingsGetPrinting,
              staleTime: Infinity,
            });
            if (settings.print_after_save.length === 0) return;
            const outcome = await reportsPrint({
              bikri_id: saved.bikri_id,
              docs: settings.print_after_save,
              language: printLanguage,
              copies: null,
              to: 'printer',
              only_buyer_id: null,
            });
            // Every document had nothing to print: nothing to say.
            if (outcome.printed.length === 0) return;
            showToast(
              'success',
              <Localized id="print-toast-printed">
                <span />
              </Localized>,
            );
          } catch (failure: unknown) {
            const first = uiErrorOf(failure).errors[0];
            showToast(
              'error',
              first === undefined ? words.getString(FALLBACK_ERROR_MESSAGE_ID) : say(first),
            );
          }
        })();
      }),
    [queryClient, latest],
  );
}
