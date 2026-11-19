// The hidden print window's page for the prints of plan 78: `#/print?print=<kind>&language=<ur|en>&paper=<a4|a5|
// mm80>&input=<JSON>&job=<job>`. The backend opens a window with this address, waits for `print-ready-<job>` and
// then prints it or saves it as a PDF. A page that cannot be read says `print-error-<job>` instead. Like the other
// print windows it sits outside the app shell, and its capability allows the print-data command of the kind and
// `reports_print_header` only. The kinds: the buyer slips, the seller bills, the whole sale print, ادھاری تفصیل,
// the R5 sheet, a statement with a row for each entry, the buyer dues, the commission page and a partner's month.
// Every figure is written by the backend; the documents lay them out and add up nothing.

import { Localized } from '@fluent/react';
import { useQuery } from '@tanstack/react-query';
import { useRouterState } from '@tanstack/react-router';
import { useEffect, useMemo } from 'react';
import type { ReportId } from '../../bindings/ReportId';
import type { TablePrintData } from '../../bindings/TablePrintData';
import { emitEvent } from '../../shared/api';
import {
  PrintView,
  jobOf,
  printErrorEvent,
  printReadyEvent,
  type PrintFailed,
  type PrintReady,
} from '../../shared/print';
import { ErrorState } from '../../shared/ui';
import {
  dashboardAdvancesOpenPrintData,
  dashboardBuyerDuesPrintData,
  dashboardCashSalesDayPrintData,
  dashboardCashSalesMonthPrintData,
  dashboardCommissionPrintData,
  dashboardLentOpenPrintData,
  dashboardSellerDuesPrintData,
  dashboardTotalDuesPrintData,
} from '../../features/dashboard';
import { partnersStatementMonthPrintData } from '../../features/partners';
import { partyLedgersStatementRowsPrintData } from '../../features/party_ledgers';
import {
  DaySlipsDocument,
  ReportPrintDocument,
  SalePrintDocument,
  printTableOf,
  reportsAdhariSheetPrintData,
  reportsBuyerParchiPrintData,
  reportsCollectionSheetPrintData,
  reportsSalePrintData,
  reportsSellerBillsPrintData,
} from '../../features/reports';
import { parseKindPrintParams, type KindPrintParams } from './params';

/** The parameters of the three prints that are made of the slips of a day. */
type DaySlipsParams = Extract<KindPrintParams, { kind: 'parchi' | 'seller-bills' | 'sale' }>;

interface WindowCallbacks {
  onReady: (ready: PrintReady) => void;
  onFailed: (codes: string[]) => void;
}

/** What the print window loads the fonts for: the firm and the mandi of the header. */
function sampleOf(frame: { header: { firm: { name: string; mandi: string } } }): string {
  return `${frame.header.firm.name} ${frame.header.firm.mandi}`;
}

/** The slips of a day: the buyer slips or the seller bills. */
function SlipsBody({
  params,
  variant,
  callbacks,
}: {
  params: DaySlipsParams;
  variant: 'parchi' | 'seller-bills';
  callbacks: WindowCallbacks;
}) {
  const { input } = params;
  const query = useQuery({
    queryKey: ['print-kind', params.kind, params.input],
    queryFn: () =>
      variant === 'parchi'
        ? reportsBuyerParchiPrintData(input)
        : reportsSellerBillsPrintData(input),
    select: (data) => ({ ...data, language: data.frame.language }),
    staleTime: Infinity,
    gcTime: 0,
  });
  return (
    <PrintView
      query={query}
      describe={(data) => ({
        empty: (variant === 'parchi' ? data.buyers : data.sellers).length === 0,
        sample: sampleOf(data.frame),
      })}
      onReady={callbacks.onReady}
      onFailed={callbacks.onFailed}
      render={(data, ref) => <DaySlipsDocument data={data} variant={variant} applyRoot ref={ref} />}
    />
  );
}

/** The whole sale print: the sellers table, then the buyers table. */
function SaleBody({ params, callbacks }: { params: DaySlipsParams; callbacks: WindowCallbacks }) {
  const { input } = params;
  const query = useQuery({
    queryKey: ['print-kind', params.kind, params.input],
    queryFn: () => reportsSalePrintData(input),
    select: (data) => ({ ...data, language: data.frame.language }),
    staleTime: Infinity,
    gcTime: 0,
  });
  return (
    <PrintView
      query={query}
      describe={(data) => ({
        empty: data.sellers.rows.length === 0 && data.buyers.rows.length === 0,
        sample: sampleOf(data.frame),
      })}
      onReady={callbacks.onReady}
      onFailed={callbacks.onFailed}
      render={(data, ref) => <SalePrintDocument data={data} applyRoot ref={ref} />}
    />
  );
}

