<script lang="ts">
	import { onMount } from 'svelte';
	import { getDbTables, getDbTableSchema, type DbTableInfo, type DbTableSummary } from '$lib/api';
	import { toast } from 'svelte-sonner';

	let tables: DbTableSummary[] = $state([]);
	let expanded = $state<Record<string, boolean>>({});
	let detailCache = $state<Record<string, DbTableInfo>>({});
	let loading = $state(true);

	onMount(async () => {
		try {
			tables = await getDbTables();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load tables');
		} finally {
			loading = false;
		}
	});

	async function toggle(name: string) {
		const next = !expanded[name];
		expanded = { ...expanded, [name]: next };
		if (next && !detailCache[name]) {
			try {
				detailCache = { ...detailCache, [name]: await getDbTableSchema(name) };
			} catch (e) {
				toast.error(e instanceof Error ? e.message : 'Schema load failed');
			}
		}
	}
</script>

<div class="p-4 space-y-2 max-w-4xl">
	{#if loading}
		<p class="text-xs text-muted-foreground">Loading schema…</p>
	{:else if tables.length === 0}
		<p class="text-xs text-muted-foreground">No user tables found.</p>
	{:else}
		{#each tables as t}
			<div class="rounded-lg border border-border bg-card overflow-hidden">
				<button
					type="button"
					onclick={() => toggle(t.name)}
					class="flex w-full items-center justify-between px-3 py-2 text-left text-xs font-medium hover:bg-secondary/80"
				>
					<span>{t.name}</span>
					<span class="text-[10px] text-muted-foreground">{t.row_count} rows</span>
				</button>
				{#if expanded[t.name] && detailCache[t.name]}
					{@const info = detailCache[t.name]}
					<div class="border-t border-border px-3 py-2 space-y-2 text-[10px]">
						<div>
							<p class="font-medium text-foreground mb-1">Columns</p>
							<ul class="space-y-0.5 text-muted-foreground">
								{#each info.columns as c}
									<li>
										<span class="text-foreground">{c.name}</span>
										{c.col_type}
										{c.nullable ? 'NULL' : 'NOT NULL'}
										{#if c.primary_key}<span class="text-primary">PK</span>{/if}
									</li>
								{/each}
							</ul>
						</div>
						{#if info.indexes.length > 0}
							<div>
								<p class="font-medium text-foreground mb-1">Indexes</p>
								<ul class="text-muted-foreground">
									{#each info.indexes as ix}
										<li>{ix.name} ({ix.columns.join(', ')}){ix.unique ? ' UNIQUE' : ''}</li>
									{/each}
								</ul>
							</div>
						{/if}
						{#if info.foreign_keys.length > 0}
							<div>
								<p class="font-medium text-foreground mb-1">Foreign keys</p>
								<ul class="text-muted-foreground">
									{#each info.foreign_keys as fk}
										<li>{fk.from_column} → {fk.to_table}.{fk.to_column}</li>
									{/each}
								</ul>
							</div>
						{/if}
					</div>
				{/if}
			</div>
		{/each}
	{/if}
</div>
