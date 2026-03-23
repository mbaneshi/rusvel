<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { updateDeptConfig } from '$lib/api';
	import type { DepartmentConfig } from '$lib/api';

	let { dept, config = $bindable(null) }: { dept: string; config: DepartmentConfig | null } =
		$props();

	async function addDir() {
		if (!config) return;
		const dir = prompt('Add directory path:');
		if (dir && !config.add_dirs.includes(dir)) {
			config.add_dirs = [...config.add_dirs, dir];
			try {
				config = await updateDeptConfig(dept, config);
				toast.success('Directory added');
			} catch (e) {
				toast.error(`Failed to add directory: ${e instanceof Error ? e.message : e}`);
			}
		}
	}

	async function removeDir(dir: string) {
		if (!config) return;
		config.add_dirs = config.add_dirs.filter((d) => d !== dir);
		try {
			config = await updateDeptConfig(dept, config);
			toast.success('Directory removed');
		} catch (e) {
			toast.error(`Failed to remove directory: ${e instanceof Error ? e.message : e}`);
		}
	}
</script>

<div class="p-3 space-y-2">
	<p class="text-[10px] text-muted-foreground">Working directories (--add-dir).</p>
	{#if config}
		{#each config.add_dirs as dir}
			<div class="flex items-center justify-between rounded-lg bg-secondary px-3 py-2">
				<span class="text-xs font-mono text-foreground">{dir}</span>
				<button
					onclick={() => removeDir(dir)}
					class="text-muted-foreground hover:text-destructive text-xs">x</button
				>
			</div>
		{/each}
		<button
			onclick={addDir}
			class="w-full rounded-lg border border-dashed border-border py-2 text-xs text-muted-foreground hover:text-foreground"
		>
			+ Add directory
		</button>
	{/if}
</div>
