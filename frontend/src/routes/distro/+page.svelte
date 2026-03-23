<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'SEO audit', prompt: 'Run an SEO audit. Check keyword rankings, page performance, and backlink profile.' },
		{ label: 'Marketplace listings', prompt: 'Review all marketplace listings. Show status, downloads, ratings, and revenue per platform.' },
		{ label: 'Affiliate program', prompt: 'Review the affiliate program. Show active partners, referrals, commissions, and top performers.' },
		{ label: 'Partnership outreach', prompt: 'Draft partnership outreach for strategic distribution channels. Identify potential partners.' },
		{ label: 'Distribution strategy', prompt: 'Analyze current distribution channels and recommend new ones based on product-market fit.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="distro" title="Distribution Department" icon="!" color="teal" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="distro" title="Distribution Department" icon="!" />
		</div>
	{/if}
</div>
