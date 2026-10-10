// Code Standards F9 and NFR-USE-11: colours, spacing, font sizes and radii come only from
// src/styles/tokens.css. No one-off values in components.
//
// Fails on, in ANY string or template string of a file (not only in className):
// - Tailwind arbitrary values: `p-[13px]`, `bg-[#2e7d32]`, `text-[15px]`, `[mask-type:alpha]`;
// - raw colours and sizes: '#2e7d32', '13px', '1.5rem', 'rgb(0 0 0)'.
// And on raw numbers for CSS sizes in object literals: `{ padding: 13 }`, `{ fontSize: 15 }`,
// including either branch of a conditional.
// Allowed: token classes (`p-3`, `bg-surface`), token references `h-(--control-h)` and
// 'var(--color-ink)', 0, and field paths such as 'lots[0].sales[2].qty'.

import { ESLintUtils } from '@typescript-eslint/utils';
import { branches, classTokens, isCodeOnlyString, propertyKey } from './class-names.js';

/** @typedef {import('@typescript-eslint/utils').TSESTree.Node} Node */

const createRule = ESLintUtils.RuleCreator(
  (name) =>
    `https://github.com/FaisalHanif12/Fruits-Mandi-Software/blob/main/frontend/eslint-rules/${name}.js`,
);

/** A Tailwind arbitrary value (`p-[13px]`) or arbitrary property (`[mask-type:alpha]`). */
const ARBITRARY_CLASS = /^[a-z][a-z0-9-]*-\[.+\]$|^\[[a-z-]+:.+\]$/;
const RAW_VALUE =
  /#[0-9a-f]{3,8}\b|(?<![\w-])\d+(\.\d+)?(px|rem|em|pt|vh|vw)\b|\b(rgba?|hsla?|oklch|oklab)\(/i;

/** Style keys whose plain number React turns into pixels. */
const CSS_SIZE_KEYS = new Set([
  'width',
  'height',
  'minWidth',
  'minHeight',
  'maxWidth',
  'maxHeight',
  'inlineSize',
  'blockSize',
  'minInlineSize',
  'minBlockSize',
  'maxInlineSize',
  'maxBlockSize',
  'margin',
  'marginBlock',
  'marginBlockStart',
  'marginBlockEnd',
  'marginInline',
  'marginInlineStart',
  'marginInlineEnd',
  'padding',
  'paddingBlock',
  'paddingBlockStart',
  'paddingBlockEnd',
  'paddingInline',
  'paddingInlineStart',
  'paddingInlineEnd',
  'gap',
  'rowGap',
  'columnGap',
  'inset',
  'insetBlock',
  'insetInline',
  'insetBlockStart',
  'insetBlockEnd',
  'insetInlineStart',
  'insetInlineEnd',
  'top',
  'bottom',
  'fontSize',
  'lineHeight',
  'letterSpacing',
  'borderRadius',
  'borderWidth',
  'outlineWidth',
  'outlineOffset',
]);

export default createRule({
  name: 'no-raw-design-values',
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow raw colours, sizes and spacing in components; use design tokens.',
    },
    messages: {
      arbitrary:
        "'{{token}}' is a one-off value. Use a token class from styles/tokens.css (Code Standards F9).",
      raw: "'{{value}}' is a raw colour or size. Use a token from styles/tokens.css, for example 'var(--color-ink)' (Code Standards F9).",
      style:
        "Raw number for '{{key}}'. Use a token from styles/tokens.css, for example 'var(--space-3)' (Code Standards F9).",
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    /**
     * @param {Node} node
     * @param {string} text
     */
    const checkText = (node, text) => {
      for (const token of classTokens(text)) {
        if (ARBITRARY_CLASS.test(token)) {
          context.report({ node, messageId: 'arbitrary', data: { token } });
          return;
        }
      }
      const raw = RAW_VALUE.exec(text);
      if (raw !== null) {
        context.report({ node, messageId: 'raw', data: { value: raw[0] } });
      }
    };

    return {
      Literal(node) {
        if (typeof node.value === 'string' && !isCodeOnlyString(node)) {
          checkText(node, node.value);
        }
      },
      TemplateElement(node) {
        checkText(node, node.value.cooked ?? node.value.raw);
      },
      Property(node) {
        const key = propertyKey(node);
        if (!CSS_SIZE_KEYS.has(key)) return;
        const rawNumber = branches(node.value).some(
          (value) =>
            value.type === 'Literal' && typeof value.value === 'number' && value.value !== 0,
        );
        if (rawNumber) {
          context.report({ node: node.value, messageId: 'style', data: { key } });
        }
      },
    };
  },
});
