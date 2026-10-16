// AG Grid is registered once. The Community modules are all MIT-licensed. The styles are the
// grid's own CSS files, imported through Vite and restyled from our tokens in grid.css, so
// nothing is injected at run time and the Content Security Policy keeps style-src 'self'
// (Code Standards S5). The "-no-font" theme leaves out the icon font, because the policy allows
// fonts from the app only.

import { AllCommunityModule, ModuleRegistry } from 'ag-grid-community';
import 'ag-grid-community/styles/ag-grid.css';
import 'ag-grid-community/styles/ag-theme-quartz-no-font.css';

ModuleRegistry.registerModules([AllCommunityModule]);
