// Shared helpers for the local rules that read class names, strings and style objects.

/** @typedef {import('@typescript-eslint/utils').TSESTree.Node} Node */
/** @typedef {import('@typescript-eslint/utils').TSESTree.JSXAttribute} JSXAttribute */
/** @typedef {import('@typescript-eslint/utils').TSESTree.Property} Property */

/**
 * The attribute name as written, for example "className" or "aria-label".
 * @param {JSXAttribute} node
 * @returns {string}
 */
export function attributeName(node) {
  return node.name.type === 'JSXIdentifier' ? node.name.name : node.name.name.name;
}

/**
 * The value expressions an expression can end up as: both branches of `a ? b : c`, both sides
 * of `a && b` and `a ?? b`, and the inside of `x as const`. Example: `on ? 'Yes' : 'No'` gives
 * the two literals.
 * @param {Node} node
 * @returns {Node[]}
 */
export function branches(node) {
  if (node.type === 'ConditionalExpression') {
    return [...branches(node.consequent), ...branches(node.alternate)];
  }
  if (node.type === 'LogicalExpression') {
    return [...branches(node.left), ...branches(node.right)];
  }
  if (node.type === 'TSAsExpression' || node.type === 'TSSatisfiesExpression') {
    return branches(node.expression);
  }
  return [node];
}

/**
 * True when a string literal is code that can hold no class name or word: an import or export
 * source, a literal type, or a plain object key.
 * @param {Node} node a Literal
 * @returns {boolean}
 */
export function isCodeOnlyString(node) {
  const parent = node.parent;
  if (parent === undefined) return false;
  switch (parent.type) {
    case 'ImportDeclaration':
    case 'ExportNamedDeclaration':
    case 'ExportAllDeclaration':
    case 'ImportExpression':
    case 'TSLiteralType':
    case 'TSExternalModuleReference':
    case 'TSModuleDeclaration':
      return true;
    case 'Property':
    case 'TSPropertySignature':
    case 'MethodDefinition':
    case 'PropertyDefinition':
      return parent.key === node && !parent.computed;
    default:
      return false;
  }
}

/**
 * The class tokens of a string, each without its variants and markers.
 * "md:!-ml-2 rtl:text-right" gives ["ml-2", "text-right"].
 * @param {string} text
 * @returns {string[]}
 */
export function classTokens(text) {
  return text
    .split(/\s+/)
    .filter((token) => token !== '')
    .map((token) => {
      // Split variants on ':' but not inside an arbitrary value such as [mask-type:alpha].
      const parts = token.split(/:(?![^[]*\])/);
      const last = parts[parts.length - 1] ?? '';
      return last.replace(/^!/, '').replace(/^-/, '').replace(/!$/, '');
    });
}

/**
 * The key of an object property as text, or '' when it is computed from code.
 * @param {Property} property
 * @returns {string}
 */
export function propertyKey(property) {
  if (property.computed) return '';
  if (property.key.type === 'Identifier') return property.key.name;
  // A non-computed key that is not a name is a string or number literal.
  return typeof property.key.value === 'string' ? property.key.value : '';
}
