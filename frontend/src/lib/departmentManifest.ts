/** Mirrors `rusvel_core::registry::QuickAction`. */
export interface QuickAction {
	label: string;
	prompt: string;
}

/** Mirrors `rusvel_core::department::UiContribution` (subset used by the shell). */
export interface UiContribution {
	tabs: string[];
	dashboard_cards?: DashboardCard[];
	has_settings?: boolean;
	custom_components?: string[];
}

export interface DashboardCard {
	title: string;
	description: string;
	size?: string;
}

/**
 * Department record from `GET /api/departments` — aligned with registry `DepartmentDef`
 * and forward-compatible with manifest-shaped payloads (`ui.tabs`).
 */
export interface DepartmentDef {
	id: string;
	name: string;
	title: string;
	icon: string;
	color: string;
	system_prompt: string;
	capabilities: string[];
	tabs: string[];
	quick_actions: QuickAction[];
	default_config: Record<string, unknown>;
	ui?: UiContribution;
}

/** Normalize API JSON whether the backend sends flat `DepartmentDef` or manifest-style `ui.tabs`. */
export function normalizeDepartment(raw: Record<string, unknown>): DepartmentDef {
	const ui = raw.ui as UiContribution | undefined;
	const tabsFromUi = ui?.tabs?.length ? ui.tabs : undefined;
	const flat = raw.tabs as string[] | undefined;
	const tabs = tabsFromUi ?? flat ?? [];

	const base = {
		...raw,
		tabs
	} as DepartmentDef;

	if (ui) {
		base.ui = { ...ui, tabs };
	}
	return base;
}

export function normalizeDepartmentList(raw: unknown): DepartmentDef[] {
	if (!Array.isArray(raw)) return [];
	return raw.map((x) => normalizeDepartment(x as Record<string, unknown>));
}

/** Effective tab IDs for the department panel (manifest-first). */
export function tabsFromDepartment(d: DepartmentDef): string[] {
	if (d.ui?.tabs?.length) return d.ui.tabs;
	return d.tabs ?? [];
}

export function deptHref(deptId: string): string {
	return `/dept/${encodeURIComponent(deptId)}/chat`;
}

/**
 * Pick a department id for deep links when the registry order or IDs may differ.
 * Prefers `preferred` if present; otherwise first department; otherwise `fallback`.
 */
export function resolveDeptId(
	depts: DepartmentDef[],
	preferred: string,
	fallback: string
): string {
	if (depts.some((d) => d.id === preferred)) return preferred;
	return depts[0]?.id ?? fallback;
}
