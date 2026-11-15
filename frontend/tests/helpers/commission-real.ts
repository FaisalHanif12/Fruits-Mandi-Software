// The real answers and requests of the backend for plan 77 (the commission page with a period and a party), worked
// day, October 2026. The backend wrote them in its own test (backend/crates/services/tests/fixtures/
// dashboard_commission_*.json); they are copied here, never edited, and read as the generated types. Money in
// them is paisa, as on the wire. A reader takes `unknown`, because the money in a binding is a branded type that a
// plain number is not.

import type { CommissionDay } from '../../src/bindings/CommissionDay';
import type { CommissionMonth } from '../../src/bindings/CommissionMonth';
import type { CommissionMonthInput } from '../../src/bindings/CommissionMonthInput';
import type { TablePrintData } from '../../src/bindings/TablePrintData';
import dayRahim from '../fixtures/dashboard_commission_day_seller_rahim.json';
import buyerAkram from '../fixtures/dashboard_commission_month_buyer_akram.json';
import buyerAkramRequest from '../fixtures/dashboard_commission_month_buyer_akram_request.json';
import buyerShahid from '../fixtures/dashboard_commission_month_buyer_shahid.json';
import buyerShahidRequest from '../fixtures/dashboard_commission_month_buyer_shahid_request.json';
import rangeShop from '../fixtures/dashboard_commission_month_range_shop.json';
import rangeShopRequest from '../fixtures/dashboard_commission_month_range_shop_request.json';
import sellerRahim from '../fixtures/dashboard_commission_month_seller_rahim.json';
import sellerRahimRequest from '../fixtures/dashboard_commission_month_seller_rahim_request.json';
import yearRahim from '../fixtures/dashboard_commission_month_year_rahim.json';
import yearRahimRequest from '../fixtures/dashboard_commission_month_year_rahim_request.json';
import printSellerEn from '../fixtures/dashboard_commission_print_data_seller_en.json';

function readMonth(value: unknown): CommissionMonth {
  return value as CommissionMonth;
}
function readInput(value: unknown): CommissionMonthInput {
  return value as CommissionMonthInput;
}

export const COMMISSION_SELLER_RAHIM = readMonth(sellerRahim);
export const COMMISSION_SELLER_RAHIM_REQUEST = readInput(sellerRahimRequest);
export const COMMISSION_BUYER_AKRAM = readMonth(buyerAkram);
export const COMMISSION_BUYER_AKRAM_REQUEST = readInput(buyerAkramRequest);
export const COMMISSION_BUYER_SHAHID = readMonth(buyerShahid);
export const COMMISSION_BUYER_SHAHID_REQUEST = readInput(buyerShahidRequest);
export const COMMISSION_YEAR_RAHIM = readMonth(yearRahim);
export const COMMISSION_YEAR_RAHIM_REQUEST = readInput(yearRahimRequest);
export const COMMISSION_RANGE_SHOP = readMonth(rangeShop);
export const COMMISSION_RANGE_SHOP_REQUEST = readInput(rangeShopRequest);
export const COMMISSION_DAY_SELLER_RAHIM = dayRahim as unknown as CommissionDay;
export const COMMISSION_PRINT_SELLER_EN = printSellerEn as unknown as TablePrintData;
