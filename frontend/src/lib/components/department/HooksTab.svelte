<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { getHooks, createHook, updateHook, deleteHook } from '$lib/api';
	import type { Hook } from '$lib/api';

	let { dept }: { dept: string } = $props();

	let hooks: Hook[] = $state([]);
	let showCreate = $state(false);
	let newName = $state('');
	let newEvent = $state('PostToolUse');
	let newAction = $state('');

	onMount(() => {
		loadHooks();
	});

	async function loadHooks() {
		try {
			hooks = await getHooks(dept);
		} catch {
			hooks = [];
		}
	}

	async function handleCreate() {
		if (!newName.trim()) return;
		try {
			await createHook({
				id: '',
				name: newName.trim(),
				event: newEvent,
				matcher: '',
				hook_type: 'command',
				action: newAction,
				enabled: true,
				metadata: { engine: dept }
			});
			newName = '';
			newAction = '';
			showCreate = false;
			await loadHooks();
			toast.success('Hook created');
		} catch (e) {
			toast.error(`Failed to create hook: ${e instanceof Error ? e.message : e}`);
		}
	}

	async function handleToggle(hook: Hook) {
		try {
			await updateHook(hook.id, { ...hook, enabled: !hook.enabled });
			await loadHooks();
		} catch (e) {
			toast.error(`Failed to update hook: ${e instanceof Error ? e.message : e}`);
		}
	}

	async function handleDelete(id: string) {
		try {
			await deleteHook(id);
			await loadHooks();
			toast.success('Hook deleted');
		} catch (e) {
			toast.error(`Failed to delete hook: ${e instanceof Error ? e.message : e}`);
		}
	}
</script>

<div class="p-3 space-y-2">
	<button
		onclick={() => (showCreate = !showCreate)}
		class="w-full rounded-lg border border-dashed border-border py-1.5 text-xs text-muted-foreground hover:border-foreground hover:text-foreground"
	>
		+ Add Hook
	</button>

	{#if showCreate}
		<div class="rounded-lg bg-secondary p-3 space-y-2">
			<input
				bind:value={newName}
				placeholder="Hook name"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<select
				bind:value={newEvent}
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground"
			>
				<option value="PreToolUse">PreToolUse</option>
				<option value="PostToolUse">PostToolUse</option>
				<option value="SessionStart">SessionStart</option>
				<option value="Stop">Stop</option>
				<option value="TaskCompleted">TaskCompleted</option>
				<option value="UserPromptSubmit">UserPromptSubmit</option>
			</select>
			<input
				bind:value={newAction}
				placeholder="Shell command to run"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<button
				onclick={handleCreate}
				class="w-full rounded-md bg-primary py-1 text-xs font-medium text-white hover:opacity-90"
				>Create</button
			>
		</div>
	{/if}

	{#each hooks as hook}
		<div class="rounded-lg bg-secondary p-2.5 group">
			<div class="flex items-center justify-between mb-1">
				<span
					class="text-xs font-medium text-foreground {!hook.enabled
						? 'line-through opacity-50'
						: ''}">{hook.name}</span
				>
				<div class="flex items-center gap-1">
					<button
						onclick={() => handleToggle(hook)}
						class="rounded px-1.5 py-0.5 text-[9px] {hook.enabled
							? 'bg-green-900/30 text-green-400'
							: 'bg-card text-muted-foreground'}"
					>
						{hook.enabled ? 'on' : 'off'}
					</button>
					<button
						onclick={() => handleDelete(hook.id)}
						class="hidden group-hover:block text-muted-foreground hover:text-red-400 text-[10px]"
						>x</button
					>
				</div>
			</div>
			<p class="text-[10px] text-muted-foreground">
				<span class="font-mono">{hook.event}</span> → {hook.action.slice(0, 50)}{hook.action
					.length > 50
					? '...'
					: ''}
			</p>
		</div>
	{/each}

	{#if hooks.length === 0 && !showCreate}
		<p class="text-center text-[10px] text-muted-foreground py-2">
			No hooks. Hooks automate lifecycle events.
		</p>
	{/if}
</div>
