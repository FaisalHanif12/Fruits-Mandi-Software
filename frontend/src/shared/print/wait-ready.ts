// Waits until a printed document is really ready to print: its fonts are loaded and the layout is
// done. Then the print route says so (`print-ready-<job>`) and gives the height of the page in CSS
// pixels, which the backend turns into the height of the 80 mm roll (plan 14 contract 7).
//
// `document.fonts.ready` alone is not enough. It is already resolved before the page is laid out,
// because nothing has asked for a font yet. So the fonts the print needs are asked for by name.
//
// The hidden print window is not visible, and a page that is not visible can pause
// `requestAnimationFrame` for good. So nothing here waits for a frame: it waits for the fonts and
// then for a short timer, and measuring the document (`getBoundingClientRect`) makes the browser
// lay it out at that moment, frame or no frame.

import type { PrintFrame } from './types';

/** The fonts a print needs: Inter for numbers and English, Gulzar for Urdu, Naskh for Urdu digits. */
export function fontFamiliesOf(data: Pick<PrintFrame, 'language' | 'urdu_digits'>): string[] {
  const families = ['Inter'];
  if (data.language === 'ur') families.push('Gulzar');
  if (data.urdu_digits) families.push('Noto Naskh Arabic');
  return families;
}

/** A short pause after the fonts are in, so the page has settled before it is measured. */
const SETTLE_MS = 50;

function settle(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, SETTLE_MS);
  });
}

export interface PrintReady {
  /**
   * The height of the printed document itself, in CSS pixels, rounded up. The 80 mm roll is cut to
   * this height. It is not `document.documentElement.scrollHeight`: that is never less than the
   * height of the hidden window (1000 px), so a short slip would print on a long strip of paper.
   */
  height_px: number;
}

/**
 * Resolves with the page height once the fonts are loaded and the layout is final. A font that
 * cannot load does not stop the print (the page then uses its fallback font); the wait ends.
 */
export async function waitForPrintReady(
  families: readonly string[],
  sampleText: string,
  root?: HTMLElement | null,
): Promise<PrintReady> {
  /** The height of the document now. Asking for it makes the browser lay the page out first. */
  const heightNow = (): number =>
    root === null || root === undefined
      ? document.documentElement.scrollHeight
      : root.getBoundingClientRect().height;
  // Laying the page out makes the browser ask for every font it uses, a fallback font too, so it
  // is done before the wait for the fonts, not after.
  heightNow();
  const fonts =
    typeof document === 'undefined' ? undefined : (document.fonts as FontFaceSet | undefined);
  if (fonts !== undefined) {
    // `large` is any valid size: the load only needs the family.
    await Promise.all(
      families.map((family) => fonts.load(`large "${family}"`, sampleText).catch(() => [])),
    );
    await fonts.ready;
  }
  await settle();
  return { height_px: Math.ceil(heightNow()) };
}
