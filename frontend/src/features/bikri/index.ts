// The public face of the bikri module (Tech Stack 5.3). Other modules import only from here.
// The sale book and the list of today's بکری are opened by the router. The start-up offer of an
// unfinished بکری (draft-offer.tsx) is mounted by the app shell from its own file, so the sale
// book itself stays a lazy route (Code Standards F36).
export { BikriBookPage } from './pages/BikriBookPage';
export { TodayBikriPage } from './pages/TodayBikriPage';
export { createBikriDraft, useBikriDraft, type BikriDraftStores } from './draft-store';
export { registerPrintAfterSave, type PrintAfterSave } from './after-save';
// What the reports module borrows for the sale book report (R1): finding a بکری by its number, and the
// print box of its bill (the printed bill is the shared print job, not a report page).
export { useBikriFind } from './hooks/useBikriReads';
export { PrintAction } from './print/PrintAction';
