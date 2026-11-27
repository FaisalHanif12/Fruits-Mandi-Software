//! What every cancel asks the Owner for (PRD FR-7.93, v1.61, plan 100): a reason, his sign
//! (دستخط, PRD FR-21.4) and the 4-digit partner code (PRD FR-17.11).
//!
//! A cancel command takes the input it had before, plus the two secrets of [`CancelProof`], in one
//! flat JSON object. The reason is already part of every cancel input. Example, `vouchers_cancel`:
//! `{ "id": 7, "version": 1, "reason": "غلط رقم", "partner_code": "4826", "sign": "adnan" }`.
//!
//! The 10-second undo of a save (PRD FR-7.47) asks for none of them.
//!
//! Pure types. The check is `mandi_services::cancel_guard`.

use serde::Deserialize;
use ts_rs::TS;

use crate::arrivals::{ArrivalCancelInput, ArrivalWriteOffUndoInput};
use crate::bikri::{CancelInput, WriteOffUndoInput};
use crate::fruit::FruitCancelInput;
use crate::gardens::{GardenEntryCancelInput, GardenReopenInput};
use crate::purchases::{PurchaseCancelInput, PurchaseWriteOffUndoInput};
use crate::trips::TripEntryCancelInput;
use crate::vouchers::VoucherCancelInput;

/// The two secrets a cancel carries. Left out of a request, each is empty and refused. Never
/// logged and never saved.
#[derive(Clone, Default, PartialEq, Eq, Deserialize, TS)]
pub struct CancelProof {
    /// The 4-digit partner code. `ERR_REQUIRED` when empty, `ERR_PARTNER_CODE_INVALID` when not 4
    /// digits, `ERR_PARTNER_CODE_NOT_SET` when no code has been set yet, `ERR_PARTNER_CODE_WRONG`
    /// (value `tries_left`) or `ERR_LOCKED_WAIT` (value `seconds`) when it is wrong. All on
    /// `partner_code`.
    #[serde(default)]
    pub partner_code: String,
    /// The Owner's sign. `ERR_SIGN_REQUIRED` when empty, `ERR_SIGN_WRONG` when wrong, on `sign`.
    #[serde(default)]
    pub sign: String,
}

impl std::fmt::Debug for CancelProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CancelProof").finish_non_exhaustive()
    }
}

/// One cancel request: the cancel input and the proof, flat in one JSON object.
macro_rules! guarded {
    ($(#[$doc:meta])* $name:ident, $base:ty) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Deserialize, TS)]
        pub struct $name {
            /// What to cancel, with the reason.
            #[serde(flatten)]
            #[ts(flatten)]
            pub base: $base,
            /// The partner code and the sign.
            #[serde(flatten)]
            #[ts(flatten)]
            pub proof: CancelProof,
        }
    };
}

guarded!(
    /// Input of `bikri_cancel`.
    BikriCancelRequest,
    CancelInput
);
guarded!(
    /// Input of `bikri_write_off_undo`.
    BikriWriteOffUndoRequest,
    WriteOffUndoInput
);
guarded!(
    /// Input of `vouchers_cancel` (every voucher type).
    VoucherCancelRequest,
    VoucherCancelInput
);
guarded!(
    /// Input of `fruit_cancel`.
    FruitCancelRequest,
    FruitCancelInput
);
guarded!(
    /// Input of `purchases_cancel`.
    PurchaseCancelRequest,
    PurchaseCancelInput
);
guarded!(
    /// Input of `purchases_write_off_undo`.
    PurchaseWriteOffUndoRequest,
    PurchaseWriteOffUndoInput
);
guarded!(
    /// Input of `trips_cost_cancel` and `trips_other_sale_cancel`.
    TripEntryCancelRequest,
    TripEntryCancelInput
);
guarded!(
    /// Input of `gardens_entry_cancel`.
    GardenEntryCancelRequest,
    GardenEntryCancelInput
);
guarded!(
    /// Input of `gardens_arrival_cancel`.
    ArrivalCancelRequest,
    ArrivalCancelInput
);
guarded!(
    /// Input of `gardens_reopen` (undo of a garden's closing).
    GardenReopenRequest,
    GardenReopenInput
);
guarded!(
    /// Input of `gardens_arrival_write_off_undo`.
    ArrivalWriteOffUndoRequest,
    ArrivalWriteOffUndoInput
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_is_one_flat_object_and_hides_the_secrets_in_debug_text() {
        let request: VoucherCancelRequest = serde_json::from_value(serde_json::json!({
            "id": 7,
            "version": 1,
            "reason": "غلط رقم",
            "partner_code": "4826",
            "sign": "adnan",
        }))
        .unwrap();
        assert_eq!(request.base.id, 7);
        assert_eq!(request.proof.partner_code, "4826");
        let text = format!("{request:?}");
        assert!(!text.contains("4826") && !text.contains("adnan"), "{text}");
    }

    #[test]
    fn a_request_without_the_secrets_has_them_empty() {
        let request: BikriCancelRequest = serde_json::from_value(serde_json::json!({
            "id": 1, "version": 1, "reason": "x",
        }))
        .unwrap();
        assert_eq!(request.proof, CancelProof::default());
    }
}
