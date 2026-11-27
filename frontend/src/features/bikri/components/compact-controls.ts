// Whether the controls of the sale book's top row are shown small (plan 95, PRD FR-5.3 v1.57): the row of tabs had
// too little room, so the print buttons show an icon and a key. `BikriToolbar` decides it by measuring and provides
// it here; a button that gives up its words reads it to put them in its tip (the CSS of the row hides the words).

import { createContext, useContext } from 'react';

const CompactControlsContext = createContext(false);

export const CompactControlsProvider = CompactControlsContext.Provider;

/** True while the top row of the sale book keeps its controls small so every tab fits. */
export function useCompactControls(): boolean {
  return useContext(CompactControlsContext);
}
