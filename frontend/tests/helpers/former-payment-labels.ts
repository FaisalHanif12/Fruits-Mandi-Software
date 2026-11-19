// The words of the fields that were taken out in plan 17b (PRD 15.1, v1.6): how the money moves,
// the bank account and the cheque. All money is روکڑ now, so no screen has any of them. The Fluent
// messages are gone; these are the old words, kept here only so a test can prove that no field
// carries one of them (the labels were "Mode", "Bank account", "Cheque number", "Cheque date").

import type { Language } from '../../src/i18n';

export const FORMER_PAYMENT_LABELS: Record<Language, readonly string[]> = {
  ur: ['ذریعہ', 'بینک کھاتہ', 'چیک نمبر', 'چیک کی تاریخ'],
  en: ['Mode', 'Bank account', 'Cheque number', 'Cheque date'],
};

/** The text of a label as the visible words, for `queryByLabelText` of a former label. */
export const labelIs = (words: string) => (name: string) =>
  name.replace(/[⁦-⁩]/g, '').trim() === words;
