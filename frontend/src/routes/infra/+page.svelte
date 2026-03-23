<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'Deploy status', prompt: 'Show the current deployment status across all services and environments.' },
		{ label: 'Health check', prompt: 'Run health checks on all monitored services. Report uptime, response times, and issues.' },
		{ label: 'Incident report', prompt: 'Generate an incident report. Show open incidents, severity, timeline, and resolution status.' },
		{ label: 'Performance metrics', prompt: 'Show performance metrics: latency, throughput, error rates, and resource utilization.' },
		{ label: 'Cost analysis', prompt: 'Analyze infrastructure costs. Break down by service, show trends, and identify optimization opportunities.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="infra" title="Infra Department" icon=">" color="red" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="infra" title="Infra Department" icon=">" />
		</div>
	{/if}
</div>
