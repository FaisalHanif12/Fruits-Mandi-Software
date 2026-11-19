// The screens and the backend agree on the Excel export (D5-Q1, backend round 10): each wrapper sends
// the command name the backend registers and the input in the shape `{ input }`; the event name is the
// backend's; the fixtures in tests/fixtures are the backend's own, copied (the same bytes), and carry
// exactly the fields of the generated types; and every message the export screens need has words in
// Urdu and in English. The names are also read from the backend's own command file, so a command
// renamed there fails here first.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as analyticsApi from '../../src/features/analytics/api';
import * as reportsApi from '../../src/features/reports/api';
import { bundles, LANGUAGES } from '../../src/i18n';
import { errorMessageId } from '../../src/shared/api';
import * as invoke from '../../src/shared/api/invoke';
import { EXPORT_PROGRESS_EVENT, exportFileName, withExtension } from '../../src/shared/export';
import {
  ANALYTICS_EXPORT,
  ANALYTICS_EXPORT_REQUEST,
  EXPORT_REFUSED_FILTER,
  EXPORT_REFUSED_JOB,
  EXPORT_REFUSED_PATH,
  REPORTS_EXPORT,
  REPORTS_EXPORT_REQUEST,
  REPORTS_EXPORT_URDU_REQUEST,
} from '../helpers/export-real';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue(null);
});

const BACKEND_FILE = path.resolve(__dirname, '../../../backend/app/src/commands/export.rs');
const BACKEND_FIXTURES = path.resolve(__dirname, '../../../backend/app/tests/fixtures');
const FRONTEND_FIXTURES = path.resolve(__dirname, '../fixtures');

describe('the commands and the event are the ones the backend declares', () => {
  it('both export commands exist in the backend file, and so does the event name', () => {
    const text = readFileSync(BACKEND_FILE, 'utf8');
    const declared = [...text.matchAll(/pub (?:async )?fn (\w+)/g)].map((match) => match[1] ?? '');
    expect(declared).toContain('reports_export_excel');
    expect(declared).toContain('analytics_export_excel');
    expect(text).toContain(`PROGRESS_EVENT: &str = "${EXPORT_PROGRESS_EVENT}"`);
  });
});

describe('each wrapper sends the command and `{ input }`', () => {
  it('reports_export_excel sends the request as the backend wrote it', async () => {
    call.mockResolvedValue(REPORTS_EXPORT);
    await expect(reportsApi.reportsExportExcel(REPORTS_EXPORT_REQUEST)).resolves.toBe(
      REPORTS_EXPORT,
    );
    expect(call).toHaveBeenLastCalledWith('reports_export_excel', {
      input: REPORTS_EXPORT_REQUEST,
    });
    await reportsApi.reportsExportExcel(REPORTS_EXPORT_URDU_REQUEST);
    expect(call).toHaveBeenLastCalledWith('reports_export_excel', {
      input: REPORTS_EXPORT_URDU_REQUEST,
    });
  });

  it('analytics_export_excel sends the request as the backend wrote it', async () => {
    call.mockResolvedValue(ANALYTICS_EXPORT);
    await expect(analyticsApi.analyticsExportExcel(ANALYTICS_EXPORT_REQUEST)).resolves.toBe(
      ANALYTICS_EXPORT,
    );
    expect(call).toHaveBeenLastCalledWith('analytics_export_excel', {
      input: ANALYTICS_EXPORT_REQUEST,
    });
  });
});

describe('the fixtures are the backend’s own', () => {
  it('every export fixture is the same text as the backend’s file, and none is missing', () => {
    const backend = readdirSync(BACKEND_FIXTURES).filter((name) => name.startsWith('export_'));
    const frontend = readdirSync(FRONTEND_FIXTURES).filter((name) => name.startsWith('export_'));
    expect(frontend.toSorted()).toEqual(backend.toSorted());
    expect(backend.length).toBeGreaterThanOrEqual(12);
    for (const name of backend) {
      expect(
        readFileSync(path.join(FRONTEND_FIXTURES, name), 'utf8'),
        `${name} differs from the backend's file`,
      ).toBe(readFileSync(path.join(BACKEND_FIXTURES, name), 'utf8'));
    }
  });
});

const keys = (value: object) => Object.keys(value).toSorted();

describe('the answers and requests carry exactly the fields of the generated types', () => {
  it('ExcelExported, and the two requests', () => {
    expect(keys(REPORTS_EXPORT)).toEqual(['path', 'rows', 'size_bytes']);
    expect(keys(REPORTS_EXPORT_REQUEST)).toEqual(['filters', 'id', 'job_id', 'language', 'path']);
    expect(keys(ANALYTICS_EXPORT_REQUEST)).toEqual([
      'filters',
      'job_id',
      'language',
      'path',
      'view',
    ]);
  });

  it('the refusals: the path and the job id on their own field, a filter on its field', () => {
    expect(EXPORT_REFUSED_PATH.errors.map((item) => [item.code, item.field])).toEqual([
      ['ERR_OUT_OF_RANGE', 'path'],
    ]);
    expect(EXPORT_REFUSED_JOB.errors.map((item) => [item.code, item.field])).toEqual([
      ['ERR_OUT_OF_RANGE', 'job_id'],
    ]);
    expect(EXPORT_REFUSED_FILTER.errors.map((item) => [item.code, item.field])).toEqual([
      ['ERR_DATE_INVALID', 'filters.from'],
    ]);
    // Neither range refusal carries a min or a max, so the range message must never be used for them.
    expect(EXPORT_REFUSED_PATH.errors[0]?.values).toEqual({});
    expect(EXPORT_REFUSED_JOB.errors[0]?.values).toEqual({});
  });
});

describe('every message the export needs has words in both languages', () => {
  it('the export messages and the error codes the export can answer', () => {
    const ids = [
      'export-excel',
      'export-file-type',
      'export-close',
      'export-reading',
      'export-writing',
      'export-finishing',
      'export-progress-label',
      'export-done',
      'export-path-refused',
      'export-too-many-rows',
      errorMessageId('ERR_DISK_FULL'),
      errorMessageId('ERR_INTERNAL'),
      errorMessageId('ERR_DATE_INVALID'),
    ];
    for (const language of LANGUAGES) {
      for (const id of ids) {
        expect(bundles[language].hasMessage(id), `${language} ${id}`).toBe(true);
      }
    }
  });
});

describe('the file name and the path', () => {
  it('suggests a name from the report and the day, and always ends the path in .xlsx', () => {
    expect(exportFileName('R18', new Date(2026, 9, 24))).toBe('R18-20261024.xlsx');
    expect(exportFileName('analytics-A', new Date(2026, 0, 5))).toBe('analytics-A-20260105.xlsx');
    expect(withExtension('C:\\Reports\\trial')).toBe('C:\\Reports\\trial.xlsx');
    expect(withExtension('C:\\Reports\\trial.xlsx')).toBe('C:\\Reports\\trial.xlsx');
    expect(withExtension('/tmp/Trial.XLSX')).toBe('/tmp/Trial.XLSX');
  });
});
