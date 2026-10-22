//! Exports the TypeScript types for the frontend with ts-rs (plan 01 Section 4).
//!
//! The root `pnpm gen:types` runs:
//!
//! ```bash
//! cd backend && TS_RS_EXPORT_DIR="$PWD/../frontend/src/bindings" cargo test -p mandi-app export_bindings
//! ```
//!
//! Without `TS_RS_EXPORT_DIR` the files go to a temporary folder, so a plain
//! `cargo test` checks the export works but never changes `frontend/`.
#![cfg(test)]

use std::path::PathBuf;

use mandi_app_lib::commands::app::AppInfo;
use mandi_app_lib::commands::{auth, day, settings, translit};
use mandi_app_lib::error::{ErrorCode, UiError, UiErrorItem};
use mandi_app_lib::state::DataStatus;
use mandi_services::core::money::Paisa;
use mandi_services::core::{
    accounts, bikri, cash, drafts, masters, partners, printing, purchases, rules, search, trips,
    ventures, vouchers,
};
use ts_rs::{Config, TS};

/// Every file the export must write. A new command type is added here too.
const EXPECTED_FILES: &[&str] = &[
    "AppInfo.ts",
    "AuthStatus.ts",
    "ChangePasswordInput.ts",
    "DataStatus.ts",
    "DayStatus.ts",
    "ErrorCode.ts",
    "FirmDisplay.ts",
    "FirmSettings.ts",
    "FirstRunDone.ts",
    "FirstRunInput.ts",
    "Grouping.ts",
    "Language.ts",
    "LoginInput.ts",
    "PrefKey.ts",
    "PrefsSetInput.ts",
    "RecoverDone.ts",
    "RecoverInput.ts",
    "SessionUser.ts",
    "SetDefaultLanguageInput.ts",
    "SetFirmInput.ts",
    "SetSecurityInput.ts",
    "Settings.ts",
    "UiError.ts",
    "UiErrorItem.ts",
    "UnlockInput.ts",
    "SuggestInput.ts",
    "Suggestions.ts",
    "LearnInput.ts",
    "PairRow.ts",
    "RemovePairInput.ts",
    "Direction.ts",
    "Kind.ts",
    "Source.ts",
    "AccountType.ts",
    "AccountInput.ts",
    "RecordRef.ts",
    "BalanceSide.ts",
    "Balance.ts",
    "AccountRow.ts",
    "AccountFilter.ts",
    "AccountsListInput.ts",
    "AccountsList.ts",
    "AccountsGetInput.ts",
    "AccountsSetActiveInput.ts",
    "AccountsNextCodeInput.ts",
    "AccountsSimilarInput.ts",
    "SimilarAccount.ts",
    "OpeningLineInput.ts",
    "OpeningsInput.ts",
    "LineError.ts",
    "OpeningPreviewRow.ts",
    "OpeningsPreview.ts",
    "OpeningsSaved.ts",
    "CoaTree.ts",
    "HeadNode.ts",
    "ControlNode.ts",
    "AccountNode.ts",
    "CoaHead.ts",
    "CoaControl.ts",
    "MasterKind.ts",
    "MasterInput.ts",
    "CityInput.ts",
    "ItemInput.ts",
    "PackingInput.ts",
    "AddaInput.ts",
    "MarkaInput.ts",
    "StoreInput.ts",
    "NarrationInput.ts",
    "ExpenseHeadInput.ts",
    "StaffInput.ts",
    "BankAccountInput.ts",
    "MastersListInput.ts",
    "MastersSetActiveInput.ts",
    "MastersNextCodeInput.ts",
    "MasterRow.ts",
    "CityRow.ts",
    "ItemRow.ts",
    "PackingRow.ts",
    "AddaRow.ts",
    "MarkaRow.ts",
    "StoreRow.ts",
    "NarrationRow.ts",
    "ExpenseHeadRow.ts",
    "StaffRow.ts",
    "BankAccountRow.ts",
    "SearchScope.ts",
    "SearchHit.ts",
    "SearchMastersInput.ts",
    "SearchRecentInput.ts",
    "SearchTouchInput.ts",
    "Suggestion.ts",
    "SuggestedWord.ts",
    "AccountScope.ts",
    "SearchAccountsInput.ts",
    "AccountHit.ts",
    "ResolveCodeInput.ts",
    "ResolveStatus.ts",
    "ResolvedCode.ts",
    "AccountsBalancesInput.ts",
    "AccountBalance.ts",
    "PartyCardInput.ts",
    "CardWarningKind.ts",
    "CardWarning.ts",
    "PartyCard.ts",
    "SearchGlobalInput.ts",
    "AccountsRecentInput.ts",
    "AccountsTouchInput.ts",
    "GlobalHit.ts",
    "ChargeSide.ts",
    "ChargeType.ts",
    "ChargeMethod.ts",
    "ItemGroup.ts",
    "RuleLevel.ts",
    "SkipReason.ts",
    "SkippedLevel.ts",
    "WinnerView.ts",
    "WorkingValue.ts",
    "WorkingArg.ts",
    "Working.ts",
    "RuleInput.ts",
    "RuleRow.ts",
    "RulesFilter.ts",
    "RulesListInput.ts",
    "RulesGetInput.ts",
    "RulesSetActiveInput.ts",
    "RuleOverlap.ts",
    "RuleSaved.ts",
    "RuleTestInput.ts",
    "ChargeTest.ts",
    "RuleTest.ts",
    "DefaultCharge.ts",
    "ChargesSettings.ts",
    "DefaultChargeInput.ts",
    "ChargesInput.ts",
    "CreditLimitMode.ts",
    "OverridesSettings.ts",
    "VoucherType.ts",
    "VoucherMode.ts",
    "AdvanceInput.ts",
    "VoucherSaved.ts",
    "AdvancesInput.ts",
    "AdvancePreviewRow.ts",
    "AdvancesPreview.ts",
    "AdvanceRowResult.ts",
    "AdvancesSaved.ts",
    "VouchersList.ts",
    "VouchersListInput.ts",
    "VoucherRow.ts",
    "AssistAdvancesInput.ts",
    "OpenAdvance.ts",
    "LastAdvance.ts",
    "AdvancesInfo.ts",
    "ReceiptInput.ts",
    "PaymentInput.ts",
    "VoucherEntry.ts",
    "VoucherPreviewInput.ts",
    "LineSource.ts",
    "OpenBill.ts",
    "BillEffect.ts",
    "AdvanceEffect.ts",
    "VoucherPreview.ts",
    "VoucherGetInput.ts",
    "VoucherEditInput.ts",
    "VoucherCancelInput.ts",
    "VoucherCancelled.ts",
    "OpenBillsInput.ts",
    "OpenBills.ts",
    "OpenPayablesInput.ts",
    "PayablePosting.ts",
    "OpenPayables.ts",
    "AdvancesReportInput.ts",
    "AdvancesReportRow.ts",
    "AdvancesReport.ts",
    "CollectionListInput.ts",
    "CollectionBuyer.ts",
    "CollectionGroup.ts",
    "CollectionList.ts",
    "ReceiptsInput.ts",
    "ReceiptRowResult.ts",
    "ReceiptsSaved.ts",
    "MakeRegularInput.ts",
    "MadeRegular.ts",
    "VoucherHit.ts",
    "DayClosePreviewInput.ts",
    "DayClosePreview.ts",
    "DayCloseInput.ts",
    "DayReopenInput.ts",
    "PartnerRow.ts",
    "ShareHistoryRow.ts",
    "PartnerDetail.ts",
    "PartnersGetInput.ts",
    "PartnerCreateInput.ts",
    "PartnerUpdateInput.ts",
    "ShareInput.ts",
    "PartnersSetSharesInput.ts",
    "PartnersCheckSharesInput.ts",
    "SharesCheck.ts",
    "PartnerOverviewRow.ts",
    "BikriPreviewInput.ts",
    "BikriDraft.ts",
    "HeaderInput.ts",
    "LotInput.ts",
    "SaleInput.ts",
    "PukhtaInput.ts",
    "ChargeInput.ts",
    "LineChargeInput.ts",
    "FreightPaidBy.ts",
    "PaymentType.ts",
    "LotChargeMethod.ts",
    "LineChargeMethod.ts",
    "StoredMethod.ts",
    "ChargeSource.ts",
    "BikriPreview.ts",
    "LotPreview.ts",
    "LinePreview.ts",
    "LineCalc.ts",
    "LineCharge.ts",
    "LotCharge.ts",
    "LineWorkings.ts",
    "SummaryTotals.ts",
    "SummaryWorkings.ts",
    "SplitPart.ts",
    "CashCheck.ts",
    "FieldError.ts",
    "RuleLabel.ts",
    "RateHint.ts",
    "RateHintSource.ts",
    "SplitFreightInput.ts",
    "FreightLotInput.ts",
    "FreightPart.ts",
    "TruckMemoryInput.ts",
    "TruckMemory.ts",
    "TruckLast.ts",
    "TruckSameDay.ts",
    "TruckSuggestion.ts",
    "SellerMemoryInput.ts",
    "SellerMemory.ts",
    "LastRateInput.ts",
    "BikriSaveInput.ts",
    "BikriSaved.ts",
    "BikriStatus.ts",
    "BikriGetInput.ts",
    "SavedPukhta.ts",
    "SavedCharge.ts",
    "SavedLotCharge.ts",
    "SavedSale.ts",
    "SavedLot.ts",
    "SavedTotals.ts",
    "SavedPosting.ts",
    "SavedSplit.ts",
    "BikriDetail.ts",
    "BikriListInput.ts",
    "BikriListRow.ts",
    "BikriFindInput.ts",
    "NavigateTo.ts",
    "BikriNavigateInput.ts",
    "PukhtaSide.ts",
    "PukhtaPeopleInput.ts",
    "PukhtaPerson.ts",
    "PaidState.ts",
    "PukhtaRegisterInput.ts",
    "PukhtaRegisterRow.ts",
    "BikriMatch.ts",
    "BikriHit.ts",
    "StockListInput.ts",
    "WriteOffRow.ts",
    "StockRow.ts",
    "StockList.ts",
    "OwnStockRow.ts",
    "SavedPurchaseRef.ts",
    "SavedCredit.ts",
    "SavedOwnLot.ts",
    "OwnLotPreview.ts",
    "MoneySource.ts",
    "VentureKind.ts",
    "VentureStatus.ts",
    "VenturePartnerInput.ts",
    "VenturePartnerRow.ts",
    "EntryPartInput.ts",
    "EntryPart.ts",
    "EntryParts.ts",
    "ShareAmount.ts",
    "TripRef.ts",
    "PurchaseInput.ts",
    "PurchasePreview.ts",
    "PurchaseNumbers.ts",
    "PurchaseWorkings.ts",
    "PurchaseSaved.ts",
    "PurchaseCancelInput.ts",
    "PurchaseCancelled.ts",
    "PurchaseGetInput.ts",
    "PurchasesListInput.ts",
    "PurchasesList.ts",
    "PurchasesTotals.ts",
    "PurchaseRow.ts",
    "PurchaseLotRef.ts",
    "PurchaseStatus.ts",
    "PurchaseStateFilter.ts",
    "PurchaseWriteOffInput.ts",
    "PurchaseWriteOffDone.ts",
    "PurchaseWriteOffUndoInput.ts",
    "PurchaseWriteOffUndone.ts",
    "TripInput.ts",
    "TripSetPartnersInput.ts",
    "TripDefaults.ts",
    "TripsListInput.ts",
    "TripsList.ts",
    "TripsTotals.ts",
    "TripRow.ts",
    "TripGetInput.ts",
    "TripDetail.ts",
    "TripEntryRow.ts",
    "TripEntryKind.ts",
    "CostSpreadPart.ts",
    "TripPartnerResult.ts",
    "TripsSummary.ts",
    "TripCostInput.ts",
    "TripCostNumbers.ts",
    "TripCostPreview.ts",
    "TripCostSaved.ts",
    "OtherSaleInput.ts",
    "OtherSaleNumbers.ts",
    "OtherSalePreview.ts",
    "OtherSaleSaved.ts",
    "TripEntryCancelInput.ts",
    "TripEntryCancelled.ts",
    "LaterSaleInput.ts",
    "LaterSaleSaveInput.ts",
    "LaterSaleSaved.ts",
    "EditOpenInput.ts",
    "EditableBikri.ts",
    "EditPreviewInput.ts",
    "EditSaveInput.ts",
    "BikriEdited.ts",
    "CancelPreviewInput.ts",
    "CancelPreview.ts",
    "CancelInput.ts",
    "BikriCancelled.ts",
    "UndoInput.ts",
    "UndoResult.ts",
    "WriteOffInput.ts",
    "WriteOffDone.ts",
    "WriteOffUndoInput.ts",
    "WriteOffUndone.ts",
    "DraftSaveInput.ts",
    "DraftSaved.ts",
    "DraftGetInput.ts",
    "DraftDiscardInput.ts",
    "DraftRow.ts",
    "PrintDoc.ts",
    "Paper.ts",
    "PrintTo.ts",
    "PrintingSettings.ts",
    "SetPrintingInput.ts",
    "PrintInput.ts",
    "PrintOutcome.ts",
    "BikriPrintDataInput.ts",
    "VoucherPrintDataInput.ts",
    "PrintVoucherInput.ts",
    "TextDir.ts",
    "PrintFirm.ts",
    "PrintFact.ts",
    "PrintParty.ts",
    "PrintLine.ts",
    "AmountRole.ts",
    "PrintAmount.ts",
    "PrintWords.ts",
    "PrintPage.ts",
    "PrintData.ts",
    // Run B, plan 18 (append-only block).
    "BackupRef.ts",
    "CloudLevel.ts",
    "StatusError.ts",
    "BackupStatus.ts",
    "BackupListInput.ts",
    "RowStatus.ts",
    "BackupRow.ts",
    "BackupList.ts",
    "TargetResult.ts",
    "RunReport.ts",
    "RestorePreviewInput.ts",
    "LostWork.ts",
    "RestorePreview.ts",
    "RestoreInput.ts",
    "RestoreDone.ts",
    "TestRestoreInput.ts",
    "TestCheckName.ts",
    "TestCheck.ts",
    "TestRestoreResult.ts",
    "KeyCreated.ts",
    "KeyConfirmInput.ts",
    "BackupSettingsView.ts",
    "BackupSettingsInput.ts",
    "BackupNowInput.ts",
    "BackupKind.ts",
    "BackupTotals.ts",
    "BackupSettings.ts",
    "BackupCode.ts",
    "WireCode.ts",
    "BackupErrorItem.ts",
    "BackupWireError.ts",
];

