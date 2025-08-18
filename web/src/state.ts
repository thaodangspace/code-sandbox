import { atom } from 'jotai';

const initialContainer = new URLSearchParams(window.location.search).get('container') ?? '';

export const containerAtom = atom(initialContainer);
