# Plan 89: Editing a وصولی, ادائیگی or پیشگی is clear, and the edit shows afterwards

| | |
|---|---|
| Status | Umar's report, 21-11-2026 (screenshot: وصولی 1 of 16-11-2026 opened for edit with the sign) |
| Branch | `fix/gardens-new-form` |
| PRD | FR-7.70 (amount, account, narration editable; date fixed), FR-7.71 (edit stamp, history, "پہلے → اب"), FR-15.3, FR-21.4 |

## What happened

The edit is allowed by the code (the amount field is not locked in edit mode, `VoucherForm.tsx`), but:
1. The amber band says only "اس کا دن نہیں بدلتا۔ دن بدلنا ہو تو اسے منسوخ کر کے صحیح دن پر دوبارہ لکھیں" (`edit-band-date-fixed`), so it reads as "you cannot change this, cancel it". It does not say what CAN be changed.
2. The party card pushes the amount (رقم) and the reason far below the screen, so the Owner does not see the amount field at all.
3. The Owner wants to see, after the save, the edited amount with the time, the sign and the reason of the change (FR-7.71).

## Fix (frontend; the backend edit already exists)

1. Band wording, ur and en: first line "آپ وصولی 1 (16-11-2026) میں تبدیلی کر رہے ہیں"; second line "رقم، کھاتہ اور تفصیل بدل سکتے ہیں۔ صرف دن نہیں بدلتا؛ دن بدلنا ہو تو منسوخ کر کے صحیح دن پر لکھیں۔" (same for ادائیگی and پیشگی).
2. Layout: in edit mode the amount field and the reason field sit in the first rows of the form, beside the date and account; the party card sits beside or below them, never pushing them out of view at 1366x768 and 1024x600. The cursor starts in the amount.
3. After the save: the entry and its list row show the تبدیل شدہ stamp (time, Owner, the sign when the day was reopened, the reason), the amount shows the new figure with the mark "پہلے: 10,000 → اب: 12,000" on hover or Ctrl+I, and "تبدیلیوں کی تاریخ" lists the versions (FR-7.71). Use what the backend already returns (edit stamp and history from the audit log); if a field is missing, tell the lead.
4. Check the save end to end on the dev bridge and with the real commands: amount changed, reason given, saved, the days close again, the stamp shows. Also ادائیگی and پیشگی.

## Acceptance

- Band says what can be changed; amount and reason visible at once; editing the amount and saving works for وصولی, ادائیگی and پیشگی; the stamp and "پہلے → اب" show after save; ur and en; tests that fail before; typecheck, lint, i18n:check pass; screenshots at 1366x768 in ur and en.
