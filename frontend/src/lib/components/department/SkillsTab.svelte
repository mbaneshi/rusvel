<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { getSkills, createSkill, deleteSkill } from '$lib/api';
	import type { Skill } from '$lib/api';
	import { pendingCommand } from '$lib/stores';

	let {
		dept,
		deptHsl
	}: {
		dept: string;
		deptHsl: string;
	} = $props();

	let skills: Skill[] = $state([]);
	let showCreate = $state(false);
	let newName = $state('');
	let newDesc = $state('');
	let newPrompt = $state('');

	onMount(() => loadSkills());

	async function loadSkills() {
		try {
			skills = await getSkills(dept);
		} catch {
			skills = [];
		}
	}

	async function handleCreateSkill() {
		if (!newName.trim()) return;
		try {
			await createSkill({
				id: '',
				name: newName.trim(),
				description: newDesc,
				prompt_template: newPrompt,
				metadata: { engine: dept }
			});
			newName = '';
			newDesc = '';
			newPrompt = '';
			showCreate = false;
			await loadSkills();
			toast.success('Skill created');
		} catch (e) {
			toast.error(`Failed to create skill: ${e instanceof Error ? e.message : e}`);
		}
	}

	async function handleDeleteSkill(id: string) {
		try {
			await deleteSkill(id);
			await loadSkills();
			toast.success('Skill deleted');
		} catch (e) {
			toast.error(`Failed to delete skill: ${e instanceof Error ? e.message : e}`);
		}
	}

	function useSkill(skill: Skill) {
		pendingCommand.set({ prompt: '/' + skill.name.toLowerCase().replace(/ /g, '-') });
	}
</script>

<div class="p-3 space-y-2">
	<button
		onclick={() => (showCreate = !showCreate)}
		class="w-full rounded-lg border border-dashed border-border py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
		style="border-color: hsl({deptHsl} / 0.3)">+ New Skill</button
	>

	{#if showCreate}
		<div class="rounded-lg bg-secondary p-3 space-y-2">
			<input
				bind:value={newName}
				placeholder="Skill name (e.g. /wire-engine)"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<input
				bind:value={newDesc}
				placeholder="Description"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none"
			/>
			<textarea
				bind:value={newPrompt}
				placeholder="Prompt template"
				rows="3"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none resize-none"
			></textarea>
			<button
				onclick={handleCreateSkill}
				class="w-full rounded-md py-1 text-xs font-medium text-white"
				style="background: hsl({deptHsl})">Create</button
			>
		</div>
	{/if}

	{#each skills as skill}
		<div
			class="rounded-lg bg-secondary p-2.5 transition-colors hover:opacity-80 group cursor-pointer"
			role="button"
			tabindex="0"
			onclick={() => useSkill(skill)}
			onkeydown={(e) => {
				if (e.key === 'Enter') useSkill(skill);
			}}
		>
			<div class="flex items-center justify-between">
				<span class="text-xs font-mono font-medium" style="color: hsl({deptHsl})">{skill.name}</span
				>
				<button
					onclick={(e) => {
						e.stopPropagation();
						handleDeleteSkill(skill.id);
					}}
					class="hidden group-hover:block text-muted-foreground hover:text-red-400 text-[10px]"
					>x</button
				>
			</div>
			<p class="text-[10px] text-muted-foreground">{skill.description}</p>
		</div>
	{/each}

	{#if skills.length === 0 && !showCreate}
		<p class="text-center text-[10px] text-muted-foreground py-2">No skills. Create one above.</p>
	{/if}
</div>
