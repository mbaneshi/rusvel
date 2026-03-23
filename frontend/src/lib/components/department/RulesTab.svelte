<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { getRules, createRule, updateRule, deleteRule } from '$lib/api';
	import type { Rule } from '$lib/api';

	let { dept, deptHsl }: { dept: string; deptHsl: string } = $props();

	let rules: Rule[] = $state([]);
	let showCreate = $state(false);
	let newName = $state('');
	let newContent = $state('');

	onMount(() => {
		loadRules();
	});

	async function loadRules() {
		try {
			rules = await getRules(dept);
		} catch {
			rules = [];
		}
	}

	async function handleCreateRule() {
		if (!newName.trim()) return;
		try {
			await createRule({
				id: '',
				name: newName.trim(),
				content: newContent,
				enabled: true,
				metadata: { engine: dept }
			});
			newName = '';
			newContent = '';
			showCreate = false;
			await loadRules();
			toast.success('Rule created');
		} catch (e) {
			toast.error(`Failed to create rule: ${e instanceof Error ? e.message : e}`);
		}
	}

	async function handleToggleRule(rule: Rule) {
		try {
			await updateRule(rule.id, { ...rule, enabled: !rule.enabled });
			await loadRules();
		} catch (e) {
			toast.error(`Failed to update rule: ${e instanceof Error ? e.message : e}`);
		}
	}

	async function handleDeleteRule(id: string) {
		try {
			await deleteRule(id);
			await loadRules();
			toast.success('Rule deleted');
		} catch (e) {
			toast.error(`Failed to delete rule: ${e instanceof Error ? e.message : e}`);
		}
	}
</script>

<div class="p-3 space-y-2">
	<button
		onclick={() => (showCreate = !showCreate)}
		class="w-full rounded-lg border border-dashed border-border py-1.5 text-xs text-muted-foreground hover:border-foreground hover:text-foreground"
	>
		+ New Rule
	</button>

	{#if showCreate}
		<div class="rounded-lg bg-secondary p-3 space-y-2">
			<input
				bind:value={newName}
				placeholder="Rule name"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<textarea
				bind:value={newContent}
				placeholder="Rule content (injected into system prompt)"
				rows="3"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none resize-none"
			></textarea>
			<button
				onclick={handleCreateRule}
				style="background: hsl({deptHsl})"
				class="w-full rounded-md py-1 text-xs font-medium text-white hover:opacity-90"
				>Create</button
			>
		</div>
	{/if}

	{#each rules as rule}
		<div class="rounded-lg bg-secondary p-2.5 group">
			<div class="flex items-center justify-between mb-1">
				<span
					class="text-xs font-medium text-foreground {!rule.enabled
						? 'line-through opacity-50'
						: ''}">{rule.name}</span
				>
				<div class="flex items-center gap-1">
					<button
						onclick={() => handleToggleRule(rule)}
						class="rounded px-1.5 py-0.5 text-[9px]"
						style={rule.enabled ? `background: hsl(${deptHsl} / 0.2); color: hsl(${deptHsl})` : ''}
						class:bg-card={!rule.enabled}
						class:text-muted-foreground={!rule.enabled}
					>
						{rule.enabled ? 'on' : 'off'}
					</button>
					<button
						onclick={() => handleDeleteRule(rule.id)}
						class="hidden group-hover:block text-muted-foreground hover:text-red-400 text-[10px]"
						>x</button
					>
				</div>
			</div>
			<p class="text-[10px] text-muted-foreground {!rule.enabled ? 'opacity-50' : ''}">
				{rule.content.slice(0, 80)}{rule.content.length > 80 ? '...' : ''}
			</p>
		</div>
	{/each}

	{#if rules.length === 0 && !showCreate}
		<p class="text-center text-[10px] text-muted-foreground py-2">
			No rules. Rules get injected into system prompts.
		</p>
	{/if}
</div>
