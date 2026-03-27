<script lang="ts">
	import { page } from '$app/state';
	import { departments } from '$lib/stores';
	import type { DepartmentDef } from '$lib/api';
	import EventsTab from '$lib/components/department/EventsTab.svelte';
	import { getDeptColor } from '$lib/components/department/colors';

	let allDepts: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (allDepts = v));

	let dept = $derived(allDepts.find((d) => d.id === page.params.id));
	let deptHsl = $derived(dept ? getDeptColor(dept.color) : '239 84% 67%');
</script>

<div class="h-full overflow-auto">
	{#if dept}
		<div class="border-b border-border px-4 py-3">
			<h1 class="text-lg font-semibold">Events — {dept.name}</h1>
			<p class="text-xs text-muted-foreground">Recent department events from the API.</p>
		</div>
		<EventsTab dept={dept.id} {deptHsl} />
	{:else}
		<div class="flex h-full items-center justify-center">
			<p class="text-sm text-muted-foreground">Department not found.</p>
		</div>
	{/if}
</div>
