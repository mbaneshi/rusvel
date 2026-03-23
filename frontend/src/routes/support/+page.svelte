<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'Open tickets', prompt: 'Show all open support tickets. Prioritize by urgency and time since creation.' },
		{ label: 'Write KB article', prompt: 'Write a knowledge base article. Ask me for the topic, audience, and key points to cover.' },
		{ label: 'NPS survey', prompt: 'Analyze recent NPS survey results. Show score breakdown, trends, and key feedback themes.' },
		{ label: 'Auto-triage', prompt: 'Triage incoming tickets. Categorize by type, assign priority, and suggest resolution paths.' },
		{ label: 'Escalation report', prompt: 'Generate an escalation report. Show unresolved high-priority tickets and SLA breaches.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="support" title="Support Department" icon="?" color="yellow" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="support" title="Support Department" icon="?" />
		</div>
	{/if}
</div>
