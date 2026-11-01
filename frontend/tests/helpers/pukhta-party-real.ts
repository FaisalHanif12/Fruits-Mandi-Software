// The real answers and requests of the backend for plan 47 (a پختہ party's own account): the next
// code of the پختہ series for each side, the account saved with the mark and refused for a wrong
// type or a change, the list under the پختہ filter, the search, the party card and the answers of
// the short code in a buyer, a seller and an any-account field. The backend wrote them in its own
// tests (backend/crates/services/tests/fixtures); they are copied here, never edited, and read as
// the generated types. The story: نذیر (Nazir), a پختہ buyer, 0032; کمال (Kamal), a پختہ seller, 0033;
// an ordinary buyer طارق محمود (Tariq Mehmood), 0022, who shares his running number 2 with نذیر.

import type { AccountHit } from '../../src/bindings/AccountHit';
import type { AccountInput } from '../../src/bindings/AccountInput';
import type { AccountRow } from '../../src/bindings/AccountRow';
import type { AccountsList } from '../../src/bindings/AccountsList';
import type { AccountsNextCodeInput } from '../../src/bindings/AccountsNextCodeInput';
import type { PartyCard } from '../../src/bindings/PartyCard';
import type { ResolvedCode } from '../../src/bindings/ResolvedCode';
import type { ResolveCodeInput } from '../../src/bindings/ResolveCodeInput';
import type { SearchAccountsInput } from '../../src/bindings/SearchAccountsInput';
import type { UiError } from '../../src/bindings/UiError';

import accountsListPukhta from '../fixtures/accounts_list_pukhta.json';
import accountsListPukhtaRequest from '../fixtures/accounts_list_pukhta_request.json';
import accountsNextCodePukhtaAfterTwo from '../fixtures/accounts_next_code_pukhta_after_two.json';
import accountsNextCodePukhtaAfterTwoRequest from '../fixtures/accounts_next_code_pukhta_after_two_request.json';
import accountsNextCodePukhtaBuyer from '../fixtures/accounts_next_code_pukhta_buyer.json';
import accountsNextCodePukhtaBuyerRequest from '../fixtures/accounts_next_code_pukhta_buyer_request.json';
import accountsNextCodePukhtaSeller from '../fixtures/accounts_next_code_pukhta_seller.json';
import accountsNextCodePukhtaSellerRequest from '../fixtures/accounts_next_code_pukhta_seller_request.json';
import accountsSavePukhtaBuyer from '../fixtures/accounts_save_pukhta_buyer.json';
import accountsSavePukhtaBuyerRequest from '../fixtures/accounts_save_pukhta_buyer_request.json';
import accountsSavePukhtaPartyChange from '../fixtures/accounts_save_pukhta_party_change.json';
import accountsSavePukhtaPartyChangeRequest from '../fixtures/accounts_save_pukhta_party_change_request.json';
import accountsSavePukhtaPartyWrongType from '../fixtures/accounts_save_pukhta_party_wrong_type.json';
import accountsSavePukhtaPartyWrongTypeRequest from '../fixtures/accounts_save_pukhta_party_wrong_type_request.json';
import accountsSavePukhtaSeller from '../fixtures/accounts_save_pukhta_seller.json';
import accountsSavePukhtaSellerRequest from '../fixtures/accounts_save_pukhta_seller_request.json';
import assistPartyCardPukhtaBuyer from '../fixtures/assist_party_card_pukhta_buyer.json';
import searchAccountsPukhtaBuyer from '../fixtures/search_accounts_pukhta_buyer.json';
import searchAccountsPukhtaBuyerRequest from '../fixtures/search_accounts_pukhta_buyer_request.json';
import searchResolveCodeAnyShort2Several from '../fixtures/search_resolve_code_any_short_2_several.json';
import searchResolveCodeAnyShort2SeveralRequest from '../fixtures/search_resolve_code_any_short_2_several_request.json';
import searchResolveCodeBuyerFull0032 from '../fixtures/search_resolve_code_buyer_full_0032.json';
import searchResolveCodeBuyerFull0032Request from '../fixtures/search_resolve_code_buyer_full_0032_request.json';
import searchResolveCodeBuyerShort2Several from '../fixtures/search_resolve_code_buyer_short_2_several.json';
import searchResolveCodeBuyerShort2SeveralRequest from '../fixtures/search_resolve_code_buyer_short_2_several_request.json';
import searchResolveCodeSellerShort2Unknown from '../fixtures/search_resolve_code_seller_short_2_unknown.json';
import searchResolveCodeSellerShort2UnknownRequest from '../fixtures/search_resolve_code_seller_short_2_unknown_request.json';
import searchResolveCodeSellerShort3Found from '../fixtures/search_resolve_code_seller_short_3_found.json';
import searchResolveCodeSellerShort3FoundRequest from '../fixtures/search_resolve_code_seller_short_3_found_request.json';