/** A table print: ادھاری تفصیل, the R5 sheet, a statement, the buyer dues, the commission page, a partner's month. */
function TableBody({
  params,
  read,
  report,
  callbacks,
}: {
  params: KindPrintParams;
  read: () => Promise<TablePrintData>;
  report?: ReportId;
  callbacks: WindowCallbacks;
}) {
  const query = useQuery({
    queryKey: ['print-kind', params.kind, params.input],
    queryFn: read,
    select: (data) => ({ ...data, language: data.frame.language }),
    staleTime: Infinity,
    gcTime: 0,
  });
  return (
    <PrintView
      query={query}
      describe={(data) => ({ empty: data.rows.length === 0, sample: sampleOf(data.frame) })}
      onReady={callbacks.onReady}
      onFailed={callbacks.onFailed}
      render={(data, ref) => (
        <ReportPrintDocument
          data={printTableOf(data, report)}
          frame={data.frame}
          applyRoot
          ref={ref}
        />
      )}
    />
  );
}

function Body({ params, callbacks }: { params: KindPrintParams; callbacks: WindowCallbacks }) {
  switch (params.kind) {
    case 'parchi':
      return <SlipsBody params={params} variant="parchi" callbacks={callbacks} />;
    case 'seller-bills':
      return <SlipsBody params={params} variant="seller-bills" callbacks={callbacks} />;
    case 'sale':
      return <SaleBody params={params} callbacks={callbacks} />;
    case 'adhari':
      return (
        <TableBody
          params={params}
          report="R5"
          callbacks={callbacks}
          read={() => reportsAdhariSheetPrintData(params.input)}
        />
      );
    case 'collection-sheet':
      return (
        <TableBody
          params={params}
          report="R5"
          callbacks={callbacks}
          read={() => reportsCollectionSheetPrintData(params.input)}
        />
      );
    case 'statement-rows':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => partyLedgersStatementRowsPrintData(params.input)}
        />
      );
    case 'buyer-dues':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => dashboardBuyerDuesPrintData(params.input)}
        />
      );
    case 'commission':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => dashboardCommissionPrintData(params.input)}
        />
      );
    case 'partner-month':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => partnersStatementMonthPrintData(params.input)}
        />
      );
    case 'seller-dues':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => dashboardSellerDuesPrintData(params.input)}
        />
      );
    case 'total-dues':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => dashboardTotalDuesPrintData(params.input)}
        />
      );
    case 'lent-open':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => dashboardLentOpenPrintData(params.input)}
        />
      );
    case 'advances-open':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => dashboardAdvancesOpenPrintData(params.input)}
        />
      );
    case 'cash-sales-month':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => dashboardCashSalesMonthPrintData(params.input)}
        />
      );
    case 'cash-sales-day':
      return (
        <TableBody
          params={params}
          callbacks={callbacks}
          read={() => dashboardCashSalesDayPrintData(params.input)}
        />
      );
  }
}

export function KindPrintPage() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  const params = useMemo(() => parseKindPrintParams(searchStr), [searchStr]);
  const job = jobOf(searchStr);

  // An address that is not a print address is a failed print, told once.
  useEffect(() => {
    if (params === null && job !== null) {
      const failed: PrintFailed = { codes: ['ERR_INTERNAL'] };
      void emitEvent(printErrorEvent(job), failed);
    }
  }, [params, job]);

  if (params === null) {
    return (
      <ErrorState
        title={<Localized id="print-preview-failed" />}
        error={{ ok: false, errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }], ref: '' }}
        onRetry={() => {
          window.location.reload();
        }}
      />
    );
  }
  const callbacks: WindowCallbacks = {
    onReady: (ready) => {
      void emitEvent(printReadyEvent(params.job), ready);
    },
    onFailed: (codes) => {
      const failed: PrintFailed = { codes };
      void emitEvent(printErrorEvent(params.job), failed);
    },
  };
  return <Body params={params} callbacks={callbacks} />;
}