fn export_dir() -> PathBuf {
    match std::env::var_os("TS_RS_EXPORT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => std::env::temp_dir().join(format!("mandi-bindings-{}", std::process::id())),
    }
}

#[test]
fn export_bindings() {
    let dir = export_dir();
    let cfg = Config::from_env().with_out_dir(&dir);

    // export_all also writes every type these types use.
    AppInfo::export_all(&cfg).unwrap();
    DataStatus::export_all(&cfg).unwrap();
    auth::AuthStatus::export_all(&cfg).unwrap();
    auth::SessionUser::export_all(&cfg).unwrap();
    auth::FirmDisplay::export_all(&cfg).unwrap();
    auth::FirstRunInput::export_all(&cfg).unwrap();
    auth::FirstRunDone::export_all(&cfg).unwrap();
    auth::LoginInput::export_all(&cfg).unwrap();
    auth::UnlockInput::export_all(&cfg).unwrap();
    auth::RecoverInput::export_all(&cfg).unwrap();
    auth::RecoverDone::export_all(&cfg).unwrap();
    auth::ChangePasswordInput::export_all(&cfg).unwrap();
    settings::Settings::export_all(&cfg).unwrap();
    settings::FirmSettings::export_all(&cfg).unwrap();
    settings::SetFirmInput::export_all(&cfg).unwrap();
    settings::SetSecurityInput::export_all(&cfg).unwrap();
    settings::SetDefaultLanguageInput::export_all(&cfg).unwrap();
    mandi_services::core::auth::PrefKey::export_all(&cfg).unwrap();
    settings::PrefsSetInput::export_all(&cfg).unwrap();
    day::DayStatus::export_all(&cfg).unwrap();
    translit::SuggestInput::export_all(&cfg).unwrap();
    translit::Suggestions::export_all(&cfg).unwrap();
    translit::LearnInput::export_all(&cfg).unwrap();
    translit::PairRow::export_all(&cfg).unwrap();
    translit::RemovePairInput::export_all(&cfg).unwrap();
    masters::MasterInput::export_all(&cfg).unwrap();
    masters::MasterRow::export_all(&cfg).unwrap();
    masters::MasterKind::export_all(&cfg).unwrap();
    masters::MastersListInput::export_all(&cfg).unwrap();
    masters::MastersSetActiveInput::export_all(&cfg).unwrap();
    masters::MastersNextCodeInput::export_all(&cfg).unwrap();
    search::SearchScope::export_all(&cfg).unwrap();
    search::SearchHit::export_all(&cfg).unwrap();
    search::SearchMastersInput::export_all(&cfg).unwrap();
    search::SearchRecentInput::export_all(&cfg).unwrap();
    search::SearchTouchInput::export_all(&cfg).unwrap();
    accounts::AccountInput::export_all(&cfg).unwrap();
    accounts::AccountRow::export_all(&cfg).unwrap();
    accounts::AccountsListInput::export_all(&cfg).unwrap();
    accounts::AccountsList::export_all(&cfg).unwrap();
    accounts::AccountsGetInput::export_all(&cfg).unwrap();
    accounts::AccountsSetActiveInput::export_all(&cfg).unwrap();
    accounts::AccountsNextCodeInput::export_all(&cfg).unwrap();
    accounts::AccountsSimilarInput::export_all(&cfg).unwrap();
    accounts::SimilarAccount::export_all(&cfg).unwrap();
    accounts::OpeningsInput::export_all(&cfg).unwrap();
    accounts::OpeningsPreview::export_all(&cfg).unwrap();
    accounts::OpeningsSaved::export_all(&cfg).unwrap();
    accounts::CoaTree::export_all(&cfg).unwrap();
    accounts::CoaHead::export_all(&cfg).unwrap();
    accounts::CoaControl::export_all(&cfg).unwrap();
    accounts::SearchAccountsInput::export_all(&cfg).unwrap();
    accounts::ResolveCodeInput::export_all(&cfg).unwrap();
    accounts::ResolvedCode::export_all(&cfg).unwrap();
    accounts::AccountsBalancesInput::export_all(&cfg).unwrap();
    accounts::AccountBalance::export_all(&cfg).unwrap();
    accounts::PartyCardInput::export_all(&cfg).unwrap();
    accounts::PartyCard::export_all(&cfg).unwrap();
    accounts::SearchGlobalInput::export_all(&cfg).unwrap();
    accounts::AccountsRecentInput::export_all(&cfg).unwrap();
    accounts::AccountsTouchInput::export_all(&cfg).unwrap();
    accounts::GlobalHit::export_all(&cfg).unwrap();
    rules::RuleInput::export_all(&cfg).unwrap();
    rules::RuleSaved::export_all(&cfg).unwrap();
    rules::RulesListInput::export_all(&cfg).unwrap();
    rules::RulesGetInput::export_all(&cfg).unwrap();
    rules::RulesSetActiveInput::export_all(&cfg).unwrap();
    rules::RuleTestInput::export_all(&cfg).unwrap();
    rules::RuleTest::export_all(&cfg).unwrap();
    rules::ChargesSettings::export_all(&cfg).unwrap();
    rules::ChargesInput::export_all(&cfg).unwrap();
    rules::OverridesSettings::export_all(&cfg).unwrap();
    vouchers::AdvanceInput::export_all(&cfg).unwrap();
    vouchers::VoucherSaved::export_all(&cfg).unwrap();
    vouchers::AdvancesInput::export_all(&cfg).unwrap();
    vouchers::AdvancesPreview::export_all(&cfg).unwrap();
    vouchers::AdvancesSaved::export_all(&cfg).unwrap();
    vouchers::VouchersListInput::export_all(&cfg).unwrap();
    vouchers::VouchersList::export_all(&cfg).unwrap();
    vouchers::VoucherRow::export_all(&cfg).unwrap();
    vouchers::AssistAdvancesInput::export_all(&cfg).unwrap();
    vouchers::AdvancesInfo::export_all(&cfg).unwrap();
    vouchers::ReceiptInput::export_all(&cfg).unwrap();
    vouchers::PaymentInput::export_all(&cfg).unwrap();
    vouchers::VoucherEntry::export_all(&cfg).unwrap();
    vouchers::VoucherPreviewInput::export_all(&cfg).unwrap();
    vouchers::LineSource::export_all(&cfg).unwrap();
    vouchers::OpenBill::export_all(&cfg).unwrap();
    vouchers::BillEffect::export_all(&cfg).unwrap();
    vouchers::AdvanceEffect::export_all(&cfg).unwrap();
    vouchers::VoucherPreview::export_all(&cfg).unwrap();
    vouchers::VoucherGetInput::export_all(&cfg).unwrap();
    vouchers::VoucherEditInput::export_all(&cfg).unwrap();
    vouchers::VoucherCancelInput::export_all(&cfg).unwrap();
    vouchers::VoucherCancelled::export_all(&cfg).unwrap();
    vouchers::OpenBillsInput::export_all(&cfg).unwrap();
    vouchers::OpenBills::export_all(&cfg).unwrap();
    vouchers::OpenPayablesInput::export_all(&cfg).unwrap();
    vouchers::PayablePosting::export_all(&cfg).unwrap();
    vouchers::OpenPayables::export_all(&cfg).unwrap();
    vouchers::AdvancesReportInput::export_all(&cfg).unwrap();
    vouchers::AdvancesReportRow::export_all(&cfg).unwrap();
    vouchers::AdvancesReport::export_all(&cfg).unwrap();
    vouchers::CollectionListInput::export_all(&cfg).unwrap();
    vouchers::CollectionBuyer::export_all(&cfg).unwrap();
    vouchers::CollectionGroup::export_all(&cfg).unwrap();
    vouchers::CollectionList::export_all(&cfg).unwrap();
    vouchers::ReceiptsInput::export_all(&cfg).unwrap();
    vouchers::ReceiptRowResult::export_all(&cfg).unwrap();
    vouchers::ReceiptsSaved::export_all(&cfg).unwrap();
    vouchers::MakeRegularInput::export_all(&cfg).unwrap();
    vouchers::MadeRegular::export_all(&cfg).unwrap();
    vouchers::VoucherHit::export_all(&cfg).unwrap();
    vouchers::DayClosePreviewInput::export_all(&cfg).unwrap();
    vouchers::DayClosePreview::export_all(&cfg).unwrap();
    vouchers::DayCloseInput::export_all(&cfg).unwrap();
    vouchers::DayReopenInput::export_all(&cfg).unwrap();
    partners::PartnerRow::export_all(&cfg).unwrap();
    partners::PartnerDetail::export_all(&cfg).unwrap();
    partners::PartnersGetInput::export_all(&cfg).unwrap();
    partners::PartnerCreateInput::export_all(&cfg).unwrap();
    partners::PartnerUpdateInput::export_all(&cfg).unwrap();
    partners::PartnersSetSharesInput::export_all(&cfg).unwrap();
    partners::PartnersCheckSharesInput::export_all(&cfg).unwrap();
    partners::SharesCheck::export_all(&cfg).unwrap();
    partners::PartnerOverviewRow::export_all(&cfg).unwrap();
    bikri::BikriPreviewInput::export_all(&cfg).unwrap();
    bikri::BikriPreview::export_all(&cfg).unwrap();
    bikri::SplitFreightInput::export_all(&cfg).unwrap();
    bikri::FreightPart::export_all(&cfg).unwrap();
    bikri::TruckMemoryInput::export_all(&cfg).unwrap();
    bikri::TruckMemory::export_all(&cfg).unwrap();
    bikri::SellerMemoryInput::export_all(&cfg).unwrap();
    bikri::SellerMemory::export_all(&cfg).unwrap();
    bikri::LastRateInput::export_all(&cfg).unwrap();
    bikri::RateHint::export_all(&cfg).unwrap();
    bikri::BikriSaveInput::export_all(&cfg).unwrap();
    bikri::BikriSaved::export_all(&cfg).unwrap();
    bikri::BikriGetInput::export_all(&cfg).unwrap();
    bikri::BikriDetail::export_all(&cfg).unwrap();
    bikri::BikriListInput::export_all(&cfg).unwrap();
    bikri::BikriListRow::export_all(&cfg).unwrap();
    bikri::BikriFindInput::export_all(&cfg).unwrap();
    bikri::BikriNavigateInput::export_all(&cfg).unwrap();
    bikri::PukhtaPeopleInput::export_all(&cfg).unwrap();
    bikri::PukhtaPerson::export_all(&cfg).unwrap();
    bikri::PukhtaRegisterInput::export_all(&cfg).unwrap();
    bikri::PukhtaRegisterRow::export_all(&cfg).unwrap();
    bikri::BikriHit::export_all(&cfg).unwrap();
    bikri::StockListInput::export_all(&cfg).unwrap();
    bikri::StockList::export_all(&cfg).unwrap();
    bikri::OwnStockRow::export_all(&cfg).unwrap();
    bikri::SavedPurchaseRef::export_all(&cfg).unwrap();
    bikri::SavedOwnLot::export_all(&cfg).unwrap();
    bikri::OwnLotPreview::export_all(&cfg).unwrap();
    ventures::MoneySource::export_all(&cfg).unwrap();
    ventures::VentureKind::export_all(&cfg).unwrap();
    ventures::VentureStatus::export_all(&cfg).unwrap();
    ventures::VenturePartnerInput::export_all(&cfg).unwrap();
    ventures::VenturePartnerRow::export_all(&cfg).unwrap();
    ventures::EntryPartInput::export_all(&cfg).unwrap();
    ventures::EntryPart::export_all(&cfg).unwrap();
    ventures::EntryParts::export_all(&cfg).unwrap();
    ventures::ShareAmount::export_all(&cfg).unwrap();
    ventures::TripRef::export_all(&cfg).unwrap();
    purchases::PurchaseInput::export_all(&cfg).unwrap();
    purchases::PurchasePreview::export_all(&cfg).unwrap();
    purchases::PurchaseNumbers::export_all(&cfg).unwrap();
    purchases::PurchaseWorkings::export_all(&cfg).unwrap();
    purchases::PurchaseSaved::export_all(&cfg).unwrap();
    purchases::PurchaseCancelInput::export_all(&cfg).unwrap();
    purchases::PurchaseCancelled::export_all(&cfg).unwrap();
    purchases::PurchaseGetInput::export_all(&cfg).unwrap();
    purchases::PurchasesListInput::export_all(&cfg).unwrap();
    purchases::PurchasesList::export_all(&cfg).unwrap();
    purchases::PurchasesTotals::export_all(&cfg).unwrap();
    purchases::PurchaseRow::export_all(&cfg).unwrap();
    purchases::PurchaseLotRef::export_all(&cfg).unwrap();
    purchases::PurchaseStatus::export_all(&cfg).unwrap();
    purchases::PurchaseStateFilter::export_all(&cfg).unwrap();
    purchases::PurchaseWriteOffInput::export_all(&cfg).unwrap();
    purchases::PurchaseWriteOffDone::export_all(&cfg).unwrap();
    purchases::PurchaseWriteOffUndoInput::export_all(&cfg).unwrap();
    purchases::PurchaseWriteOffUndone::export_all(&cfg).unwrap();
    trips::TripInput::export_all(&cfg).unwrap();
    trips::TripSetPartnersInput::export_all(&cfg).unwrap();
    trips::TripDefaults::export_all(&cfg).unwrap();
    trips::TripsListInput::export_all(&cfg).unwrap();
    trips::TripsList::export_all(&cfg).unwrap();
    trips::TripsTotals::export_all(&cfg).unwrap();
    trips::TripRow::export_all(&cfg).unwrap();
    trips::TripGetInput::export_all(&cfg).unwrap();
    trips::TripDetail::export_all(&cfg).unwrap();
    trips::TripEntryRow::export_all(&cfg).unwrap();
    trips::TripEntryKind::export_all(&cfg).unwrap();
    trips::CostSpreadPart::export_all(&cfg).unwrap();
    trips::TripPartnerResult::export_all(&cfg).unwrap();
    trips::TripsSummary::export_all(&cfg).unwrap();
    trips::TripCostInput::export_all(&cfg).unwrap();
    trips::TripCostNumbers::export_all(&cfg).unwrap();
    trips::TripCostPreview::export_all(&cfg).unwrap();
    trips::TripCostSaved::export_all(&cfg).unwrap();
    trips::OtherSaleInput::export_all(&cfg).unwrap();
    trips::OtherSaleNumbers::export_all(&cfg).unwrap();
    trips::OtherSalePreview::export_all(&cfg).unwrap();
    trips::OtherSaleSaved::export_all(&cfg).unwrap();
    trips::TripEntryCancelInput::export_all(&cfg).unwrap();
    trips::TripEntryCancelled::export_all(&cfg).unwrap();
    bikri::LaterSaleInput::export_all(&cfg).unwrap();
    bikri::LaterSaleSaveInput::export_all(&cfg).unwrap();
    bikri::LaterSaleSaved::export_all(&cfg).unwrap();
    bikri::EditOpenInput::export_all(&cfg).unwrap();
    bikri::EditableBikri::export_all(&cfg).unwrap();
    bikri::EditPreviewInput::export_all(&cfg).unwrap();
    bikri::EditSaveInput::export_all(&cfg).unwrap();
    bikri::BikriEdited::export_all(&cfg).unwrap();
    bikri::CancelPreviewInput::export_all(&cfg).unwrap();
    bikri::CancelPreview::export_all(&cfg).unwrap();
    bikri::CancelInput::export_all(&cfg).unwrap();
    bikri::BikriCancelled::export_all(&cfg).unwrap();
    bikri::UndoInput::export_all(&cfg).unwrap();
    bikri::UndoResult::export_all(&cfg).unwrap();
    bikri::WriteOffInput::export_all(&cfg).unwrap();
    bikri::WriteOffDone::export_all(&cfg).unwrap();
    bikri::WriteOffUndoInput::export_all(&cfg).unwrap();
    bikri::WriteOffUndone::export_all(&cfg).unwrap();
    drafts::DraftSaveInput::export_all(&cfg).unwrap();
    drafts::DraftSaved::export_all(&cfg).unwrap();
    drafts::DraftGetInput::export_all(&cfg).unwrap();
    drafts::DraftDiscardInput::export_all(&cfg).unwrap();
    drafts::DraftRow::export_all(&cfg).unwrap();
    cash::CashCheck::export_all(&cfg).unwrap();
    printing::PrintingSettings::export_all(&cfg).unwrap();
    printing::SetPrintingInput::export_all(&cfg).unwrap();
    printing::PrintInput::export_all(&cfg).unwrap();
    printing::PrintOutcome::export_all(&cfg).unwrap();
    mandi_reports::print::BikriPrintDataInput::export_all(&cfg).unwrap();
    mandi_reports::print::VoucherPrintDataInput::export_all(&cfg).unwrap();
    printing::PrintVoucherInput::export_all(&cfg).unwrap();
    mandi_reports::print::PrintData::export_all(&cfg).unwrap();
    UiError::export_all(&cfg).unwrap();
    UiErrorItem::export_all(&cfg).unwrap();
    ErrorCode::export_all(&cfg).unwrap();
    // Run B, plan 18 (append-only block).
    mandi_backup::api::BackupStatus::export_all(&cfg).unwrap();
    mandi_backup::api::BackupListInput::export_all(&cfg).unwrap();
    mandi_backup::api::BackupList::export_all(&cfg).unwrap();
    mandi_backup::api::BackupNowInput::export_all(&cfg).unwrap();
    mandi_backup::api::RunReport::export_all(&cfg).unwrap();
    mandi_backup::api::RestorePreviewInput::export_all(&cfg).unwrap();
    mandi_backup::api::RestorePreview::export_all(&cfg).unwrap();
    mandi_backup::api::RestoreInput::export_all(&cfg).unwrap();
    mandi_backup::api::RestoreDone::export_all(&cfg).unwrap();
    mandi_backup::api::TestRestoreInput::export_all(&cfg).unwrap();
    mandi_backup::api::TestRestoreResult::export_all(&cfg).unwrap();
    mandi_backup::api::KeyCreated::export_all(&cfg).unwrap();
    mandi_backup::api::KeyConfirmInput::export_all(&cfg).unwrap();
    mandi_backup::api::BackupSettingsView::export_all(&cfg).unwrap();
    mandi_backup::api::BackupSettingsInput::export_all(&cfg).unwrap();
    mandi_backup::BackupCode::export_all(&cfg).unwrap();
    mandi_app_lib::commands::backup::BackupWireError::export_all(&cfg).unwrap();

    for file in EXPECTED_FILES {
        assert!(
            dir.join(file).is_file(),
            "missing binding {file} in {}",
            dir.display()
        );
    }
}

