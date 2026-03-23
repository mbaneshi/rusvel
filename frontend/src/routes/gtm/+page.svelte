<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'List all contacts', prompt: 'List all contacts in the CRM. Show name, company, status, and last interaction date.' },
		{ label: 'Draft outreach sequence', prompt: 'Draft a multi-step outreach sequence for a prospect. Ask me for the contact details and context.' },
		{ label: 'Generate invoice', prompt: 'Generate an invoice for completed work. Ask me for client details, line items, and payment terms.' },
		{ label: 'Deal pipeline status', prompt: 'Show the current deal pipeline. List all active deals with stage, value, probability, and next action.' },
		{ label: 'Revenue report', prompt: 'Generate a revenue report. Show closed deals, pending invoices, projected revenue, and trends.' },
		{ label: 'Follow up with leads', prompt: 'Review all leads that need follow-up. Prioritize by deal value and time since last contact.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="gtm" title="GoToMarket Department" icon="^" color="cyan" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="gtm" title="GoToMarket Department" icon="^" />
		</div>
	{/if}
</div>
