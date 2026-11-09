// Where Print and Save PDF go (plan 69, PRD FR-12.12, FR-19.3 Mac-phase note). On Windows the backend
// prints to the printer chosen in Settings or writes the PDF files itself. On a Mac, both open the
// system print window of the web view, and its PDF button saves a PDF. This tells a screen which of
// the two it is on, so it can say so in words. It is only about words on the screen: what is printed
// is the same page.

/**
 * True on a Mac, where Print and Save PDF open the system print window. Read from the web view's
 * user agent each time it is asked (the Tauri web view on a Mac is WebKit: "Macintosh").
 */
export function opensSystemPrintWindow(): boolean {
  return typeof navigator !== 'undefined' && /Macintosh|Mac OS X/.test(navigator.userAgent);
}