#[test]
fn bindings_have_the_contract_shape() {
    let cfg = Config::new();
    let app_info = AppInfo::decl(&cfg);
    assert!(app_info.contains("version: string"), "{app_info}");
    assert!(app_info.contains("identifier: string"), "{app_info}");

    let status = DataStatus::decl(&cfg);
    assert!(
        status.contains("state: 'checking' | 'ok' | 'failed'")
            || status.contains("state: \"checking\" | \"ok\" | \"failed\""),
        "{status}"
    );
    assert!(status.contains("reference: string | null"), "{status}");

    let ui_error = UiError::decl(&cfg);
    assert!(ui_error.contains("ok: false"), "{ui_error}");
    assert!(
        ui_error.contains("errors: Array<UiErrorItem>"),
        "{ui_error}"
    );
    assert!(ui_error.contains("ref: string"), "{ui_error}");

    let item = UiErrorItem::decl(&cfg);
    assert!(item.contains("code: ErrorCode"), "{item}");
    assert!(item.contains("field: string | null"), "{item}");
    assert!(
        item.contains("values: Record<string, string | number>"),
        "{item}"
    );

    let codes = ErrorCode::decl(&cfg);
    assert!(codes.contains("\"ERR_NOT_IMPLEMENTED\""), "{codes}");
    assert!(codes.contains("\"ERR_INTERNAL\""), "{codes}");
}

