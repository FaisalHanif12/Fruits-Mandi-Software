// The public face of the gardens module (Tech Stack 5.3). Other modules import only from here. The
// screens are opened by the router (lazy routes, Code Standards F36), so the pages are not
// exported here. The dashboard tile باغات reads its figures from `dashboard_get`; `useGardensSummary`
// (`gardens_summary`) stays for a screen that wants only the open gardens.
export { useGardensSummary } from './hooks';
export { gardenNumberText } from './list-columns';
export type { GardenRow, GardensSummary } from './types';
export { GardenFilterSelect } from './components/GardenFilterSelect';
