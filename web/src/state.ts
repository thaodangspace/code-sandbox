import { atom } from 'jotai';

// Container is now sourced from the router param `/container/:containerName`.
// Keep a global atom as an optional fallback (empty by default).
export const containerAtom = atom('');