/// Every code of plan 02 Section 4.6 and plan 03 Section 4.6 is in the exported union, spelled as the JSON the
/// commands send, and nothing else is.
#[test]
fn error_code_union_has_every_code() {
    let decl = ErrorCode::decl(&Config::new());
    for code in ErrorCode::ALL {
        let json = serde_json::to_string(&code).unwrap();
        assert!(decl.contains(&json), "{json} missing in {decl}");
    }
    assert_eq!(
        decl.matches("\"ERR_").count(),
        ErrorCode::ALL.len(),
        "{decl}"
    );
    for code in [
        "ERR_DISK_FULL",
        "ERR_DATA_CHECK_FAILED",
        "ERR_MIGRATION_FAILED",
        "ERR_DB_BUSY",
        "ERR_REQUIRED",
        "ERR_NOT_A_NUMBER",
        "ERR_DECIMALS_NOT_ALLOWED",
        "ERR_NEGATIVE_NOT_ALLOWED",
        "ERR_AMOUNT_TOO_LARGE",
        "ERR_MUST_BE_MORE_THAN_ZERO",
        "ERR_TOO_MANY_DECIMALS",
        "ERR_PERCENT_OVER_100",
        "ERR_QTY_TOO_LARGE",
        "ERR_QTY_NOT_WHOLE",
        "ERR_WEIGHT_TOO_LARGE",
        "ERR_DATE_INVALID",
        "ERR_DATE_CLOSED",
        "ERR_DATE_IN_FUTURE",
        "ERR_CODE_INVALID",
        "ERR_NAME_UR_SCRIPT",
        "ERR_NAME_EN_SCRIPT",
        "ERR_NAME_TOO_LONG",
        "ERR_PHONE_INVALID",
        "ERR_TEXT_TOO_LONG",
        "ERR_CONTROL_CHARS",
        "ERR_OVERFLOW",
        "ERR_SHARES_NOT_100",
    ] {
        assert!(
            decl.contains(&format!("\"{code}\"")),
            "{code} missing in {decl}"
        );
    }
}

