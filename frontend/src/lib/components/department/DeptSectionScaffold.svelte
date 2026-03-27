<script lang="ts">
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import { departments } from '$lib/stores';
	import type { DepartmentDef } from '$lib/api';
	import { getDeptColor } from './colors';

	let {
		children
	}: {
		children: Snippet<[{ dept: DepartmentDef; deptHsl: string }]>;
	} = $props();

	let allDepts: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (allDepts = v));

	let dept = $derived(allDepts.find((d) => d.id === page.params.id));
	let deptHsl = $derived(dept ? getDeptColor(dept.color) : '');
</script>

{#if !dept}
	<div class="flex h-full items-center justify-center">
		<p class="text-sm text-muted-foreground">Department not found.</p>
	</div>
{:else}
	{@render children({ dept, deptHsl })}
{/if}
