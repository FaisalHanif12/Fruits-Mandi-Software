// The public face of the gardens module (Tech Stack 5.3). Other modules import only from here. The
// screens are opened by the router (lazy routes, Code Standards F36), so the pages are not
// exported here. The dashboard tile باغات (D5) reads `useGardensSummary`.
export { useGardensSummary } from './hooks';
export { gardenNumberText } from './list-columns';
export type { GardenRow, GardensSummary } from './types';
