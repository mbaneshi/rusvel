<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { getAgents, createAgent, deleteAgent } from '$lib/api';
	import type { Agent } from '$lib/api';

	let {
		dept,
		deptHsl
	}: {
		dept: string;
		deptHsl: string;
	} = $props();

	let agents: Agent[] = $state([]);
	let showCreate = $state(false);
	let newName = $state('');
	let newRole = $state('');
	let newModel = $state('sonnet');
	let newInstructions = $state('');

	onMount(() => loadAgents());

	async function loadAgents() {
		try {
			agents = await getAgents(dept);
		} catch {
			agents = [];
		}
	}

	async function handleCreateAgent() {
		if (!newName.trim()) return;
		try {
			await createAgent({
				name: newName.trim(),
				role: newRole,
				model: newModel,
				instructions: newInstructions,
				metadata: { engine: dept }
			});
			newName = '';
			newRole = '';
			newInstructions = '';
			showCreate = false;
			await loadAgents();
			toast.success('Agent created');
		} catch (e) {
			toast.error(`Failed to create agent: ${e instanceof Error ? e.message : e}`);
		}
	}

	async function handleDeleteAgent(id: string) {
		try {
			await deleteAgent(id);
			await loadAgents();
			toast.success('Agent deleted');
		} catch (e) {
			toast.error(`Failed to delete agent: ${e instanceof Error ? e.message : e}`);
		}
	}
</script>

<div class="p-3 space-y-2">
	<button
		onclick={() => (showCreate = !showCreate)}
		class="w-full rounded-lg border border-dashed border-border py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
		style="border-color: hsl({deptHsl} / 0.3)">+ New Agent</button
	>

	{#if showCreate}
		<div class="rounded-lg bg-secondary p-3 space-y-2">
			<input
				bind:value={newName}
				placeholder="Agent name"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<input
				bind:value={newRole}
				placeholder="Role description"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<select
				bind:value={newModel}
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground"
			>
				<option value="sonnet">Sonnet</option>
				<option value="opus">Opus</option>
				<option value="haiku">Haiku</option>
			</select>
			<textarea
				bind:value={newInstructions}
				placeholder="System prompt / instructions"
				rows="3"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none resize-none"
			></textarea>
			<button
				onclick={handleCreateAgent}
				class="w-full rounded-md py-1 text-xs font-medium text-white"
				style="background: hsl({deptHsl})">Create</button
			>
		</div>
	{/if}

	{#each agents as agent}
		<div class="rounded-lg bg-secondary p-2.5 group">
			<div class="flex items-center justify-between mb-1">
				<span class="text-xs font-medium text-foreground">{agent.name}</span>
				<div class="flex items-center gap-1">
					<span
						class="rounded px-1.5 py-0.5 text-[9px]"
						style="background: hsl({deptHsl} / 0.2); color: hsl({deptHsl})"
						>{agent.default_model.model}</span
					>
					<button
						onclick={() => handleDeleteAgent(agent.id)}
						class="hidden group-hover:block text-muted-foreground hover:text-red-400 text-[10px]"
						>x</button
					>
				</div>
			</div>
			<p class="text-[10px] text-muted-foreground">{agent.role}</p>
		</div>
	{/each}

	{#if agents.length === 0 && !showCreate}
		<p class="text-center text-[10px] text-muted-foreground py-2">No agents. Create one above.</p>
	{/if}
</div>