// Each fixture is the backend's own JSON, read as its generated type. A reader takes `unknown`,
// because the money in a binding is a branded type that a plain number is not.
const asRow = (value: unknown): AccountRow => value as AccountRow;
const asInput = (value: unknown): AccountInput => value as AccountInput;
const asList = (value: unknown): AccountsList => value as AccountsList;
const asNextCode = (value: unknown): AccountsNextCodeInput => value as AccountsNextCodeInput;
const asCard = (value: unknown): PartyCard => value as PartyCard;
const asResolved = (value: unknown): ResolvedCode => value as ResolvedCode;
const asResolveInput = (value: unknown): ResolveCodeInput => value as ResolveCodeInput;
const asSearchInput = (value: unknown): SearchAccountsInput => value as SearchAccountsInput;
const asHits = (value: unknown): AccountHit[] => value as AccountHit[];
const asError = (value: unknown): UiError => value as UiError;

export const NEXT_CODE_BUYER = accountsNextCodePukhtaBuyer;
export const NEXT_CODE_BUYER_REQUEST = asNextCode(accountsNextCodePukhtaBuyerRequest);
export const NEXT_CODE_SELLER = accountsNextCodePukhtaSeller;
export const NEXT_CODE_SELLER_REQUEST = asNextCode(accountsNextCodePukhtaSellerRequest);
export const NEXT_CODE_AFTER_TWO = accountsNextCodePukhtaAfterTwo;
export const NEXT_CODE_AFTER_TWO_REQUEST = asNextCode(accountsNextCodePukhtaAfterTwoRequest);

export const SAVE_BUYER = asRow(accountsSavePukhtaBuyer);
export const SAVE_BUYER_REQUEST = asInput(accountsSavePukhtaBuyerRequest);
export const SAVE_SELLER = asRow(accountsSavePukhtaSeller);
export const SAVE_SELLER_REQUEST = asInput(accountsSavePukhtaSellerRequest);
export const SAVE_WRONG_TYPE = asError(accountsSavePukhtaPartyWrongType);
export const SAVE_WRONG_TYPE_REQUEST = asInput(accountsSavePukhtaPartyWrongTypeRequest);
export const SAVE_CHANGE = asError(accountsSavePukhtaPartyChange);
export const SAVE_CHANGE_REQUEST = asInput(accountsSavePukhtaPartyChangeRequest);

export const LIST_PUKHTA = asList(accountsListPukhta);
export const LIST_PUKHTA_REQUEST = accountsListPukhtaRequest;

export const SEARCH_BUYER = asHits(searchAccountsPukhtaBuyer);
export const SEARCH_BUYER_REQUEST = asSearchInput(searchAccountsPukhtaBuyerRequest);
export const PARTY_CARD_BUYER = asCard(assistPartyCardPukhtaBuyer);

export const RESOLVE_BUYER_SHORT_2 = asResolved(searchResolveCodeBuyerShort2Several);
export const RESOLVE_BUYER_SHORT_2_REQUEST = asResolveInput(
  searchResolveCodeBuyerShort2SeveralRequest,
);
export const RESOLVE_ANY_SHORT_2 = asResolved(searchResolveCodeAnyShort2Several);
export const RESOLVE_ANY_SHORT_2_REQUEST = asResolveInput(searchResolveCodeAnyShort2SeveralRequest);
export const RESOLVE_SELLER_SHORT_2 = asResolved(searchResolveCodeSellerShort2Unknown);
export const RESOLVE_SELLER_SHORT_2_REQUEST = asResolveInput(
  searchResolveCodeSellerShort2UnknownRequest,
);
export const RESOLVE_SELLER_SHORT_3 = asResolved(searchResolveCodeSellerShort3Found);
export const RESOLVE_SELLER_SHORT_3_REQUEST = asResolveInput(
  searchResolveCodeSellerShort3FoundRequest,
);
export const RESOLVE_BUYER_FULL = asResolved(searchResolveCodeBuyerFull0032);
export const RESOLVE_BUYER_FULL_REQUEST = asResolveInput(searchResolveCodeBuyerFull0032Request);
