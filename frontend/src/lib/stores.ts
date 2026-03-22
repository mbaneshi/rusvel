import { writable } from 'svelte/store';
import type { SessionSummary } from './api';

export const sessions = writable<SessionSummary[]>([]);
export const activeSession = writable<SessionSummary | null>(null);
