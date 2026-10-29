// The public face of the trips module (Tech Stack 5.3). Other modules import only from here. The
// screens are opened by the router (lazy routes, Code Standards F36), so the pages are not
// exported here. The dashboard tile مارکیٹ سفر (D5) reads `useTripsSummary`.
export { useTripsList, useTripsSummary } from './hooks';
export type { TripRow, TripsSummary } from './types';
