<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'Generate daily plan', prompt: 'Generate a prioritized daily plan based on my goals, recent activity, and products.' },
		{ label: 'Review goals progress', prompt: 'Review all active goals. Show progress, status, and what needs attention.' },
		{ label: 'Hire persona for task', prompt: 'I need to hire a persona/agent for a specific task. Ask me what kind of work needs to be done.' },
		{ label: 'System health check', prompt: 'Check the health of all RUSVEL systems: run tests, check build, verify all engines are operational.' },
		{ label: 'Weekly review', prompt: 'Generate a weekly review: accomplishments, blockers, insights, and next actions.' },
		{ label: 'What should I focus on?', prompt: 'Based on my goals, products, and current state — what should I focus on right now to maximize impact?' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="forge" title="Forge Department" icon="=" color="indigo" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="forge" title="Forge Department" icon="=" />
		</div>
	{/if}
</div>
