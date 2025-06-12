import { writable } from 'svelte/store';

// Define all possible tab values as a TypeScript union type
export type TabKey = 'upload' | 'peers' | 'account';

export const tabStore = writable<TabKey>('account');

