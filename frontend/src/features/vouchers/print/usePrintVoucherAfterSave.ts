// `Ctrl+Enter` on the voucher screen: save, print the receipt slip if it is ticked in "print after
// save", and start a new voucher (PRD FR-7.57). A print that fails never undoes the save: the
// voucher is already in the books, so the failure is a message, and the slip can be printed again
// with F9. The slip is printed in the app language; the print box is the place to choose another
// one (PRD FR-7.36). The tick comes from the Printing settings of the user.

import { useLocalization } from '@fluent/react';
import { useQueryClient } from '@tanstack/react-query';
import { FALLBACK_ERROR_MESSAGE_ID, uiErrorOf } from '../../../shared/api';
import { useLatest } from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { showToast, useErrorText } from '../../../shared/ui';
import { reportsPrintVoucher } from '../../reports';
import { PRINTING_KEY, settingsGetPrinting } from '../../settings';

/** Returns the function that prints the slip of a voucher that was just saved, if it is ticked. */
export function usePrintVoucherAfterSave(): (voucherId: number) => void {
  const queryClient = useQueryClient();
  const language = useLanguage();
  const errorText = useErrorText();
  const { l10n } = useLocalization();
  const latest = useLatest({ language, errorText, l10n });
  return (voucherId) => {
    void (async () => {
      const { language: printLanguage, errorText: say, l10n: words } = latest();
      try {
        const settings = await queryClient.query({
          queryKey: PRINTING_KEY,
          queryFn: settingsGetPrinting,
          staleTime: Infinity,
        });
        if (!settings.print_after_save.includes('receipt_slip')) return;
        const outcome = await reportsPrintVoucher({
          voucher_id: voucherId,
          language: printLanguage,
          copies: null,
          to: 'printer',
        });
        // Nothing to print: nothing to say.
        if (outcome.printed.length === 0) return;
        showToast('success', words.getString('print-toast-printed'));
      } catch (failure: unknown) {
        const first = uiErrorOf(failure).errors[0];
        showToast(
          'error',
          first === undefined ? words.getString(FALLBACK_ERROR_MESSAGE_ID) : say(first),
        );
      }
    })();
  };
}
