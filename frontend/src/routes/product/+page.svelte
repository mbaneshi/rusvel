<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'View roadmap', prompt: 'Show the current product roadmap with all features, milestones, and priorities.' },
		{ label: 'Add feature', prompt: 'Add a new feature to the roadmap. Ask me for name, description, priority, and target milestone.' },
		{ label: 'Prioritize backlog', prompt: 'Review and prioritize the feature backlog. Use impact vs effort scoring.' },
		{ label: 'Pricing analysis', prompt: 'Analyze current pricing tiers. Compare with competitors and suggest optimizations.' },
		{ label: 'User feedback summary', prompt: 'Summarize recent user feedback. Group by feature requests, bugs, and sentiment.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="product" title="Product Department" icon="@" color="rose" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="product" title="Product Department" icon="@" />
		</div>
	{/if}
</div>
