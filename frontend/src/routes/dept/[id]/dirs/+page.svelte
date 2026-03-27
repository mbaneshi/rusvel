<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { departments } from '$lib/stores';
	import { getDeptConfig } from '$lib/api';
	import type { DepartmentDef, DepartmentConfig } from '$lib/api';
	import DirsTab from '$lib/components/department/DirsTab.svelte';
	let allDepts: DepartmentDef[] = $state([]);
	departments.subscribe((v) => (allDepts = v));

	let dept = $derived(allDepts.find((d) => d.id === page.params.id));
	let config = $state<DepartmentConfig | null>(null);

	onMount(() => {
		const id = page.params.id;
		if (!id) return;
		void getDeptConfig(id)
			.then((c) => {
				config = c;
			})
			.catch(() => {});
	});
</script>

{#if !dept}
	<div class="flex h-full items-center justify-center">
		<p class="text-sm text-muted-foreground">Department not found.</p>
	</div>
{:else}
	<div class="h-full overflow-auto">
		<DirsTab dept={dept.id} bind:config />
	</div>
{/if}
