// CLAUDE.md rule 8, Code Standards F24: shorthands whose values are listed in physical order
// (top, right, bottom, left) break one of the two directions, so they are not allowed.
//
// Fails on:
// - margin, padding, scroll-margin, scroll-padding, border-width, border-style, border-color
//   with 3 or 4 values: `margin: 0 var(--space-1) 0 0` puts the space on the right in Urdu too;
// - inset with 2, 3 or 4 values: `inset: 0 auto auto 0`;
// - border-radius with more than one value: `border-radius: var(--radius-2) 0 0 0`.
// One or two values of margin and padding are the same on both sides, so they stay allowed.
//
// Use the logical longhands instead: margin-block, margin-inline, margin-inline-start,
// padding-inline-end, inset-block, inset-inline, border-inline-start-width,
// border-start-start-radius and so on.

import stylelint from 'stylelint';

const {
  createPlugin,
  utils: { report, ruleMessages, validateOptions },
} = stylelint;

export const ruleName = 'mandi/no-physical-shorthand';

export const messages = ruleMessages(ruleName, {
  rejected: (/** @type {string} */ property, /** @type {number} */ count) =>
    `"${property}" with ${String(count)} values sets left and right separately, so it is wrong in one of the two directions. Use the logical forms (margin-block, margin-inline, inset-inline-start, border-start-end-radius ...) (CLAUDE.md rule 8, Code Standards F24).`,
});

/** The most values each shorthand may have before it becomes direction-dependent. */
const MAX_VALUES = new Map([
  ['margin', 2],
  ['padding', 2],
  ['scroll-margin', 2],
  ['scroll-padding', 2],
  ['border-width', 2],
  ['border-style', 2],
  ['border-color', 2],
  ['inset', 1],
  ['border-radius', 1],
]);

/**
 * The number of values in a declaration, counting only spaces outside brackets, so
 * `calc(1px + 2px)` and `var(--a, 0 0)` count as one value. A `/` (border-radius) counts as a
 * value too, which keeps `a / b` above the limit.
 * @param {string} value
 * @returns {number}
 */
function countValues(value) {
  let depth = 0;
  let count = 0;
  let inValue = false;
  for (const character of value.trim()) {
    if (character === '(') depth += 1;
    if (character === ')') depth -= 1;
    const isSpace = /\s/.test(character);
    if (isSpace && depth === 0) {
      inValue = false;
    } else if (!inValue) {
      inValue = true;
      count += 1;
    }
  }
  return count;
}

/** @type {import('stylelint').Rule} */
const rule = (primary) => (root, result) => {
  const valid = validateOptions(result, ruleName, { actual: primary, possible: [true] });
  if (!valid) return;
  root.walkDecls((declaration) => {
    const property = declaration.prop.toLowerCase();
    const max = MAX_VALUES.get(property);
    if (max === undefined) return;
    const count = countValues(declaration.value);
    if (count > max) {
      report({
        ruleName,
        result,
        node: declaration,
        message: messages.rejected(property, count),
        word: declaration.prop,
      });
    }
  });
};

rule.ruleName = ruleName;
rule.messages = messages;
rule.meta = {
  url: 'https://github.com/FaisalHanif12/Fruits-Mandi-Software/blob/main/frontend/stylelint-rules/no-physical-shorthand.js',
};

export default createPlugin(ruleName, rule);
