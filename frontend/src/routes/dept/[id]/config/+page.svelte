<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { getDeptConfig } from '$lib/api';
	import type { DepartmentConfig } from '$lib/api';

	let config: DepartmentConfig | null = $state(null);
	let err = $state('');

	onMount(() => {
		const id = page.params.id;
		if (!id) return;
		void getDeptConfig(id)
			.then((c) => {
				config = c;
			})
			.catch((e) => {
				err = e instanceof Error ? e.message : String(e);
			});
	});
</script>

<div class="h-full overflow-auto p-4">
	<h1 class="mb-4 text-lg font-semibold">Department config</h1>
	{#if err}
		<p class="text-sm text-destructive">{err}</p>
	{:else if !config}
		<p class="text-sm text-muted-foreground">Loading…</p>
	{:else}
		<dl class="grid max-w-2xl gap-3 text-sm">
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Engine</dt>
				<dd class="font-mono text-xs">{config.engine}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Model</dt>
				<dd class="font-mono text-xs">{config.model}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Effort</dt>
				<dd>{config.effort}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Permission mode</dt>
				<dd>{config.permission_mode}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Max budget (USD)</dt>
				<dd>{config.max_budget_usd ?? '—'}</dd>
			</div>
			<div class="grid grid-cols-[minmax(0,10rem)_1fr] gap-2 border-b border-border py-2">
				<dt class="text-muted-foreground">Max turns</dt>
				<dd>{config.max_turns ?? '—'}</dd>
			</div>
			<div class="grid grid-cols-1 gap-2 py-2">
				<dt class="text-muted-foreground">Allowed tools</dt>
				<dd class="font-mono text-xs">{config.allowed_tools.join(', ') || '—'}</dd>
			</div>
			<div class="grid grid-cols-1 gap-2 py-2">
				<dt class="text-muted-foreground">Disallowed tools</dt>
				<dd class="font-mono text-xs">{config.disallowed_tools.join(', ') || '—'}</dd>
			</div>
			<div class="grid grid-cols-1 gap-2 py-2">
				<dt class="text-muted-foreground">Add dirs</dt>
				<dd class="font-mono text-xs">{config.add_dirs.join(', ') || '—'}</dd>
			</div>
			<div class="grid grid-cols-1 gap-2 py-2">
				<dt class="text-muted-foreground">System prompt</dt>
				<dd class="whitespace-pre-wrap rounded-md bg-secondary p-3 font-mono text-xs">{config.system_prompt}</dd>
			</div>
		</dl>
	{/if}
</div>
