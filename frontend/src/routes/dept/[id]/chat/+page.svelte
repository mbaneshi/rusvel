<script lang="ts">
	import { page } from '$app/state';
	import { activeSession, departments } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import type { DepartmentDef } from '$lib/api';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	let allDepts: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (allDepts = v));

	let dept = $derived(allDepts.find((d) => d.id === page.params.id));
</script>

<div class="flex h-full min-h-0">
	{#if !dept}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--muted-foreground)]">Department not found.</p>
		</div>
	{:else if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--muted-foreground)]">Select a session to begin.</p>
		</div>
	{:else}
		<div class="min-w-0 flex-1">
			{#key dept.id}
				<DepartmentChat dept={dept.id} title={dept.title} />
			{/key}
		</div>
	{/if}
</div>
