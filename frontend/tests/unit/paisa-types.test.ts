// @vitest-environment node
// Paisa is opaque: TypeScript itself rejects money maths outside money.ts (reviewer recheck
// finding A, Code Standards F3, M12). The fixtures are compiled here with the app's compiler
// options, because a `@ts-expect-error` comment is banned by lint.

import path from 'node:path';
import ts from 'typescript';
import { describe, expect, it } from 'vitest';

const FRONTEND_DIR = path.resolve(import.meta.dirname, '../..');
const FIXTURES = path.join(import.meta.dirname, 'fixtures', 'paisa-types');
const MISUSE = path.join(FIXTURES, 'misuse.ts');
const ALLOWED = path.join(FIXTURES, 'allowed.ts');
const EXPECT_MARK = '// expect-error';

/** The app's own compiler options (tsconfig.json), so the check matches `pnpm typecheck`. */
function appCompilerOptions(): ts.CompilerOptions {
  const configPath = path.join(FRONTEND_DIR, 'tsconfig.json');
  const config = ts.readConfigFile(configPath, (file) => ts.sys.readFile(file));
  return ts.parseJsonConfigFileContent(config.config, ts.sys, FRONTEND_DIR).options;
}

/** Line numbers (1-based) of the type errors in each fixture file. */
function errorLines(): Map<string, number[]> {
  const program = ts.createProgram([MISUSE, ALLOWED], appCompilerOptions());
  const lines = new Map<string, number[]>([
    [MISUSE, []],
    [ALLOWED, []],
  ]);
  for (const diagnostic of ts.getPreEmitDiagnostics(program)) {
    const file = diagnostic.file;
    if (file === undefined || diagnostic.start === undefined) continue;
    const line = file.getLineAndCharacterOfPosition(diagnostic.start).line + 1;
    lines.get(path.normalize(file.fileName))?.push(line);
  }
  return lines;
}

function markedLines(file: string): number[] {
  return (
    ts.sys
      .readFile(file)
      ?.split('\n')
      .flatMap((line, index) => (line.includes(EXPECT_MARK) ? [index + 1] : [])) ?? []
  );
}

describe('opaque Paisa', () => {
  const lines = errorLines();

  it('rejects every marked misuse, and nothing else, in misuse.ts', () => {
    const expected = markedLines(MISUSE);
    expect(expected.length).toBe(11);
    expect([...new Set(lines.get(MISUSE))].sort((x, y) => x - y)).toEqual(expected);
  });

  it('accepts sorting with comparePaisa, equality and passing Paisa on', () => {
    expect(lines.get(ALLOWED)).toEqual([]);
  });
});
