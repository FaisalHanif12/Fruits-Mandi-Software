// Money fields in the Rust types are exported as `Paisa` (#[ts(type = "Paisa")], plan 02).
// ts-rs writes the name but not its import, so gen:types adds
//   import type { Paisa } from '../shared/format/money';
// to every binding file that uses the word Paisa as a type, and only to those.
//
// Example: `export type Bill = { total: Paisa };` gets the import line after the ts-rs header;
// `export type AppInfo = { version: string };` is left exactly as it was.

export const PAISA_IMPORT = "import type { Paisa } from '../shared/format/money';";

const USES_PAISA = /(?<![\w$])Paisa(?![\w$])/;
const HAS_IMPORT = /import\s+type\s*\{[^}]*\bPaisa\b[^}]*\}/;

/**
 * Returns the binding source with the Paisa import added when it is needed.
 * @param {string} source one file written by ts-rs
 * @returns {string}
 */
export function addPaisaImport(source) {
  const withoutComments = source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
  if (!USES_PAISA.test(withoutComments) || HAS_IMPORT.test(source)) {
    return source;
  }
  const lines = source.split('\n');
  // Keep the ts-rs "generated, do not edit" header first.
  const insertAt = lines[0]?.startsWith('//') ? 1 : 0;
  lines.splice(insertAt, 0, PAISA_IMPORT);
  return lines.join('\n');
}
