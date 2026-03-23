<script lang="ts">
	import { page } from '$app/state';
	import { activeSession, departments } from '$lib/stores';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';
	import type { DepartmentDef } from '$lib/api';

	let currentSession: import('$lib/api').SessionSummary | null = $state(null);
	activeSession.subscribe((v) => (currentSession = v));

	let allDepts: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (allDepts = v));

	let dept = $derived(allDepts.find((d) => d.id === page.params.id));
</script>

<div class="flex h-full">
	{#if !dept}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Department not found.</p>
		</div>
	{:else if !currentSession}
		<div class="flex flex-1 items-center justify-center">
			<p class="text-sm text-[var(--r-fg-muted)]">Select a session to begin.</p>
		</div>
	{:else}
		<DepartmentPanel
			dept={dept.id}
			title={dept.title}
			icon={dept.icon}
			color={dept.color}
			quickActions={dept.quick_actions}
			tabs={dept.tabs}
		/>
		<div class="flex-1">
			<DepartmentChat dept={dept.id} title={dept.title} icon={dept.icon} />
		</div>
	{/if}
</div>
