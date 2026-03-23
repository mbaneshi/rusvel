<script lang="ts">
	import { activeSession } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	const quickActions = [
		{ label: 'Analyze this codebase', prompt: 'Analyze the RUSVEL codebase. Show me: total crates, lines of code, test count, and any issues found.' },
		{ label: 'Run all tests', prompt: 'Run `cargo test` and `npm run check` in the frontend. Report results.' },
		{ label: 'Review architecture', prompt: 'Review the current RUSVEL architecture. Check if engines follow hexagonal rules (no adapter imports). List any violations.' },
		{ label: 'Find TODOs', prompt: 'Search the codebase for TODO, FIXME, HACK comments. List them grouped by crate.' },
		{ label: 'Git status', prompt: 'Run `git log --oneline -10` and `git status`. Show what changed recently.' },
		{ label: 'What needs work?', prompt: 'Read docs/status/gap-analysis.md and docs/status/current-state.md. What are the highest priority items to work on next?' },
	];
</script>

<div class="flex h-full">
	{#if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel dept="code" title="Code Department" icon="#" color="emerald" {quickActions} />
		<div class="flex-1">
			<DepartmentChat dept="code" title="Code Department" icon="#" />
		</div>
	{/if}
</div>
