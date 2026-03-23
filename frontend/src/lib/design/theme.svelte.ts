/**
 * RUSVEL Theme State — Svelte 5 runes-based theme management.
 * Dark by default, with light mode support.
 */

type Theme = 'dark' | 'light';

let current = $state<Theme>('dark');

export function getTheme(): Theme {
	return current;
}

export function setTheme(theme: Theme) {
	current = theme;
	if (typeof document !== 'undefined') {
		document.documentElement.classList.toggle('light', theme === 'light');
		document.cookie = `rusvel-theme=${theme};path=/;max-age=31536000;SameSite=Lax`;
	}
}

export function toggleTheme() {
	setTheme(current === 'dark' ? 'light' : 'dark');
}

/** Call once on app init to restore saved theme from cookie */
export function initTheme() {
	if (typeof document === 'undefined') return;
	const match = document.cookie.match(/rusvel-theme=(dark|light)/);
	if (match) {
		setTheme(match[1] as Theme);
	}
}
