// Test helper: message IDs shown as text. A screen or print that asks Fluent for a message that no
// file has shows the ID itself (`print-partner-month-col-advances`), and in Urdu mode that is
// English text in the middle of Urdu (plan 81). This finds them in what was drawn: an element with
// no children whose whole text is written like an ID of a family of messages the Fluent files have.

/** The first words of the IDs the Fluent files define: `print`, `report`, `gardens` ... */
const families = new Set<string>();
const sources = import.meta.glob<string>('../../src/i18n/en/*.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
});
for (const source of Object.values(sources)) {
  for (const match of source.matchAll(/^([a-z][a-z0-9]*)-[a-z0-9_-]+\s*=/gm)) {
    if (match[1] !== undefined) families.add(match[1]);
  }
}

/** Lower-case words joined by dashes, for example `print-partner-month-col-advances`. */
const ID_TEXT = /^[a-z][a-z0-9]*(?:-[a-z0-9_]+)+$/;

/** The text of an element without the marks that isolate a number or a name. */
function plainText(element: Element): string {
  return element.textContent.replace(/[⁦-⁩‎‏]/g, '').trim();
}

/** Every text in `root` that is a message ID, once each. */
export function rawIdsIn(root: ParentNode): string[] {
  const found = new Set<string>();
  for (const element of root.querySelectorAll('*')) {
    if (element.children.length > 0) continue;
    if (element.tagName === 'SCRIPT' || element.tagName === 'STYLE') continue;
    const text = plainText(element);
    if (ID_TEXT.test(text) && families.has(text.split('-')[0] ?? '')) found.add(text);
  }
  return [...found];
}
