// The public face of the fruit module (Tech Stack 5.3). Other modules import only from here. The
// screen is opened by the router (a lazy route, Code Standards F36), so the page is not exported
// here. The partner statement and the stock screens read their fruit words through Fluent, not
// through this module.
export { FRUIT_KEY, readAgainAfterFruit } from './hooks';
export { fruitNumberText } from './list-columns';
export type { FruitRow } from './types';
