<script lang="ts">
	import { postDbSql, type DbSqlExecuteResponse } from '$lib/api';
	import { toast } from 'svelte-sonner';

	let query = $state('SELECT name FROM sqlite_master WHERE type = \'table\' ORDER BY name;');
	let result: DbSqlExecuteResponse | null = $state(null);
	let error = $state('');
	let running = $state(false);

	async function run() {
		error = '';
		result = null;
		running = true;
		try {
			result = await postDbSql(query, true);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			toast.error(error);
		} finally {
			running = false;
		}
	}

	function cellStr(v: unknown): string {
		if (v === null || v === undefined) return '';
		if (typeof v === 'object') return JSON.stringify(v);
		return String(v);
	}
</script>

<div class="flex h-full min-h-0 flex-col gap-2 p-4">
	<div class="flex gap-2">
		<button
			type="button"
			onclick={run}
			disabled={running}
			class="rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground disabled:opacity-50"
		>
			{running ? 'Running…' : 'Run (read-only)'}
		</button>
		<span class="text-[10px] text-muted-foreground self-center">
			Uses PRAGMA query_only — writes rejected
		</span>
	</div>
	<textarea
		bind:value={query}
		rows="8"
		class="min-h-[120px] w-full flex-1 rounded-md border border-border bg-background p-2 font-mono text-xs"
		spellcheck="false"
	></textarea>
	{#if error}
		<p class="text-xs text-destructive whitespace-pre-wrap">{error}</p>
	{/if}
	{#if result}
		<p class="text-[10px] text-muted-foreground">
			{result.row_count} rows · {result.duration_ms} ms
		</p>
		{#if result.columns.length > 0}
			<div class="min-h-0 flex-1 overflow-auto rounded border border-border">
				<table class="w-full text-left text-[10px]">
					<thead class="sticky top-0 bg-secondary">
						<tr>
							{#each result.columns as col}
								<th class="border-b border-border px-2 py-1 font-medium">{col.name}</th>
							{/each}
						</tr>
					</thead>
					<tbody>
						{#each result.rows as row}
							<tr class="border-b border-border/60">
								{#each row as cell}
									<td class="max-w-[280px] truncate px-2 py-1 align-top font-mono">
										{cellStr(cell)}
									</td>
								{/each}
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	{/if}
</div>
