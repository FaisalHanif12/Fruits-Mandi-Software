// The words of the app in Node, for the end-to-end specs. They read the same Fluent files as the
// app, so a spec never types a label by hand: when a label changes, the spec follows it.
// Isolation marks are left out here, and the page helpers strip them from the page text, so the
// two sides compare equal.

import { FluentBundle, FluentResource } from '@fluent/bundle';
import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { FRONTEND_DIR } from '../paths.mjs';

/** @typedef {'ur' | 'en'} Language */

/** @type {Map<Language, FluentBundle>} */
const bundles = new Map();

/** @param {Language} language */
function bundleOf(language) {
  const known = bundles.get(language);
  if (known !== undefined) return known;
  const folder = path.join(FRONTEND_DIR, 'src', 'i18n', language);
  const bundle = new FluentBundle(language, { useIsolating: false });
  for (const file of readdirSync(folder).filter((name) => name.endsWith('.ftl'))) {
    const errors = bundle.addResource(
      new FluentResource(readFileSync(path.join(folder, file), 'utf8')),
    );
    if (errors[0] !== undefined) throw new Error(`${file}: ${errors[0].message}`);
  }
  bundles.set(language, bundle);
  return bundle;
}

/**
 * One message in one language.
 * @param {Language} language
 * @param {string} id
 * @param {Record<string, string | number>} [vars]
 * @returns {string}
 */
export function t(language, id, vars = {}) {
  const pattern = bundleOf(language).getMessage(id)?.value;
  if (pattern === null || pattern === undefined) throw new Error(`${language}: no message ${id}`);
  return bundleOf(language).formatPattern(pattern, vars);
}

/** Both languages, Urdu first: the app starts in Urdu (PRD FR-7.30). */
/** @type {readonly Language[]} */
export const LANGUAGES = ['ur', 'en'];

/** The direction each language is written in. */
/** @type {Record<Language, 'rtl' | 'ltr'>} */
export const DIRECTION = { ur: 'rtl', en: 'ltr' };
