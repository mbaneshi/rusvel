import { writable } from 'svelte/store';
import type { SessionSummary, DepartmentDef } from './api';

export const sessions = writable<SessionSummary[]>([]);
export const activeSession = writable<SessionSummary | null>(null);
export const sidebarOpen = writable(true);
export const panelOpen = writable(true);
export const sidebarWidth = writable(256);
export const panelWidth = writable(288);
export const departments = writable<DepartmentDef[]>([]);
export const commandPaletteOpen = writable(false);

// ── Onboarding state ─────────────────────────────────────────
export interface OnboardingState {
	sessionCreated: boolean;
	goalAdded: boolean;
	planGenerated: boolean;
	deptChatUsed: boolean;
	agentCreated: boolean;
	dismissed: boolean;
	tourCompleted: boolean;
}

const defaultOnboarding: OnboardingState = {
	sessionCreated: false,
	goalAdded: false,
	planGenerated: false,
	deptChatUsed: false,
	agentCreated: false,
	dismissed: false,
	tourCompleted: false,
};

function loadOnboarding(): OnboardingState {
	if (typeof localStorage === 'undefined') return defaultOnboarding;
	try {
		const raw = localStorage.getItem('rusvel-onboarding');
		return raw ? { ...defaultOnboarding, ...JSON.parse(raw) } : defaultOnboarding;
	} catch {
		return defaultOnboarding;
	}
}

function createOnboardingStore() {
	const { subscribe, set, update } = writable<OnboardingState>(loadOnboarding());
	return {
		subscribe,
		set,
		update,
		complete(step: keyof OnboardingState) {
			update((s) => {
				const next = { ...s, [step]: true };
				if (typeof localStorage !== 'undefined') {
					localStorage.setItem('rusvel-onboarding', JSON.stringify(next));
				}
				return next;
			});
		},
		dismiss() {
			update((s) => {
				const next = { ...s, dismissed: true };
				if (typeof localStorage !== 'undefined') {
					localStorage.setItem('rusvel-onboarding', JSON.stringify(next));
				}
				return next;
			});
		},
	};
}

export const onboarding = createOnboardingStore();
