// The real answers and requests of the backend for the party ledgers screen (plan 66, PRD FR-12.43):
// the buyers and the sellers after Events 1 to 5 (E2E-67), and the statements of اکرم, a buyer, and
// رحیم, a seller. The backend wrote them in its own test (backend/crates/services/tests/fixtures);
// they are copied here by `scripts/copy-backend-fixtures.mjs party_ledgers_`, never edited, and read
// as the generated types. Money in them is paisa, as on the wire. A reader takes `unknown`, because
// the money in a binding is a branded type that a plain number is not.

import type { PartyLedgersList } from '../../src/bindings/PartyLedgersList';
import type { PartyLedgersListInput } from '../../src/bindings/PartyLedgersListInput';
import type { PartyLedgerStatement } from '../../src/bindings/PartyLedgerStatement';
import type { PartyLedgerStatementInput } from '../../src/bindings/PartyLedgerStatementInput';
import listBuyers from '../fixtures/party_ledgers_list_buyers.json';
import listBuyersRequest from '../fixtures/party_ledgers_list_buyers_request.json';
import listSellers from '../fixtures/party_ledgers_list_sellers.json';
import listSellersRequest from '../fixtures/party_ledgers_list_sellers_request.json';
import statementAkram from '../fixtures/party_ledgers_statement_akram.json';
import statementAkramRequest from '../fixtures/party_ledgers_statement_akram_request.json';
import statementRahim from '../fixtures/party_ledgers_statement_rahim.json';
import statementRahimRequest from '../fixtures/party_ledgers_statement_rahim_request.json';

function readList(value: unknown): PartyLedgersList {
  return value as PartyLedgersList;
}
function readListInput(value: unknown): PartyLedgersListInput {
  return value as PartyLedgersListInput;
}
function readStatement(value: unknown): PartyLedgerStatement {
  return value as PartyLedgerStatement;
}
function readStatementInput(value: unknown): PartyLedgerStatementInput {
  return value as PartyLedgerStatementInput;
}

/** اکرم ٹول ٹیکس 2,880 بنام and ارشد ملتانی 18,684 بنام; total debit 21,564, credit 0. */
export const LEDGER_BUYERS = readList(listBuyers);
export const LEDGER_BUYERS_REQUEST = readListInput(listBuyersRequest);
/** رحیم 0 and کریم 10,160 جمع; total debit 0, credit 10,160. */
export const LEDGER_SELLERS = readList(listSellers);
export const LEDGER_SELLERS_REQUEST = readListInput(listSellersRequest);
/** اکرم on 10-10-2026: previous 0, charged 12,880, paid 10,000, balance 2,880 بنام. */
export const STATEMENT_AKRAM = readStatement(statementAkram);
export const STATEMENT_AKRAM_REQUEST = readStatementInput(statementAkramRequest);
/** رحیم: 10-10-2026 (previous 15,000 بنام, credited 24,316, paid 9,316, balance 0) and 01-09-2026 (the advance). */
export const STATEMENT_RAHIM = readStatement(statementRahim);
export const STATEMENT_RAHIM_REQUEST = readStatementInput(statementRahimRequest);
