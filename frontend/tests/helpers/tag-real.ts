// The real answers and requests of the backend for plan 17e (the tag, the firm's contacts and trade
// mark, the shared print header, the collection list and global search): the account saved with a
// tag and refused for a bad one, the duplicate check, the party card, the searches, the firm saved
// with contacts and refused, the header and footer of a report print, the collection list in both
// languages and empty, and the sale book's documents with a tag. The backend wrote them in its own
// tests (backend/crates/services/tests/fixtures); they are copied here, never edited, and read as
// the generated types. The story: Akram Toll Tax K (0020, short code 1) owes 2,880 and Arshad
// Multani 18,684 on 10-10-2026.

import type { AccountInput } from '../../src/bindings/AccountInput';
import type { AccountMatch } from '../../src/shared/accounts';
import type { AccountRow } from '../../src/bindings/AccountRow';
import type { AccountsSimilarInput } from '../../src/bindings/AccountsSimilarInput';
import type { CollectionPrintData } from '../../src/bindings/CollectionPrintData';
import type { CollectionPrintDataInput } from '../../src/bindings/CollectionPrintDataInput';
import type { GlobalHit } from '../../src/bindings/GlobalHit';
import type { PartyCard } from '../../src/bindings/PartyCard';
import type { PrintData } from '../../src/bindings/PrintData';
import type { PrintFrame } from '../../src/bindings/PrintFrame';
import type { PrintFrameInput } from '../../src/bindings/PrintFrameInput';
import type { SearchAccountsInput } from '../../src/bindings/SearchAccountsInput';
import type { SearchGlobalInput } from '../../src/bindings/SearchGlobalInput';
import type { Settings } from '../../src/bindings/Settings';
import type { SetFirmInput } from '../../src/bindings/SetFirmInput';
import type { AccountHit } from '../../src/bindings/AccountHit';
import type { UiError } from '../../src/bindings/UiError';

import accountsSaveTag from '../fixtures/accounts_save_tag.json';
import accountsSaveTagRequest from '../fixtures/accounts_save_tag_request.json';
import accountsSaveTagRefused from '../fixtures/accounts_save_tag_refused.json';
import accountsSaveTagRefusedRequest from '../fixtures/accounts_save_tag_refused_request.json';
import accountsSimilarTagDuplicate from '../fixtures/accounts_similar_tag_duplicate.json';
import accountsSimilarTagDuplicateRequest from '../fixtures/accounts_similar_tag_duplicate_request.json';
import accountsSimilarTagOther from '../fixtures/accounts_similar_tag_other.json';
import accountsSimilarTagOtherRequest from '../fixtures/accounts_similar_tag_other_request.json';
import assistPartyCardTag from '../fixtures/assist_party_card_tag.json';
import reportsBikriPrintDataTagBuyerSlipsEn from '../fixtures/reports_bikri_print_data_tag_buyer_slips_en.json';
import reportsBikriPrintDataTagBuyerSlipsUr from '../fixtures/reports_bikri_print_data_tag_buyer_slips_ur.json';
import reportsBikriPrintDataTagSellerBillUr from '../fixtures/reports_bikri_print_data_tag_seller_bill_ur.json';
import reportsCollectionPrintDataEmpty from '../fixtures/reports_collection_print_data_empty.json';
import reportsCollectionPrintDataEmptyRequest from '../fixtures/reports_collection_print_data_empty_request.json';
import reportsCollectionPrintDataEn from '../fixtures/reports_collection_print_data_en.json';
import reportsCollectionPrintDataEnRequest from '../fixtures/reports_collection_print_data_en_request.json';
import reportsCollectionPrintDataUr from '../fixtures/reports_collection_print_data_ur.json';
import reportsCollectionPrintDataUrRequest from '../fixtures/reports_collection_print_data_ur_request.json';
import reportsPrintHeaderEn from '../fixtures/reports_print_header_en.json';
import reportsPrintHeaderEnRequest from '../fixtures/reports_print_header_en_request.json';
import reportsPrintHeaderUr from '../fixtures/reports_print_header_ur.json';
import reportsPrintHeaderUrRequest from '../fixtures/reports_print_header_ur_request.json';
import searchAccountsTag from '../fixtures/search_accounts_tag.json';
import searchAccountsTagRequest from '../fixtures/search_accounts_tag_request.json';
import searchGlobalTag from '../fixtures/search_global_tag.json';
import searchGlobalTagNumber from '../fixtures/search_global_tag_number.json';
import searchGlobalTagRequest from '../fixtures/search_global_tag_request.json';
import settingsGetContacts from '../fixtures/settings_get_contacts.json';
import settingsSetFirmContacts from '../fixtures/settings_set_firm_contacts.json';
import settingsSetFirmContactsRefused from '../fixtures/settings_set_firm_contacts_refused.json';
import settingsSetFirmContactsRefusedRequest from '../fixtures/settings_set_firm_contacts_refused_request.json';
import settingsSetFirmContactsRequest from '../fixtures/settings_set_firm_contacts_request.json';

// The fixture files are the backend's own JSON; each is read as its generated type. A reader takes
// `unknown`, because the money in a binding is a branded type that a plain number is not.
function readAccountHits(value: unknown): AccountHit[] {
  return value as AccountHit[];
}

function readAccountInput(value: unknown): AccountInput {
  return value as AccountInput;
}

