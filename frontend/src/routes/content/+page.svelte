<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'Draft blog post', prompt: 'Draft a blog post about a topic I will describe. Ask me for the topic, target audience, and tone.' },
		{ label: 'Adapt for Twitter thread', prompt: 'Take existing content and adapt it into a compelling Twitter thread. Ask me for the source content.' },
		{ label: 'Generate content calendar', prompt: 'Generate a content calendar for the next 2 weeks. Include blog posts, social media, and newsletter items.' },
		{ label: 'Review unpublished drafts', prompt: 'Review all unpublished content drafts. List them with status, topic, and suggested next steps.' },
		{ label: 'Engagement report', prompt: 'Generate an engagement report across all content platforms. Show metrics, trends, and recommendations.' },
		{ label: 'Draft LinkedIn post', prompt: 'Draft a professional LinkedIn post. Ask me for the topic and key message.' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="content" title="Content Department" icon="*" color="purple" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="content" title="Content Department" icon="*" />
		</div>
	{/if}
</div>
