// The row of tabs of the section a screen belongs to (PRD FR-5.3, v1.56, plan 93), handed from the shell to the title
// row of whatever screen is open. The shell knows the section and the address; a screen only says where its title
// row is (`TitleOrTabs`), so no screen builds tabs of its own and a screen outside a section keeps its plain title.

import { createContext, useContext, type ReactNode } from 'react';

const SectionTabsContext = createContext<ReactNode>(null);

/** The shell wraps the screen in this, with the tabs of the section the address belongs to, or null. */
export const SectionTabsProvider = SectionTabsContext.Provider;

/** The tabs of the section the open screen belongs to, or null when it belongs to none. */
export function useSectionTabs(): ReactNode {
  return useContext(SectionTabsContext);
}
