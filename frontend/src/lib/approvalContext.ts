import type { Job } from './api';

/** Human-readable label for `JobKind` JSON (string or `{"Custom":"..."}`). */
export function humanizeJobKind(kind: Job['kind']): string {
	if (typeof kind === 'string') return kind;
	if (kind && typeof kind === 'object' && 'Custom' in kind) {
		const c = (kind as { Custom?: string }).Custom;
		if (typeof c === 'string' && c.trim()) return c;
	}
	return JSON.stringify(kind);
}

export interface ApprovalSummaryRow {
	label: string;
	value: string;
}

function str(v: unknown): string | null {
	if (v === null || v === undefined) return null;
	if (typeof v === 'string') return v;
	if (typeof v === 'number' || typeof v === 'boolean') return String(v);
	return null;
}

/** Key fields from a job payload for the approvals UI (order preserved). */
export function payloadSummaryRows(payload: unknown): ApprovalSummaryRow[] {
	const rows: ApprovalSummaryRow[] = [];
	if (!payload || typeof payload !== 'object') return rows;
	const p = payload as Record<string, unknown>;
	const order: [string, string][] = [
		['title', 'Title'],
		['content_id', 'Content ID'],
		['platform', 'Platform'],
		['opportunity_id', 'Opportunity'],
		['path', 'Path'],
		['profile', 'Profile'],
		['message', 'Message']
	];
	for (const [key, label] of order) {
		const v = str(p[key]);
		if (v?.trim()) {
			let val = v;
			if (key === 'profile' && val.length > 160) val = `${val.slice(0, 160)}…`;
			rows.push({ label, value: val });
		}
	}
	return rows;
}

/** Preview text from `metadata.approval_pending_result` (worker `hold_for_approval`). */
export function approvalPendingPreview(
	metadata: Record<string, unknown>
): { headline: string; body: string } | null {
	const raw = metadata.approval_pending_result;
	if (!raw || typeof raw !== 'object') return null;
	const out = (raw as { output?: unknown }).output;
	if (!out || typeof out !== 'object') return null;
	const o = out as Record<string, unknown>;
	const body = str(o.body);
	if (body?.trim()) {
		return { headline: 'Generated output (pending your decision)', body };
	}
	const text = str(o.message) ?? str(o.title);
	if (text?.trim()) return { headline: 'Pending details', body: text };
	try {
		const s = JSON.stringify(out, null, 2);
		if (s.length > 2) return { headline: 'Pending output', body: s.slice(0, 8000) };
	} catch {
		/* ignore */
	}
	return null;
}

export function formatIsoDate(iso: string | null | undefined): string | null {
	if (!iso) return null;
	try {
		const d = new Date(iso);
		if (Number.isNaN(d.getTime())) return null;
		return d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
	} catch {
		return null;
	}
}

/** Map tool names in chat to short labels for approval cards. */
export function toolNameToApprovalLabel(toolName: string): string {
	const m: Record<string, string> = {
		harvest_propose: 'Harvest · proposal',
		content_publish: 'Content · publish',
		content_approve: 'Content · approve',
		content_draft: 'Content · draft',
		content_list: 'Content · list'
	};
	return m[toolName] ?? toolName.replace(/_/g, ' ');
}
