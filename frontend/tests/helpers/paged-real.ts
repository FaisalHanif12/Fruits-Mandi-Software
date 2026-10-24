// The real paged answers and requests of the backend for plan 17b (PRD FR-19.11, Code Standards
// B31): the masters, the rules and the learned words, and the second pages of the voucher list. The
// backend wrote them in its own tests (backend/crates/services/tests/fixtures); they are copied
// here, never edited, and read as the generated types. A request file is exactly the `input` the
// command took, with `offset` and `limit`.

import mastersListItems from '../fixtures/masters_list_items.json';
import mastersListItemsPage2 from '../fixtures/masters_list_items_page_2.json';
import mastersListItemsPage2Request from '../fixtures/masters_list_items_page_2_request.json';
import mastersListItemsRequest from '../fixtures/masters_list_items_request.json';
import rulesListAll from '../fixtures/rules_list_all.json';
import rulesListAllPage2 from '../fixtures/rules_list_all_page_2.json';
import rulesListAllPage2Request from '../fixtures/rules_list_all_page_2_request.json';
import rulesListAllRequest from '../fixtures/rules_list_all_request.json';
import translitPairsList from '../fixtures/translit_pairs_list.json';
import translitPairsListPage2 from '../fixtures/translit_pairs_list_page_2.json';
import translitPairsListPage2Request from '../fixtures/translit_pairs_list_page_2_request.json';
import translitPairsListRequest from '../fixtures/translit_pairs_list_request.json';
import vouchersListAllPage2 from '../fixtures/vouchers_list_all_page_2.json';
import vouchersListAllPage2Request from '../fixtures/vouchers_list_all_page_2_request.json';
import type { MastersList } from '../../src/bindings/MastersList';
import type { MastersListInput } from '../../src/bindings/MastersListInput';
import type { PairsList } from '../../src/bindings/PairsList';
import type { PairsListInput } from '../../src/bindings/PairsListInput';
import type { RulesList } from '../../src/bindings/RulesList';
import type { RulesListInput } from '../../src/bindings/RulesListInput';
import type { VouchersList } from '../../src/bindings/VouchersList';
import type { VouchersListInput } from '../../src/bindings/VouchersListInput';

// A fixture file is the backend's own JSON; each is read as its generated type.
function readMastersList(value: unknown): MastersList {
  return value as MastersList;
}
function readMastersListInput(value: unknown): MastersListInput {
  return value as MastersListInput;
}
function readRulesList(value: unknown): RulesList {
  return value as RulesList;
}
function readRulesListInput(value: unknown): RulesListInput {
  return value as RulesListInput;
}
function readPairsList(value: unknown): PairsList {
  return value as PairsList;
}
function readPairsListInput(value: unknown): PairsListInput {
  return value as PairsListInput;
}
function readVouchersList(value: unknown): VouchersList {
  return value as VouchersList;
}
function readVouchersListInput(value: unknown): VouchersListInput {
  return value as VouchersListInput;
}

export const MASTERS_ITEMS: MastersList = readMastersList(mastersListItems);
export const MASTERS_ITEMS_PAGE_2: MastersList = readMastersList(mastersListItemsPage2);
export const REQUEST_MASTERS_ITEMS: MastersListInput =
  readMastersListInput(mastersListItemsRequest);
export const REQUEST_MASTERS_ITEMS_PAGE_2: MastersListInput = readMastersListInput(
  mastersListItemsPage2Request,
);

export const RULES_ALL: RulesList = readRulesList(rulesListAll);
export const RULES_ALL_PAGE_2: RulesList = readRulesList(rulesListAllPage2);
export const REQUEST_RULES_ALL: RulesListInput = readRulesListInput(rulesListAllRequest);
export const REQUEST_RULES_ALL_PAGE_2: RulesListInput =
  readRulesListInput(rulesListAllPage2Request);

export const PAIRS: PairsList = readPairsList(translitPairsList);
export const PAIRS_PAGE_2: PairsList = readPairsList(translitPairsListPage2);
export const REQUEST_PAIRS: PairsListInput = readPairsListInput(translitPairsListRequest);
export const REQUEST_PAIRS_PAGE_2: PairsListInput = readPairsListInput(
  translitPairsListPage2Request,
);

export const VOUCHERS_ALL_PAGE_2: VouchersList = readVouchersList(vouchersListAllPage2);
export const REQUEST_VOUCHERS_ALL_PAGE_2: VouchersListInput = readVouchersListInput(
  vouchersListAllPage2Request,
);
