/**
 * RUSVEL Design System — Tokens
 *
 * The single source of truth for colors, spacing, and component variants.
 * All UI components reference these tokens for consistency.
 */

// ── Status colors ────────────────────────────────────────────
export const status = {
	active: 'bg-green-900/50 text-green-300 border-green-800/50',
	warning: 'bg-yellow-900/50 text-yellow-300 border-yellow-800/50',
	error: 'bg-red-900/50 text-red-300 border-red-800/50',
	info: 'bg-blue-900/50 text-blue-300 border-blue-800/50',
	neutral: 'bg-gray-800 text-gray-400 border-gray-700',
	accent: 'bg-indigo-900/50 text-indigo-300 border-indigo-800/50'
} as const;

// ── Priority colors ──────────────────────────────────────────
export const priority = {
	urgent: 'text-red-400',
	high: 'text-orange-400',
	medium: 'text-yellow-400',
	low: 'text-gray-400'
} as const;

// ── Engine colors (each engine gets a distinct accent) ───────
export const engine = {
	forge: {
		bg: 'bg-indigo-600/15',
		text: 'text-indigo-300',
		border: 'border-indigo-500/30',
		icon: '='
	},
	code: {
		bg: 'bg-emerald-600/15',
		text: 'text-emerald-300',
		border: 'border-emerald-500/30',
		icon: '#'
	},
	harvest: {
		bg: 'bg-amber-600/15',
		text: 'text-amber-300',
		border: 'border-amber-500/30',
		icon: '$'
	},
	content: {
		bg: 'bg-purple-600/15',
		text: 'text-purple-300',
		border: 'border-purple-500/30',
		icon: '*'
	},
	gtm: { bg: 'bg-cyan-600/15', text: 'text-cyan-300', border: 'border-cyan-500/30', icon: '^' }
} as const;

// ── Component size variants ──────────────────────────────────
export const size = {
	xs: { text: 'text-xs', px: 'px-1.5', py: 'py-0.5', gap: 'gap-1' },
	sm: { text: 'text-sm', px: 'px-2.5', py: 'py-1', gap: 'gap-1.5' },
	md: { text: 'text-sm', px: 'px-3', py: 'py-1.5', gap: 'gap-2' },
	lg: { text: 'text-base', px: 'px-4', py: 'py-2', gap: 'gap-2.5' }
} as const;

export type StatusVariant = keyof typeof status;
export type PriorityVariant = keyof typeof priority;
export type EngineVariant = keyof typeof engine;
export type SizeVariant = keyof typeof size;
