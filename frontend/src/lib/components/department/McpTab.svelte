<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { getMcpServers, createMcpServer, deleteMcpServer } from '$lib/api';
	import type { McpServer } from '$lib/api';

	let { dept }: { dept: string } = $props();

	let servers: McpServer[] = $state([]);
	let showCreate = $state(false);
	let newName = $state('');
	let newType = $state('stdio');
	let newCommand = $state('');

	onMount(() => {
		loadServers();
	});

	async function loadServers() {
		try {
			servers = await getMcpServers(dept);
		} catch {
			servers = [];
		}
	}

	async function handleCreate() {
		if (!newName.trim()) return;
		try {
			await createMcpServer({
				id: '',
				name: newName.trim(),
				description: '',
				server_type: newType,
				command: newCommand || null,
				args: [],
				url: null,
				env: {},
				enabled: true,
				metadata: { engine: dept }
			});
			newName = '';
			newCommand = '';
			showCreate = false;
			await loadServers();
			toast.success('MCP server added');
		} catch (e) {
			toast.error(`Failed to add MCP server: ${e instanceof Error ? e.message : e}`);
		}
	}

	async function handleDelete(id: string) {
		try {
			await deleteMcpServer(id);
			await loadServers();
			toast.success('MCP server removed');
		} catch (e) {
			toast.error(`Failed to remove MCP server: ${e instanceof Error ? e.message : e}`);
		}
	}
</script>

<div class="p-3 space-y-2">
	<button
		onclick={() => (showCreate = !showCreate)}
		class="w-full rounded-lg border border-dashed border-border py-1.5 text-xs text-muted-foreground hover:border-foreground hover:text-foreground"
	>
		+ Add MCP Server
	</button>

	{#if showCreate}
		<div class="rounded-lg bg-secondary p-3 space-y-2">
			<input
				bind:value={newName}
				placeholder="Server name"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<select
				bind:value={newType}
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground"
			>
				<option value="stdio">stdio</option>
				<option value="http">HTTP</option>
				<option value="sse">SSE</option>
				<option value="ws">WebSocket</option>
			</select>
			<input
				bind:value={newCommand}
				placeholder="Command (e.g. npx @server/mcp)"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<button
				onclick={handleCreate}
				class="w-full rounded-md bg-primary py-1 text-xs font-medium text-white hover:opacity-90"
				>Create</button
			>
		</div>
	{/if}

	{#each servers as server}
		<div class="rounded-lg bg-secondary p-2.5 group">
			<div class="flex items-center justify-between mb-1">
				<span class="text-xs font-medium text-foreground">{server.name}</span>
				<div class="flex items-center gap-1">
					<span class="rounded bg-card px-1.5 py-0.5 text-[9px] text-muted-foreground"
						>{server.server_type}</span
					>
					<button
						onclick={() => handleDelete(server.id)}
						class="hidden group-hover:block text-muted-foreground hover:text-red-400 text-[10px]"
						>x</button
					>
				</div>
			</div>
			<p class="text-[10px] font-mono text-muted-foreground">
				{server.command || server.url || '\u2014'}
			</p>
		</div>
	{/each}

	{#if servers.length === 0 && !showCreate}
		<p class="text-center text-[10px] text-muted-foreground py-2">
			No MCP servers. Add one to extend capabilities.
		</p>
	{/if}
</div>
