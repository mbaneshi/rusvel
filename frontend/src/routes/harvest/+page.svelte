<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'Scan for Rust gigs', prompt: 'Scan Upwork for Rust-related gigs. Filter for remote, hourly or fixed-price, and rank by fit.' },
		{ label: 'Score an opportunity', prompt: 'Score an opportunity I will describe. Evaluate fit, revenue potential, effort, and strategic value.' },
		{ label: 'Draft proposal', prompt: 'Draft a proposal for a freelance gig. Ask me for the job description and any specific requirements.' },
		{ label: 'Pipeline status', prompt: 'Show the current opportunity pipeline. List all tracked opportunities with status, score, and next action.' },
		{ label: 'Competitor analysis', prompt: 'Analyze competitors in my niche. Look at their positioning, pricing, and recent wins.' },
		{ label: 'Weekly digest', prompt: 'Generate a weekly digest of new opportunities, pipeline changes, and recommended actions.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="harvest" title="Harvest Department" icon="$" color="amber" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="harvest" title="Harvest Department" icon="$" />
		</div>
	{/if}
</div>