function readAccountMatches(value: unknown): AccountMatch[] {
  return value as AccountMatch[];
}

function readAccountRow(value: unknown): AccountRow {
  return value as AccountRow;
}

function readAccountsSimilarInput(value: unknown): AccountsSimilarInput {
  return value as AccountsSimilarInput;
}

function readCollectionPrintData(value: unknown): CollectionPrintData {
  return value as CollectionPrintData;
}

function readCollectionPrintDataInput(value: unknown): CollectionPrintDataInput {
  return value as CollectionPrintDataInput;
}

function readGlobalHits(value: unknown): GlobalHit[] {
  return value as GlobalHit[];
}

function readPartyCard(value: unknown): PartyCard {
  return value as PartyCard;
}

function readPrintData(value: unknown): PrintData {
  return value as PrintData;
}

function readPrintFrame(value: unknown): PrintFrame {
  return value as PrintFrame;
}

function readPrintFrameInput(value: unknown): PrintFrameInput {
  return value as PrintFrameInput;
}

function readSearchAccountsInput(value: unknown): SearchAccountsInput {
  return value as SearchAccountsInput;
}

function readSearchGlobalInput(value: unknown): SearchGlobalInput {
  return value as SearchGlobalInput;
}

function readSetFirmInput(value: unknown): SetFirmInput {
  return value as SetFirmInput;
}

function readSettings(value: unknown): Settings {
  return value as Settings;
}

function readFirmSettings(value: unknown): Settings['firm'] {
  return value as Settings['firm'];
}

function readUiError(value: unknown): UiError {
  return value as UiError;
}

export const ACCOUNTS_SAVE_TAG = readAccountRow(accountsSaveTag);
export const ACCOUNTS_SAVE_TAG_REQUEST = readAccountInput(accountsSaveTagRequest);
export const ACCOUNTS_SAVE_TAG_REFUSED = readUiError(accountsSaveTagRefused);
export const ACCOUNTS_SAVE_TAG_REFUSED_REQUEST = readAccountInput(accountsSaveTagRefusedRequest);
export const ACCOUNTS_SIMILAR_DUPLICATE = readAccountMatches(accountsSimilarTagDuplicate);
export const ACCOUNTS_SIMILAR_DUPLICATE_REQUEST = readAccountsSimilarInput(
  accountsSimilarTagDuplicateRequest,
);
export const ACCOUNTS_SIMILAR_OTHER = readAccountMatches(accountsSimilarTagOther);
export const ACCOUNTS_SIMILAR_OTHER_REQUEST = readAccountsSimilarInput(
  accountsSimilarTagOtherRequest,
);
export const PARTY_CARD_TAG = readPartyCard(assistPartyCardTag);
export const SEARCH_ACCOUNTS_TAG = readAccountHits(searchAccountsTag);
export const SEARCH_ACCOUNTS_TAG_REQUEST = readSearchAccountsInput(searchAccountsTagRequest);
export const SEARCH_GLOBAL_TAG = readGlobalHits(searchGlobalTag);
export const SEARCH_GLOBAL_TAG_NUMBER = readGlobalHits(searchGlobalTagNumber);
export const SEARCH_GLOBAL_TAG_REQUEST = readSearchGlobalInput(searchGlobalTagRequest);
export const SETTINGS_GET_CONTACTS = readSettings(settingsGetContacts);
export const SETTINGS_SET_FIRM_CONTACTS = readFirmSettings(settingsSetFirmContacts);
export const SETTINGS_SET_FIRM_CONTACTS_REQUEST = readSetFirmInput(settingsSetFirmContactsRequest);
export const SETTINGS_SET_FIRM_CONTACTS_REFUSED = readUiError(settingsSetFirmContactsRefused);
export const SETTINGS_SET_FIRM_CONTACTS_REFUSED_REQUEST = readSetFirmInput(
  settingsSetFirmContactsRefusedRequest,
);
export const PRINT_HEADER_UR = readPrintFrame(reportsPrintHeaderUr);
export const PRINT_HEADER_UR_REQUEST = readPrintFrameInput(reportsPrintHeaderUrRequest);
export const PRINT_HEADER_EN = readPrintFrame(reportsPrintHeaderEn);
export const PRINT_HEADER_EN_REQUEST = readPrintFrameInput(reportsPrintHeaderEnRequest);
export const COLLECTION_UR = readCollectionPrintData(reportsCollectionPrintDataUr);
export const COLLECTION_UR_REQUEST = readCollectionPrintDataInput(
  reportsCollectionPrintDataUrRequest,
);
export const COLLECTION_EN = readCollectionPrintData(reportsCollectionPrintDataEn);
export const COLLECTION_EN_REQUEST = readCollectionPrintDataInput(
  reportsCollectionPrintDataEnRequest,
);
export const COLLECTION_EMPTY = readCollectionPrintData(reportsCollectionPrintDataEmpty);
export const COLLECTION_EMPTY_REQUEST = readCollectionPrintDataInput(
  reportsCollectionPrintDataEmptyRequest,
);
export const BIKRI_TAG_SELLER_BILL_UR = readPrintData(reportsBikriPrintDataTagSellerBillUr);
export const BIKRI_TAG_BUYER_SLIPS_UR = readPrintData(reportsBikriPrintDataTagBuyerSlipsUr);
export const BIKRI_TAG_BUYER_SLIPS_EN = readPrintData(reportsBikriPrintDataTagBuyerSlipsEn);
