// The print commands as hooks (Code Standards F6).

import { keepPreviousData, useInfiniteQuery, useMutation, useQuery } from '@tanstack/react-query';
import { useDebouncedValue } from '../../shared/hooks';
import type { BikriPrintDataInput } from '../../bindings/BikriPrintDataInput';
import type { CollectionPrintDataInput } from '../../bindings/CollectionPrintDataInput';
import type { PrintCollectionInput } from '../../bindings/PrintCollectionInput';
import type { PrintFrameInput } from '../../bindings/PrintFrameInput';
import type { Language } from '../../bindings/Language';
import type { PrintFact } from '../../bindings/PrintFact';
import type { PrintInput } from '../../bindings/PrintInput';
import type { PrintReportInput } from '../../bindings/PrintReportInput';
import type { PrintVoucherInput } from '../../bindings/PrintVoucherInput';
import type { ReportFilters } from '../../bindings/ReportFilters';
import type { ReportId } from '../../bindings/ReportId';
import type { ReportPrintInput } from '../../bindings/ReportPrintInput';
import type { VoucherPrintDataInput } from '../../bindings/VoucherPrintDataInput';
import {
  reportsBikriPrintData,
  reportsBuyerBillsPrintData,
  reportsCatalog,
  reportsFindPeople,
  reportsListPrinters,
  reportsPrint,
  reportsPrintData,
  reportsPrintReport,
  reportsCollectionPrintData,
  reportsPrintCollection,
  reportsPrintHeader,
  reportsPrintVoucher,
  reportsRun,
  reportsVoucherPrintData,
} from './api';

