// CLAUDE.md rule 8, Code Standards F24: both directions, one layout.
// Only logical directions (start and end) are allowed, so the same screen works in Urdu
// (right to left) and English (left to right).
//
// Fails on Tailwind classes in ANY string or template string of a file, not only in
// `className`, so `const cls = 'ml-2'` and `VARIANT_CLASSES = { a: 'text-left' }` are caught:
//   ml- mr- pl- pr- scroll-ml- scroll-pr- left- right- text-left text-right float-left
//   float-right clear-left clear-right border-l border-r rounded-l rounded-r rounded-tl
//   rounded-tr rounded-bl rounded-br bg-left object-right origin-top-left ...
// and on keys in ANY object literal (an object may become a style later):
//   marginLeft marginRight paddingLeft paddingRight left right borderLeft* borderRight*
//   scrollMarginLeft ... and textAlign, float, clear, backgroundPosition, transformOrigin,
//   objectPosition or maskPosition with the keyword 'left' or 'right' ('left top' too),
//   including in either branch of `on ? 'right' : 'left'`.
//
// Use instead: ms- me- ps- pe- start- end- text-start text-end border-s border-e rounded-s
// rounded-e, and marginInlineStart, insetInlineEnd, textAlign: 'end'.
// Number cells reach the physical right edge with `direction: ltr; text-align: end`
// (Code Standards F17), so they need no exception.
// CSS files are checked by Stylelint (stylelint-use-logical), not by this rule.

import { ESLintUtils } from '@typescript-eslint/utils';
import { branches, classTokens, isCodeOnlyString, propertyKey } from './class-names.js';

/** @typedef {import('@typescript-eslint/utils').TSESTree.Node} Node */

const createRule = ESLintUtils.RuleCreator(
  (name) =>
    `https://github.com/FaisalHanif12/Fruits-Mandi-Software/blob/main/frontend/eslint-rules/${name}.js`,
);

const PHYSICAL_CLASS_PATTERNS = [
  /^(m|p|scroll-m|scroll-p)[lr]-/,
  /^(left|right)-/,
  /^(text|float|clear)-(left|right)$/,
  /^border-[lr](-|$)/,
  /^rounded-([lr]|tl|tr|bl|br)(-|$)/,
  /^(bg|object|origin)-(.+-)?(left|right)(-|$)/,
];

const PHYSICAL_STYLE_KEY =
  /^(left|right|(margin|padding|scrollMargin|scrollPadding)(Left|Right)|border(Left|Right)\w*|border(Top|Bottom)(Left|Right)Radius)$/;
/** Style keys whose value may hold the keyword left or right. */
const DIRECTION_VALUE_KEYS = new Set([
  'textAlign',
  'float',
  'clear',
  'backgroundPosition',
  'backgroundPositionX',
  'transformOrigin',
  'objectPosition',
  'maskPosition',
  'perspectiveOrigin',
]);
const PHYSICAL_VALUE = /\b(left|right)\b/;

export default createRule({
  name: 'no-physical-direction',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow physical left and right in classes and style objects; use start and end.',
    },
    messages: {
      className:
        "'{{token}}' is a physical direction. Use the logical class (ms-, me-, ps-, pe-, start-, end-, text-start, text-end, border-s, rounded-e) so Urdu and English share one layout (CLAUDE.md rule 8, Code Standards F24).",
      style:
        "'{{key}}' is a physical direction. Use the logical property (marginInlineStart, paddingInlineEnd, insetInlineStart, textAlign: 'end') (CLAUDE.md rule 8, Code Standards F24).",
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    /**
     * @param {Node} node
     * @param {string} text
     */
    const checkClasses = (node, text) => {
      for (const token of classTokens(text)) {
        if (PHYSICAL_CLASS_PATTERNS.some((pattern) => pattern.test(token))) {
          context.report({ node, messageId: 'className', data: { token } });
        }
      }
    };

    return {
      Literal(node) {
        if (typeof node.value === 'string' && !isCodeOnlyString(node)) {
          checkClasses(node, node.value);
        }
      },
      TemplateElement(node) {
        checkClasses(node, node.value.cooked ?? node.value.raw);
      },
      Property(node) {
        const key = propertyKey(node);
        if (key === '') return;
        if (PHYSICAL_STYLE_KEY.test(key)) {
          context.report({ node: node.key, messageId: 'style', data: { key } });
          return;
        }
        if (!DIRECTION_VALUE_KEYS.has(key)) return;
        const physical = branches(node.value).some(
          (value) =>
            value.type === 'Literal' &&
            typeof value.value === 'string' &&
            PHYSICAL_VALUE.test(value.value),
        );
        if (physical) {
          context.report({ node: node.key, messageId: 'style', data: { key } });
        }
      },
    };
  },
});
