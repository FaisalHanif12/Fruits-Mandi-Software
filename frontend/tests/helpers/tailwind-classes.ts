// Test helper: finds the class names the code uses and asks Tailwind which of them make no CSS.
//
// tokens.css drops Tailwind's defaults (`--*: initial`), so a class such as `pe-12` or `max-w-xl`
// compiles to nothing and the screen silently loses its spacing or width (D1+D2 review, finding
// 6). The classes are read from the TypeScript syntax tree (className attributes, `className`
// properties and constants named like classes), then built by the real Tailwind compiler against
// the real globals.css.

import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import ts from 'typescript';
import { compile } from 'tailwindcss';

const FRONTEND_DIR = path.resolve(import.meta.dirname, '../..');
const SRC_DIR = path.join(FRONTEND_DIR, 'src');
const GLOBALS_CSS = path.join(SRC_DIR, 'styles/globals.css');
const require = createRequire(import.meta.url);

/** Class names that are not Tailwind's: set by other libraries or by plain CSS we do not compile. */
const FOREIGN_PREFIXES = ['ag-'];

export interface ClassUse {
  /** The class as written, for example `pe-12` or `normal:flex`. */
  name: string;
  /** Repository-relative file of the first use. */
  file: string;
}

function sourceFiles(dir: string): string[] {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) return entry.name === 'bindings' ? [] : sourceFiles(full);
    return /\.tsx?$/.test(entry.name) && !entry.name.endsWith('.d.ts') ? [full] : [];
  });
}

const CLASS_NAME_OF_CODE = /class/i;

/** A string that is compared, looked up or named as a type is code, not a class name. */
function isCodeString(node: ts.Node): boolean {
  const parent = node.parent;
  if (ts.isLiteralTypeNode(parent) || ts.isCaseClause(parent)) return true;
  if (ts.isElementAccessExpression(parent) && parent.argumentExpression === node) return true;
  if (ts.isPropertyAssignment(parent) && parent.name === node) return true;
  return (
    ts.isBinaryExpression(parent) &&
    [
      ts.SyntaxKind.EqualsEqualsEqualsToken,
      ts.SyntaxKind.ExclamationEqualsEqualsToken,
      ts.SyntaxKind.EqualsEqualsToken,
      ts.SyntaxKind.ExclamationEqualsToken,
    ].includes(parent.operatorToken.kind)
  );
}

/** True when the node sits inside something that holds class names. */
function inClassPosition(node: ts.Node): boolean {
  if (isCodeString(node)) return false;
  for (let current: ts.Node = node.parent; !ts.isSourceFile(current); current = current.parent) {
    if (ts.isJsxAttribute(current)) return current.name.getText() === 'className';
    if (ts.isPropertyAssignment(current) && CLASS_NAME_OF_CODE.test(current.name.getText())) {
      return true;
    }
    if (ts.isVariableDeclaration(current) && CLASS_NAME_OF_CODE.test(current.name.getText())) {
      return true;
    }
    if (ts.isStatement(current)) return false;
  }
  return false;
}

/** The class tokens of a piece of text; a token cut by a `${}` on either side is left out. */
function tokensOf(text: string, cutBefore: boolean, cutAfter: boolean): string[] {
  const parts = text.split(/\s+/).filter((part) => part !== '');
  if (parts.length === 0) return [];
  if (cutBefore && !/^\s/.test(text)) parts.shift();
  if (cutAfter && !/\s$/.test(text)) parts.pop();
  return parts;
}

/** Every class name used in the source, once, with the file of its first use. */
export function classesInSource(): ClassUse[] {
  const found = new Map<string, string>();
  for (const file of sourceFiles(SRC_DIR)) {
    const text = fs.readFileSync(file, 'utf8');
    const tree = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
    const rel = path.relative(FRONTEND_DIR, file);
    const add = (tokens: string[]) => {
      for (const token of tokens) if (!found.has(token)) found.set(token, rel);
    };
    const visit = (node: ts.Node) => {
      if (
        (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) &&
        inClassPosition(node)
      ) {
        add(tokensOf(node.text, false, false));
      } else if (ts.isTemplateExpression(node) && inClassPosition(node)) {
        add(tokensOf(node.head.text, false, true));
        for (const span of node.templateSpans) {
          const last = span === node.templateSpans[node.templateSpans.length - 1];
          add(tokensOf(span.literal.text, true, !last));
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(tree);
  }
  return [...found].map(([name, file]) => ({ name, file }));
}

/** Loads the style sheets Tailwind asks for (`@import 'tailwindcss'` and the relative ones). */
function loadStylesheet(id: string, base: string) {
  const file = id.startsWith('.')
    ? path.resolve(base, id)
    : id === 'tailwindcss'
      ? require.resolve('tailwindcss/index.css')
      : require.resolve(id);
  return Promise.resolve({
    path: file,
    base: path.dirname(file),
    content: fs.readFileSync(file, 'utf8'),
  });
}

async function newCompiler() {
  return compile(fs.readFileSync(GLOBALS_CSS, 'utf8'), {
    base: path.dirname(GLOBALS_CSS),
    from: GLOBALS_CSS,
    loadStylesheet,
    loadModule: () => Promise.reject(new Error('No JavaScript plugins are used.')),
  });
}

/**
 * The class names that make no usable CSS: Tailwind builds nothing for them, or builds a line that
 * reads a variable no token defines. Each class is built by a compiler of its own, because a
 * compiler keeps every class it has built before and would carry one bad class into the next.
 */
export async function classesWithNoCss(uses: ClassUse[] = classesInSource()): Promise<ClassUse[]> {
  const empty = (await newCompiler()).build([]);
  // Plain classes that globals.css and grid.css define themselves (`.entry-grid`): their rules
  // are in the base build, so they need no candidate.
  const defined = new Set(
    [...empty.matchAll(/\.((?:\\.|[\w-])+)/g)].map((m) => unescape(m[1] ?? '')),
  );
  const bad: ClassUse[] = [];
  for (const use of uses) {
    if (FOREIGN_PREFIXES.some((prefix) => use.name.startsWith(prefix))) continue;
    if (defined.has(use.name)) continue;
    const built = (await newCompiler()).build([use.name]);
    if (built === empty || usesUndefinedToken(built)) bad.push(use);
  }
  return bad;
}

/**
 * True when the CSS of a class reads a variable that nothing defines. With the defaults dropped,
 * `pe-12` builds to `padding-inline-end: calc(var(--spacing) * 12)` and `--spacing` does not
 * exist, so the browser throws the whole line away. Tailwind's own `--tw-*` variables have
 * fallbacks of their own, a `var()` with a fallback is safe, and `--radix-*` are set by Radix at
 * run time.
 */
function usesUndefinedToken(css: string): boolean {
  const declared = new Set([...css.matchAll(/(--[\w-]+)\s*:/g)].map((m) => m[1]));
  for (const match of css.matchAll(/var\((--[\w-]+)\s*([,)])/g)) {
    const variable = match[1] ?? '';
    if (variable.startsWith('--tw-') || variable.startsWith('--radix-') || match[2] === ',')
      continue;
    if (!declared.has(variable)) return true;
  }
  return false;
}

function unescape(selector: string): string {
  return selector.replace(/\\(.)/g, '$1');
}
