// The check behind `pnpm i18n:check` (entry: scripts/i18n-check.mjs). The Urdu and English
// Fluent files must match (NFR-URD-22, Code Standards F32).
//
// Fails when:
// - a .ftl file exists in one language folder and not the other;
// - a file cannot be parsed (Fluent "junk");
// - a message or term ID exists in one language and not the other;
// - a message or term is empty;
// - a message has the same words in both languages, unless its ID is in IDENTICAL_ALLOWED
//   (for example the name of each language, written in its own script). A message made only
//   of references, such as `glossary-bikri = { -bikri }`, has no words of its own and is not
//   flagged: each language fills it from its own glossary.
//
// Example: ur/common.ftl has `common-app-name` but en/common.ftl does not, so it fails with
// "en/common.ftl: missing common-app-name (present in ur)".

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { Junk, Message, StringLiteral, Term, parse, serializeExpression } from '@fluent/syntax';

/** Message IDs that may read the same in Urdu and English. Keep this list short. */
export const IDENTICAL_ALLOWED = new Set([
  // The language switch shows each language's own name in its own script (PRD FR-7.30).
  'common-language-ur',
  'common-language-en',
  // The badge of the "English, no conversion" typing mode is "EN" in both languages (Tech Stack 4.4).
  'translit-mode-english',
]);

const LANGUAGES = ['ur', 'en'];

/**
 * Turns a Fluent pattern into plain text so values can be compared and checked for emptiness.
 * @param {import('@fluent/syntax').Pattern | null} pattern
 * @returns {string}
 */
function patternText(pattern) {
  if (pattern === null) {
    return '';
  }
  return pattern.elements
    .map((element) => {
      if (element.type === 'TextElement') return element.value;
      // A quoted string such as { "" } counts as its text, so an empty one is caught.
      if (element.expression instanceof StringLiteral) return element.expression.parse().value;
      return `{${serializeExpression(element.expression)}}`;
    })
    .join('')
    .trim();
}

/**
 * True when the text has letters outside its {placeables}.
 * @param {string} text
 */
function hasOwnWords(text) {
  return /\p{L}/u.test(text.replace(/\{[^}]*\}/g, '').replace(/^\.[\w-]+=/gm, ''));
}

/**
 * Reads one .ftl file into a map of ID to its text (value plus attributes).
 * @param {string} file
 * @param {string} label used in error messages, for example "ur/common.ftl"
 * @param {string[]} errors
 * @returns {Map<string, string>}
 */
function readEntries(file, label, errors) {
  const resource = parse(readFileSync(file, 'utf8'), { withSpans: false });
  /** @type {Map<string, string>} */
  const entries = new Map();
  for (const entry of resource.body) {
    if (entry instanceof Junk) {
      const first = entry.content.split('\n')[0] ?? '';
      errors.push(`${label}: cannot be read near "${first.trim()}"`);
      continue;
    }
    if (!(entry instanceof Message) && !(entry instanceof Term)) {
      continue;
    }
    const id = entry instanceof Term ? `-${entry.id.name}` : entry.id.name;
    if (entries.has(id)) {
      errors.push(`${label}: ${id} is defined twice`);
    }
    const value = patternText(entry.value);
    const attributes = entry.attributes.map(
      (attribute) => `.${attribute.id.name}=${patternText(attribute.value)}`,
    );
    if (value === '' && attributes.length === 0) {
      errors.push(`${label}: ${id} is empty`);
    }
    for (const attribute of entry.attributes) {
      if (patternText(attribute.value) === '') {
        errors.push(`${label}: ${id}.${attribute.id.name} is empty`);
      }
    }
    entries.set(id, [value, ...attributes].join('\n'));
  }
  return entries;
}

/**
 * Checks the Fluent folders under `i18nDir` (which holds `ur/` and `en/`).
 * @param {string} i18nDir
 * @param {{ identicalAllowed?: Set<string> }} [options]
 * @returns {string[]} one line per problem; empty when everything matches
 */
export function checkI18n(i18nDir, options = {}) {
  const identicalAllowed = options.identicalAllowed ?? IDENTICAL_ALLOWED;
  /** @type {string[]} */
  const errors = [];

  /** @type {Record<string, string[]>} */
  const filesByLanguage = {};
  for (const language of LANGUAGES) {
    const dir = path.join(i18nDir, language);
    if (!existsSync(dir)) {
      errors.push(`${language}/: folder is missing`);
      filesByLanguage[language] = [];
      continue;
    }
    filesByLanguage[language] = readdirSync(dir)
      .filter((name) => name.endsWith('.ftl'))
      .sort();
  }

  const [ur, en] = LANGUAGES;
  const urFiles = filesByLanguage[ur] ?? [];
  const enFiles = filesByLanguage[en] ?? [];
  for (const name of urFiles) {
    if (!enFiles.includes(name)) errors.push(`en/${name}: file is missing (present in ur)`);
  }
  for (const name of enFiles) {
    if (!urFiles.includes(name)) errors.push(`ur/${name}: file is missing (present in en)`);
  }

  for (const name of urFiles.filter((file) => enFiles.includes(file))) {
    const urEntries = readEntries(path.join(i18nDir, 'ur', name), `ur/${name}`, errors);
    const enEntries = readEntries(path.join(i18nDir, 'en', name), `en/${name}`, errors);
    for (const id of urEntries.keys()) {
      if (!enEntries.has(id)) errors.push(`en/${name}: missing ${id} (present in ur)`);
    }
    for (const [id, enText] of enEntries) {
      const urText = urEntries.get(id);
      if (urText === undefined) {
        errors.push(`ur/${name}: missing ${id} (present in en)`);
      } else if (urText === enText && hasOwnWords(enText) && !identicalAllowed.has(id)) {
        errors.push(
          `${name}: ${id} is the same in ur and en; translate it or add it to the allow list`,
        );
      }
    }
  }
  return errors;
}
