// What the top bar's search adds to the backend's accounts, بکری and vouchers (PRD FR-5.4): the
// one-time (پختہ کھاتہ) people, found by name or phone (`reports_find_people`), and the screens and
// reports of the app, found by their name in Urdu or English. The screens come from the router's
// own table (nav-routes.ts) and the report catalogue, so there is no second list to keep. It is the
// frame's code (Run B, plan 20, step F3); GlobalSearch only merges its rows into its own list.

import { Localized, useLocalization } from '@fluent/react';
import { useRouter } from '@tanstack/react-router';
import { useMemo } from 'react';
import type { PukhtaPersonHit } from '../../bindings/PukhtaPersonHit';
import type { ReportId } from '../../bindings/ReportId';
import {
  personFromHit,
  useOpenReport,
  usePeopleSearch,
  useReportCatalog,
} from '../../features/reports';
import { PukhtaPersonRow } from '../../features/pukhta_khata';
import { asScreenType } from '../../features/vouchers/screen-type';
import { bundles, LANGUAGES, type Language } from '../../i18n';
import { useLanguage } from '../../shared/i18n';
import { MAIN_ROUTES, MODULE_ROUTES } from '../nav-routes';

/** A screen or a report is listed after this many people; each kind shows only a few rows. */
const MAX_PEOPLE = 5;
const MAX_SCREENS = 5;

/** A one-time person found by name or phone. Opens his register (R9). */
export interface PersonRow {
  kind: 'person';
  /** His key, unique among the rows. */
  id: string;
  person: PukhtaPersonHit;
}

/** A screen of the main menu or the بکری side menu, or a report, found by its name. */
export interface ScreenRow {
  kind: 'screen';
  id: string;
  /** The Fluent message of its name. */
  titleId: string;
  /** Where it opens: a path, or a report. */
  target: { path: string } | { report: ReportId };
}

export type ExtraRow = PersonRow | ScreenRow;

/** A message in one language, whatever the app language is (a screen is found by either name). */
function messageIn(language: Language, id: string): string | null {
  const bundle = bundles[language];
  const pattern = bundle.getMessage(id)?.value;
  return pattern === undefined || pattern === null ? null : bundle.formatPattern(pattern);
}

/** True when the text is part of the name in either language (or starts a report's ID, "R18"). */
function matches(text: string, titleId: string, reportId: ReportId | null): boolean {
  const needle = text.trim().toLowerCase();
  if (needle === '') return false;
  if (reportId !== null && needle.length >= 2 && reportId.toLowerCase().startsWith(needle)) {
    return true;
  }
  return LANGUAGES.some((language) => {
    const title = messageIn(language, titleId);
    return title?.toLowerCase().includes(needle) === true;
  });
}

/** Every screen the search can open, in menu order: the main menu, the بکری side menu. */
function menuScreens(): ScreenRow[] {
  const main = (Object.keys(MAIN_ROUTES) as (keyof typeof MAIN_ROUTES)[]).map<ScreenRow>((id) => ({
    kind: 'screen',
    id: `screen:${id}`,
    titleId: `layout-nav-${id}`,
    target: { path: MAIN_ROUTES[id] },
  }));
  const module = (Object.keys(MODULE_ROUTES) as (keyof typeof MODULE_ROUTES)[]).map<ScreenRow>(
    (id) => ({
      kind: 'screen',
      id: `screen:${id}`,
      titleId: `layout-module-${id}`,
      target: { path: MODULE_ROUTES[id] },
    }),
  );
  return [...main, ...module].filter((row) => messageIn('en', row.titleId) !== null);
}

/**
 * The extra rows for the typed text: people first (the most likely thing someone types a name
 * for), then screens and reports. Empty text gives nothing, and nothing is asked of the backend
 * while `enabled` is false (no router around, so no row could be opened).
 */
export function useExtraSearch(text: string, enabled: boolean): ExtraRow[] {
  const typed = text.trim();
  const asking = enabled && typed !== '';
  const people = usePeopleSearch(typed, asking).data;
  const catalog = useReportCatalog(asking);
  const reports = catalog.data;
  return useMemo(() => {
    if (!asking) return [];
    const personRows: ExtraRow[] = (people ?? []).slice(0, MAX_PEOPLE).map((person) => ({
      kind: 'person',
      id: `person:${person.person_key}`,
      person,
    }));
    const reportRows: ScreenRow[] = (reports ?? []).map((entry) => ({
      kind: 'screen',
      id: `report:${entry.id}`,
      titleId: entry.title_id,
      target: { report: entry.id },
    }));
    const screenRows = [...menuScreens(), ...reportRows]
      .filter((row) =>
        matches(typed, row.titleId, 'report' in row.target ? row.target.report : null),
      )
      .slice(0, MAX_SCREENS);
    return [...personRows, ...screenRows];
  }, [asking, typed, people, reports]);
}

/** The words of a row in the box when it is highlighted or chosen. */
export function extraText(row: ExtraRow, language: Language): string {
  if (row.kind === 'person') return language === 'ur' ? row.person.name_ur : row.person.name_en;
  return messageIn(language, row.titleId) ?? row.titleId;
}

/** A result row: a one-time person with his phone and last day, or a screen with its kind. */
export function ExtraResult({ row }: { row: ExtraRow }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  if (row.kind === 'person') {
    return (
      <>
        <PukhtaPersonRow person={row.person} />
        <Localized id="report-search-person">
          <span className="type-chip text-ink-muted" />
        </Localized>
      </>
    );
  }
  const isReport = 'report' in row.target;
  return (
    <span className="flex items-baseline justify-between gap-3">
      <span className="type-field truncate">
        {extraText(row, language) || l10n.getString(row.titleId)}
      </span>
      <Localized id={isReport ? 'report-search-report' : 'report-search-screen'}>
        <span className="type-chip text-ink-muted" />
      </Localized>
    </span>
  );
}

/**
 * Opens what a row stands for. Null when there is no router to open it in (a test, a gallery page),
 * and then the search shows no extra rows to open.
 */
export function useOpenExtra(): ((row: ExtraRow) => void) | null {
  const openReport = useOpenReport();
  const router = useRouter({ warn: false }) as ReturnType<typeof useRouter> | undefined;
  return useMemo(() => {
    if (openReport === null || router === undefined) return null;
    return (row: ExtraRow) => {
      if (row.kind === 'person') {
        // His register (R9): his first account and his key, the way the contract says.
        const person = personFromHit(row.person, null);
        if (person !== null) openReport('R9', { person });
        return;
      }
      if ('report' in row.target) {
        openReport(row.target.report);
        return;
      }
      const { path } = row.target;
      // وصولی, ادائیگی and پیشگی are one screen with the type chosen (PRD FR-15.10).
      const type = asScreenType(path.split('/').at(-1));
      if (type === null) void router.navigate({ to: path });
      else void router.navigate({ to: '/bikri/voucher', search: { type } });
    };
  }, [openReport, router]);
}
