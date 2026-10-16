// Stylelint for the frontend CSS (CLAUDE.md rule 8, Code Standards F9 and F24).
// csstools/use-logical fails on physical directions: `margin-left`, `left:`, `text-align: left`,
// `border-right`, `float: right` and the like. Use `margin-inline-start`, `inset-inline-start`,
// `text-align: start` instead.

import noPhysicalShorthand from './stylelint-rules/no-physical-shorthand.js';

/** @type {import('stylelint').Config} */
export default {
  extends: ['stylelint-config-standard'],
  // A `/* stylelint-disable */` comment can never switch a rule off (reviewer finding 1).
  ignoreDisables: true,
  plugins: ['stylelint-use-logical', noPhysicalShorthand],
  rules: {
    'csstools/use-logical': ['always', { severity: 'error' }],
    // 3- and 4-value margin, padding, inset ... are physical too (stylelint-rules/).
    'mandi/no-physical-shorthand': true,
    // Position keywords left and right are physical too (reviewer recheck 2, finding E). Use
    // start/end where the property allows it, or a token offset set per direction.
    'declaration-property-value-disallowed-list': [
      {
        // The `background` and `mask` shorthands hold a position too, and a gradient's
        // `to left` / `to right` (plan 01 review finding 01-M2).
        '/^(-webkit-)?(background(-position(-x)?)?|transform-origin|object-position|mask(-position(-x)?)?|perspective-origin|offset-anchor|offset-position)$/':
          ['/\\b(left|right)\\b/'],
      },
      {
        message: (/** @type {string} */ property) =>
          `"left" and "right" in ${property} are physical directions and wrong in one of the two languages (CLAUDE.md rule 8, Code Standards F24).`,
      },
    ],
    // Colours, sizes, fonts and weights live only in tokens.css (Code Standards F9). Elsewhere
    // CSS uses var(--token): no hex colours, no px/rem/em units, and no raw font size, line
    // height, weight, family or radius.
    'color-no-hex': true,
    'unit-disallowed-list': ['px', 'rem', 'em', 'pt'],
    'declaration-property-value-allowed-list': {
      '/^(font-size|line-height|font-weight|font-family|border-radius)$/': ['/^var\\(/'],
    },
    // Tailwind CSS v4 at-rules.
    'at-rule-no-unknown': [
      true,
      {
        ignoreAtRules: [
          'theme',
          'source',
          'utility',
          'variant',
          'custom-variant',
          'apply',
          'reference',
          'plugin',
          'config',
        ],
      },
    ],
    'import-notation': 'string',
    // `&:lang(ur)` inside a Tailwind @utility block.
    'nesting-selector-no-missing-scoping-root': [true, { ignoreAtRules: ['utility'] }],
    // Kebab-case tokens, plus Tailwind's `--*: initial` reset in tokens.css.
    'custom-property-pattern': [
      '^([a-z][a-z0-9]*)(-[a-z0-9]+)*$|^\\*$',
      {
        message:
          'Tokens are kebab-case with a group prefix, for example --color-money-in (Tech Stack 6).',
      },
    ],
    // Blank lines group the tokens by kind.
    'custom-property-empty-line-before': null,
  },
  overrides: [
    {
      files: ['src/styles/tokens.css'],
      rules: {
        'color-no-hex': null,
        'unit-disallowed-list': null,
        'declaration-property-value-allowed-list': null,
      },
    },
    {
      // @font-face descriptors name the font files and their weights.
      files: ['src/styles/fonts.css'],
      rules: { 'declaration-property-value-allowed-list': null },
    },
  ],
};
