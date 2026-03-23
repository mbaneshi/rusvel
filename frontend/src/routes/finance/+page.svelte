<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'Record income', prompt: 'Record a new income transaction. Ask me for amount, source, category, and date.' },
		{ label: 'Log expense', prompt: 'Log a new expense. Ask me for amount, description, category, and date.' },
		{ label: 'Calculate runway', prompt: 'Calculate the current runway based on cash on hand and burn rate. Show months remaining.' },
		{ label: 'Tax estimate', prompt: 'Estimate tax liability for the current quarter. Break down by income tax, self-employment, and deductions.' },
		{ label: 'P&L report', prompt: 'Generate a profit & loss report. Show revenue, expenses, and net income by category.' },
		{ label: 'Unit economics', prompt: 'Analyze unit economics for each product. Show CAC, LTV, margins, and payback period.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="finance" title="Finance Department" icon="%" color="green" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="finance" title="Finance Department" icon="%" />
		</div>
	{/if}
</div>