/// Money fields are typed `Paisa` (the frontend's opaque type) and no `Paisa.ts` is
/// written or imported from `./Paisa` (plan 02 Section 4.1).
#[test]
fn money_fields_are_typed_paisa() {
    #[derive(TS)]
    #[allow(dead_code)]
    struct MoneyProbe {
        amount: Paisa,
        maybe: Option<Paisa>,
    }
    let cfg = Config::new();
    let decl = MoneyProbe::decl(&cfg);
    assert!(decl.contains("amount: Paisa"), "{decl}");
    assert!(decl.contains("maybe: Paisa | null"), "{decl}");
    let file = MoneyProbe::export_to_string(&cfg).unwrap();
    assert!(!file.contains("./Paisa"), "{file}");
    assert!(<Paisa as TS>::output_path().is_none());
}

/// The plan 04 contract: property names are the snake_case names of the plan, ids are numbers,
/// and the closed lists are string unions.
#[test]
fn plan_04_contract_shapes() {
    let cfg = Config::new();
    let has = |decl: String, parts: &[&str]| {
        for part in parts {
            assert!(decl.contains(part), "{part} missing in {decl}");
        }
    };
    has(
        auth::AuthStatus::decl(&cfg),
        &[
            "first_run: boolean",
            "session: SessionUser | null",
            "locked: boolean",
            "login_language: Language",
            "firm: FirmDisplay",
            "app_version: string",
        ],
    );
    has(
        auth::SessionUser::decl(&cfg),
        &[
            "id: number",
            "username: string",
            "display_name: string",
            "language: Language",
            "text_size: number",
        ],
    );
    has(
        auth::FirstRunInput::decl(&cfg),
        &[
            "firm_name_ur: string",
            "firm_name_en: string",
            "mandi_ur: string",
            "mandi_en: string",
            "address: string",
            "phone: string",
            "owner_username: string",
            "owner_display_name: string",
            "password: string",
            "password_repeat: string",
            "confirmed_rules_words: Array<string>",
        ],
    );
    has(auth::FirstRunDone::decl(&cfg), &["recovery_code: string"]);
    has(
        auth::LoginInput::decl(&cfg),
        &["username: string", "password: string"],
    );
    has(auth::UnlockInput::decl(&cfg), &["password: string"]);
    has(
        auth::RecoverInput::decl(&cfg),
        &[
            "username: string",
            "recovery_code: string",
            "new_password: string",
            "new_password_repeat: string",
        ],
    );
    has(
        auth::RecoverDone::decl(&cfg),
        &["new_recovery_code: string"],
    );
    has(
        auth::ChangePasswordInput::decl(&cfg),
        &["current: string", "new: string", "repeat: string"],
    );
    has(
        settings::SetFirmInput::decl(&cfg),
        &[
            "name_ur: string",
            "name_en: string",
            "mandi_ur: string",
            "mandi_en: string",
            "address: string",
            "phone: string",
            "confirmed_rules_words: Array<string>",
        ],
    );
    has(
        settings::Settings::decl(&cfg),
        &[
            "firm: FirmSettings",
            "idle_lock_minutes: number",
            "default_language: Language",
            "number_grouping: Grouping",
        ],
    );
    has(
        settings::SetSecurityInput::decl(&cfg),
        &["idle_lock_minutes: string"],
    );
    has(
        settings::PrefsSetInput::decl(&cfg),
        &["key: PrefKey", "value: string"],
    );
    has(
        day::DayStatus::decl(&cfg),
        &[
            "today: string",
            "closed_up_to: string | null",
            "today_open: boolean",
        ],
    );
    let keys = mandi_services::core::auth::PrefKey::decl(&cfg);
    for word in ["language", "text_size", "last_screen"] {
        assert!(keys.contains(word), "{keys}");
    }
    let grouping = mandi_services::core::money::Grouping::decl(&cfg);
    assert!(
        grouping.contains("international") && grouping.contains("lakh"),
        "{grouping}"
    );
}