/** The print data of one document. A print is a snapshot, so it is never read again by itself. */
export function useBikriPrintData(input: BikriPrintDataInput | null) {
  return useQuery({
    queryKey: ['reports', 'bikri-print-data', input],
    queryFn: () => {
      if (input === null) throw new Error('no print input');
      return reportsBikriPrintData(input);
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** The printers of this computer (Windows). An empty list is an answer, not an error. */
export function usePrinters() {
  return useQuery({ queryKey: ['reports', 'printers'], queryFn: reportsListPrinters });
}

/** Print or save as PDF. The caller shows the outcome or the error. */
export function usePrintDocs() {
  return useMutation({ mutationFn: (input: PrintInput) => reportsPrint(input) });
}

/** The print data of the receipt slip of one saved voucher. A print is a snapshot, like the above. */
export function useVoucherPrintData(input: VoucherPrintDataInput | null) {
  return useQuery({
    queryKey: ['reports', 'voucher-print-data', input],
    queryFn: () => {
      if (input === null) throw new Error('no print input');
      return reportsVoucherPrintData(input);
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** Print the receipt slip of a voucher, or save it as PDF. The caller shows the outcome or the error. */
export function usePrintVoucher() {
  return useMutation({ mutationFn: (input: PrintVoucherInput) => reportsPrintVoucher(input) });
}

// ---- Plan 20: the report engine. Run B, D5 early track. ----

/** A page of a report (Code Standards B31, NFR-SPD-24). */
export const REPORT_PAGE_SIZE = 100;

/** The catalogue changes only with a new version of the app, so it is read once. */
export function useReportCatalog(enabled = true) {
  return useQuery({
    queryKey: ['reports_catalog'],
    queryFn: reportsCatalog,
    staleTime: Infinity,
    enabled,
  });
}

/**
 * A report, page by page. `filters` null means the report cannot run yet (a party is still to be
 * picked). The totals and the lines above the table come with every page and are the same for all of
 * them (they are over the whole filter), so the screen reads them from the last page. While the
 * filters change, the rows already on screen stay until the new ones come (no flicker).
 */
export function useReportPages(id: ReportId, filters: ReportFilters | null) {
  return useInfiniteQuery({
    queryKey: ['reports_run', id, filters],
    queryFn: ({ pageParam }) => {
      if (filters === null) throw new Error('no filters');
      return reportsRun({ id, filters, offset: pageParam, limit: REPORT_PAGE_SIZE });
    },
    enabled: filters !== null,
    initialPageParam: 0,
    getNextPageParam: (last, pages) => {
      const loaded = pages.reduce((count, page) => count + page.rows.length, 0);
      return last.rows.length > 0 && loaded < last.total_count ? loaded : undefined;
    },
    placeholderData: keepPreviousData,
  });
}

/** The print data of a report with every row. A print is a snapshot: never read again by itself. */
export function useReportPrintData(input: ReportPrintInput | null) {
  return useQuery({
    queryKey: ['reports_print_data', input],
    queryFn: () => {
      if (input === null) throw new Error('no print input');
      return reportsPrintData(input);
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** The print data of the collection list of one day. A print is a snapshot, like the above. */
export function useCollectionPrintData(input: CollectionPrintDataInput | null) {
  return useQuery({
    queryKey: ['reports', 'collection-print-data', input],
    queryFn: () => {
      if (input === null) throw new Error('no print input');
      return reportsCollectionPrintData(input);
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** The print data of all the credit buyers' slips of R5: a snapshot, like the print above. */
export function useBuyerBillsPrintData(input: ReportPrintInput | null) {
  return useQuery({
    queryKey: ['reports_buyer_bills_print_data', input],
    queryFn: () => {
      if (input === null) throw new Error('no print input');
      return reportsBuyerBillsPrintData(input);
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** Print the collection list of a day, or save it as PDF. The caller shows the outcome or the error. */
export function usePrintCollection() {
  return useMutation({
    mutationFn: (input: PrintCollectionInput) => reportsPrintCollection(input),
  });
}

/** The shared header and footer of a report print (title and facts in, header and footer out). */
export function usePrintFrame(input: PrintFrameInput | null) {
  return useQuery({
    queryKey: ['reports', 'print-frame', input],
    queryFn: () => {
      if (input === null) throw new Error('no print input');
      return reportsPrintHeader(input);
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** Sends a report to the printer or saves it as a PDF. The caller shows the outcome or the error. */
export function usePrintReport() {
  return useMutation({ mutationFn: (input: PrintReportInput) => reportsPrintReport(input) });
}

/**
 * The page the hidden print window draws for a report (PRD FR-19.3): its print data and the shared
 * header and footer, read together so the window says "ready" only when both are there. The language
 * is the one the print was asked for.
 */
export function useReportPrintBundle(input: ReportPrintInput | null) {
  return useQuery({
    queryKey: ['reports', 'print-bundle', input],
    queryFn: async () => {
      if (input === null) throw new Error('no print input');
      const data = await reportsPrintData(input);
      const frame = await reportsPrintHeader({
        language: input.language,
        title_id: data.title_id,
        facts: data.filters,
      });
      return { language: frame.language, urdu_digits: frame.urdu_digits, data, frame };
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** The same for the slips of all the credit buyers of R5 ("Print All"). */
export function useSlipsPrintBundle(input: ReportPrintInput | null) {
  return useQuery({
    queryKey: ['reports', 'slips-print-bundle', input],
    queryFn: async () => {
      if (input === null) throw new Error('no print input');
      const data = await reportsBuyerBillsPrintData(input);
      const frame = await reportsPrintHeader({
        language: input.language,
        title_id: data.title_id,
        facts: data.filters,
      });
      return { language: frame.language, urdu_digits: frame.urdu_digits, data, frame };
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/**
 * The shared header and footer of a print whose data holds its title and its filters (every report and
 * analytics print): asked once the data is there, in the language the print was asked for, from that
 * title and those filters (PRD FR-19.13).
 */
export function usePrintFrameOf(
  language: Language,
  source: { title_id: string; filters: PrintFact[] } | undefined,
) {
  return usePrintFrame(
    source === undefined ? null : { language, title_id: source.title_id, facts: source.filters },
  );
}

/** How long the text of a person search rests before the backend is asked (Code Standards F35). */
export const PEOPLE_SEARCH_DELAY_MS = 120;

/** How many one-time people a search lists. The list is short on purpose: type more to narrow. */
export const PEOPLE_RESULT_LIMIT = 10;

/**
 * The one-time people that match the typed text (PRD FR-5.4, FR-13.9). Nothing is asked for empty
 * text, and the list stays on screen while the next answer is on its way, so it never flashes empty.
 * The text is sent as typed; a search never converts it to Urdu (Code Standards F41).
 */
export function usePeopleSearch(text: string, enabled = true) {
  const settled = useDebouncedValue(text.trim(), PEOPLE_SEARCH_DELAY_MS);
  return useQuery({
    queryKey: ['reports_find_people', settled],
    queryFn: () => reportsFindPeople({ text: settled, limit: PEOPLE_RESULT_LIMIT }),
    enabled: enabled && settled !== '',
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
}
