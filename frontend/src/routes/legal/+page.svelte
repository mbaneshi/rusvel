<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'Draft contract', prompt: 'Draft a contract. Ask me for contract type, parties, terms, and special clauses.' },
		{ label: 'Compliance check', prompt: 'Run a compliance check. Verify GDPR, privacy policy, and data handling practices.' },
		{ label: 'IP review', prompt: 'Review intellectual property assets. List patents, trademarks, copyrights, and trade secrets.' },
		{ label: 'Terms of service', prompt: 'Draft or review terms of service for our products. Ensure legal compliance.' },
		{ label: 'Privacy policy', prompt: 'Draft or review the privacy policy. Ensure GDPR and CCPA compliance.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="legal" title="Legal Department" icon="§" color="slate" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="legal" title="Legal Department" icon="§" />
		</div>
	{/if}
</div>
