<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'Funnel analysis', prompt: 'Analyze the conversion funnel. Show drop-off rates at each stage and recommend optimizations.' },
		{ label: 'Cohort report', prompt: 'Generate a cohort retention report. Show weekly/monthly retention rates and trends.' },
		{ label: 'KPI dashboard', prompt: 'Show the current KPI dashboard with MRR, DAU, churn rate, and growth rate.' },
		{ label: 'Churn prediction', prompt: 'Analyze churn signals and predict at-risk customers. Suggest retention interventions.' },
		{ label: 'Growth experiments', prompt: 'List active and planned growth experiments. Show hypothesis, metrics, and results.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="growth" title="Growth Department" icon="&" color="orange" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="growth" title="Growth Department" icon="&" />
		</div>
	{/if}
</div>
