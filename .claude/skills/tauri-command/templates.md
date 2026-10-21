# Templates for a Tauri command

The example is `vouchers_save_receipt` (PRD 15.2). Change the names, keep the shape. Check exact library calls against the versions pinned in `Cargo.lock` and `pnpm-lock.yaml`.

Shared helpers such as `FieldErrors`, `UiError::single`, `toApiError` and `applyApiErrors` are written once, in build steps 2 and 4, and then reused. If the code already has a helper with a different name, use the code's name and keep the shape.

## 1. Input and output types (Rust)

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What the وصولی screen sends. Everything typed is text. PRD 15.1, 15.2.
/// All money is روکڑ (PRD v1.6), so there is no mode, bank or cheque field.
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReceiptInput {
    pub entry_date: String,          // "10-10-2026", parsed by core::validate::parse_date
    pub account_id: AccountId,       // picked from AccountPicker
    pub pukhta_person_key: Option<String>,
    pub amount: String,              // "10,000", parsed by core::money::parse_sum
    pub collector_staff_id: Option<StaffId>,
    pub narration: Option<String>,
}

/// What the screen gets back after the save.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReceiptSaved {
    pub voucher_no: String,
    #[ts(type = "number")]
    pub new_balance_paisa: Paisa,    // shown with formatPaisa, never calculated on screen
    pub balance_side: BalanceSide,   // Dr or Cr
    pub settled_bills: Vec<SettledBill>,
}
```

## 2. Parse step (layer 2 validation)

```rust
impl ReceiptDraft {
    /// Parses and checks every field. Returns all errors together. Code Standards 4.2, 4.3.
    pub fn parse(input: ReceiptInput, today: NaiveDate, closed_up_to: NaiveDate) -> Result<Self, UiError> {
        let mut errors = FieldErrors::default();

        let entry_date = errors.take("entryDate", validate::parse_posting_date(&input.entry_date, today, closed_up_to));
        let amount = errors.take("amount", validate::parse_sum(&input.amount).and_then(validate::more_than_zero));
        let narration = errors.take("narration", validate::free_text(input.narration.as_deref()));

        errors.finish()?; // returns UiError { errors: [...] } if any field failed
        Ok(Self { /* every field, now typed and checked */ })
    }
}
```

## 3. Command (thin)

```rust
/// Saves one وصولی. PRD FR-15.4, FR-6.2.
#[tauri::command]
pub async fn vouchers_save_receipt(
    state: State<'_, AppState>,
    input: ReceiptInput,
) -> Result<ReceiptSaved, UiError> {
    let db = state.db.clone();
    let user = state.require_login()?; // first line: sign-in check (NFR-SEC-4)
    tauri::async_runtime::spawn_blocking(move || {
        let day = services::days::open_range(&db)?;
        let draft = ReceiptDraft::parse(input, day.today, day.closed_up_to)?;
        services::vouchers::save_receipt(&db, &user, draft) // one transaction
    })
    .await
    .map_err(UiError::internal)?
}
```

## 4. Stub for the contract (task T1)

```rust
#[tauri::command]
pub async fn vouchers_save_receipt(
    state: State<'_, AppState>,
    _input: ReceiptInput,
) -> Result<ReceiptSaved, UiError> {
    let _user = state.require_login()?;
    Err(UiError::single(ErrorCode::ErrNotImplemented, None))
}
```

## 5. Registration

`backend/app/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ...existing commands, grouped by module, alphabetical inside a group
    commands::vouchers::vouchers_save_receipt,
])
```

`backend/app/build.rs`: add the command name to the app command list given to the Tauri build, so a permission is generated for it. Then add that permission to `backend/app/capabilities/main.json`. Use the exact permission name from the generated schema under `backend/app/gen/schemas/`.

## 6. Tests (Rust)

```rust
#[test]
fn receipt_rejects_decimal_amount() {
    let err = ReceiptDraft::parse(input_with_amount("850.50"), today(), closed()).unwrap_err();
    assert_eq!(err.first_code_for("amount"), Some(ErrorCode::ErrDecimalsNotAllowed));
}

#[test]
fn receipt_settles_oldest_bill_first() {
    // PRD 4.6: اکرم owes 12,880, pays 10,000, still owes 2,880.
    let db = test_db_with_events(&[Event::Opening, Event::Bikri1]);
    let saved = services::vouchers::save_receipt(&db, &owner(), receipt(AKRAM, rupees(10_000))).unwrap();
    assert_eq!(saved.new_balance_paisa, rupees(2_880));
    assert_trial_balance_balanced(&db);
    assert_audit_written(&db, "voucher", saved.voucher_id);
}

#[test]
fn receipt_refused_before_sign_in() {
    let result = command_signed_out(|s| vouchers_save_receipt(s, valid_receipt()));
    assert_eq!(result.unwrap_err().first_code(), Some(ErrorCode::ErrNotLoggedIn));
    assert_no_rows_written();
}
```

## 7. Frontend wrapper and hook (TypeScript)

`frontend/src/features/vouchers/api.ts`:

```ts
import { invoke } from '@tauri-apps/api/core';
import type { ReceiptInput } from '@/bindings/ReceiptInput';
import type { ReceiptSaved } from '@/bindings/ReceiptSaved';
import { toApiError } from '@/shared/api/errors';

export async function saveReceipt(input: ReceiptInput): Promise<ReceiptSaved> {
  try {
    return await invoke<ReceiptSaved>('vouchers_save_receipt', { input });
  } catch (raw) {
    throw toApiError(raw); // ApiError { errors: { code, field, values }[] }
  }
}
```

`frontend/src/features/vouchers/hooks/useSaveReceipt.ts`:

```ts
export function useSaveReceipt() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: saveReceipt,
    onSuccess: (_saved, input) => {
      // PRD 6.2: what a وصولی updates
      void queryClient.invalidateQueries({ queryKey: ['ledger', input.accountId] });
      void queryClient.invalidateQueries({ queryKey: ['cashbook'] });
      void queryClient.invalidateQueries({ queryKey: ['openBills', input.accountId] });
      void queryClient.invalidateQueries({ queryKey: ['agrahi'] });
      void queryClient.invalidateQueries({ queryKey: ['dashboard'] });
    },
  });
}
```

The form shows each error under its field with the shared helper, for example `applyApiErrors(form, error)`, which reads the message from `errors.ftl`.
