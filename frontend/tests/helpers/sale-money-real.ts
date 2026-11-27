// The real answers and requests of the backend for plan 99 (the garden's sale money, the two lists of
// goods arrived, and a partner's open gardens and trips by month). The backend wrote them in its own
// tests; they are copied here, never edited, and read as the generated types. No test runner in it.

import type { ArrivalsList } from '../../src/bindings/ArrivalsList';
import type { ArrivalsListInput } from '../../src/bindings/ArrivalsListInput';
import type { GardensSaleMoney } from '../../src/bindings/GardensSaleMoney';
import type { GardensSaleMoneyInput } from '../../src/bindings/GardensSaleMoneyInput';
import type { PartnerOpenVentures } from '../../src/bindings/PartnerOpenVentures';
import type { PartnersOpenVenturesInput } from '../../src/bindings/PartnersOpenVenturesInput';
import type { WindowsBoundsInput } from '../../src/bindings/WindowsBoundsInput';
import soldOut from '../fixtures/gardens_arrivals_list_sold_out.json';
import soldOutRequest from '../fixtures/gardens_arrivals_list_sold_out_request.json';
import saleArrival from '../fixtures/gardens_sale_money_arrival.json';
import saleArrivalRequest from '../fixtures/gardens_sale_money_arrival_request.json';
import saleMonth from '../fixtures/gardens_sale_money_month.json';
import saleMonthRequest from '../fixtures/gardens_sale_money_month_request.json';
import saleSecondPage from '../fixtures/gardens_sale_money_second_page.json';
import saleSecondPageRequest from '../fixtures/gardens_sale_money_second_page_request.json';
import venturesLatest from '../fixtures/partners_open_ventures_latest.json';
import venturesLatestRequest from '../fixtures/partners_open_ventures_latest_request.json';
import venturesMonth from '../fixtures/partners_open_ventures_month.json';
import venturesMonthRequest from '../fixtures/partners_open_ventures_month_request.json';
import boundsGardenSalesRequest from '../fixtures/windows_bounds_garden_sales_request.json';
import boundsVenturesRequest from '../fixtures/windows_bounds_partner_ventures_request.json';

// The files are the wire shapes the backend sent; these casts are the one place that says so.
const sale = (value: unknown) => value as GardensSaleMoney;
const saleInput = (value: unknown) => value as GardensSaleMoneyInput;
const ventures = (value: unknown) => value as PartnerOpenVentures;
const venturesInput = (value: unknown) => value as PartnersOpenVenturesInput;

export const SALE_MONEY_MONTH = sale(saleMonth);
export const SALE_MONEY_MONTH_REQUEST = saleInput(saleMonthRequest);
export const SALE_MONEY_ARRIVAL = sale(saleArrival);
export const SALE_MONEY_ARRIVAL_REQUEST = saleInput(saleArrivalRequest);
export const SALE_MONEY_SECOND_PAGE = sale(saleSecondPage);
export const SALE_MONEY_SECOND_PAGE_REQUEST = saleInput(saleSecondPageRequest);
export const ARRIVALS_SOLD_OUT = soldOut as unknown as ArrivalsList;
export const ARRIVALS_SOLD_OUT_REQUEST = soldOutRequest as unknown as ArrivalsListInput;
export const OPEN_VENTURES_MONTH = ventures(venturesMonth);
export const OPEN_VENTURES_MONTH_REQUEST = venturesInput(venturesMonthRequest);
export const OPEN_VENTURES_LATEST = ventures(venturesLatest);
export const OPEN_VENTURES_LATEST_REQUEST = venturesInput(venturesLatestRequest);
export const BOUNDS_GARDEN_SALES_REQUEST =
  boundsGardenSalesRequest as unknown as WindowsBoundsInput;
export const BOUNDS_VENTURES_REQUEST = boundsVenturesRequest as unknown as WindowsBoundsInput;
