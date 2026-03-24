<script lang="ts">
	import { onMount } from 'svelte';
	import {
		getDbTables,
		getDbTableRows,
		type DbRowsResponse,
		type DbTableSummary
	} from '$lib/api';
	import { toast } from 'svelte-sonner';

	let tables: DbTableSummary[] = $state([]);
	let selected = $state<string>('');
	let limit = $state(50);
	let offset = $state(0);
	let order = $state('');
	let data: DbRowsResponse | null = $state(null);
	let loading = $state(true);
	let rowsLoading = $state(false);

	onMount(async () => {
		try {
			tables = await getDbTables();
			if (tables.length > 0) selected = tables[0].name;
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load tables');
		} finally {
			loading = false;
		}
	});

	async function loadRows() {
		if (!selected) return;
		rowsLoading = true;
		data = null;
		try {
			data = await getDbTableRows(selected, {
				limit,
				offset,
				order: order.trim() || undefined
			});
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load rows');
		} finally {
			rowsLoading = false;
		}
	}

	$effect(() => {
		if (!loading && selected) {
			void loadRows();
		}
	});

	function cellStr(v: unknown): string {
		if (v === null || v === undefined) return '';
		if (typeof v === 'object') return JSON.stringify(v);
		return String(v);
	}
</script>

<div class="flex h-full min-h-0 flex-col md:flex-row">
	<aside class="w-full shrink-0 border-b border-border md:w-48 md:border-b-0 md:border-r p-2 overflow-y-auto">
		{#if loading}
			<p class="text-[10px] text-muted-foreground p-1">Loading…</p>
		{:else}
			{#each tables as t}
				<button
					type="button"
					onclick={() => {
						selected = t.name;
						offset = 0;
					}}
					class="w-full rounded px-2 py-1 text-left text-[10px] truncate
					{selected === t.name ? 'bg-secondary font-medium' : 'hover:bg-secondary/50'}"
				>
					{t.name}
					<span class="text-muted-foreground">({t.row_count})</span>
				</button>
			{/each}
		{/if}
	</aside>
	<div class="min-w-0 flex-1 flex flex-col p-3 gap-2">
		<div class="flex flex-wrap items-end gap-2 text-[10px]">
			<label class="flex flex-col gap-0.5">
				<span class="text-muted-foreground">Limit</span>
				<input
					type="number"
					bind:value={limit}
					min="1"
					max="500"
					class="w-16 rounded border border-border bg-background px-1 py-0.5"
				/>
			</label>
			<label class="flex flex-col gap-0.5">
				<span class="text-muted-foreground">Offset</span>
				<input
					type="number"
					bind:value={offset}
					min="0"
					class="w-20 rounded border border-border bg-background px-1 py-0.5"
				/>
			</label>
			<label class="flex flex-col gap-0.5 min-w-[120px] flex-1">
				<span class="text-muted-foreground">Order (col or col.desc)</span>
				<input
					type="text"
					bind:value={order}
					placeholder="e.g. created_at.desc"
					class="rounded border border-border bg-background px-1 py-0.5"
				/>
			</label>
			<button
				type="button"
				onclick={() => loadRows()}
				class="rounded-md bg-primary px-2 py-1 text-primary-foreground"
			>
				Refresh
			</button>
		</div>
		{#if rowsLoading}
			<p class="text-xs text-muted-foreground">Loading rows…</p>
		{:else if data}
			<p class="text-[10px] text-muted-foreground">
				Showing {data.row_count} of {data.table_row_count} rows
			</p>
			<div class="min-h-0 flex-1 overflow-auto rounded border border-border">
				<table class="w-full text-left text-[10px]">
					<thead class="sticky top-0 bg-secondary">
						<tr>
							{#each data.columns as col}
								<th class="border-b border-border px-2 py-1 font-medium whitespace-nowrap">
									{col.name}
								</th>
							{/each}
						</tr>
					</thead>
					<tbody>
						{#each data.rows as row}
							<tr class="border-b border-border/60 hover:bg-secondary/30">
								{#each row as cell}
									<td class="max-w-[240px] truncate px-2 py-1 align-top font-mono">
										{cellStr(cell)}
									</td>
								{/each}
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
</div>
