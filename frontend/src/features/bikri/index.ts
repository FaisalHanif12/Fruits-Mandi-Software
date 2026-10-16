// The public face of the bikri module (Tech Stack 5.3). Other modules import only from here.
// The sale book screen is opened by the router. Plan 13 (save, drafts, edit and cancel) adds to
// this module, and the draft store is exported so the save and the leave guard can read it.
export { BikriBookPage } from './pages/BikriBookPage';
export { createBikriDraft, useBikriDraft, type BikriDraftStores } from './draft-store';
