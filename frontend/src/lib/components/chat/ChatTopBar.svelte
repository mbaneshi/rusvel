<script lang="ts">
	import { onMount } from 'svelte';
	import { getConfig, updateConfig, getModels, getTools } from '$lib/api';
	import type { ChatConfig, ModelOption, ToolOption } from '$lib/api';
	import { cached } from '$lib/cache';

	let config: ChatConfig = $state({
		model: 'cursor/sonnet-4',
		effort: 'medium',
		max_budget_usd: null,
		permission_mode: 'default',
		allowed_tools: [],
		disallowed_tools: [],
		max_turns: null
	});

	let models: ModelOption[] = $state([]);
	let tools: ToolOption[] = $state([]);
	let showTools = $state(false);
	let saving = $state(false);

	const effortLevels = [
		{ value: 'low', label: 'Low', color: 'bg-gray-500' },
		{ value: 'medium', label: 'Med', color: 'bg-yellow-500' },
		{ value: 'high', label: 'High', color: 'bg-orange-500' },
		{ value: 'max', label: 'Max', color: 'bg-red-500' }
	];

	onMount(async () => {
		try {
			const [cfg, mdls, tls] = await Promise.all([
				cached('global-config', getConfig),
				cached('models', getModels),
				cached('tools', getTools)
			]);
			config = cfg;
			models = mdls;
			tools = tls;
		} catch {
			// Defaults are fine
		}
	});

	async function save() {
		saving = true;
		try {
			config = await updateConfig(config);
		} catch {
			// Silent fail — config will be used next request anyway
		}
		saving = false;
	}

	function setModel(e: globalThis.Event) {
		config.model = (e.target as HTMLSelectElement).value;
		save();
	}

	function setEffort(level: string) {
		config.effort = level;
		save();
	}

	function toggleTool(toolName: string) {
		const idx = config.disallowed_tools.indexOf(toolName);
		if (idx >= 0) {
			config.disallowed_tools = config.disallowed_tools.filter((t) => t !== toolName);
		} else {
			config.disallowed_tools = [...config.disallowed_tools, toolName];
		}
		save();
	}

	function isToolEnabled(toolName: string): boolean {
		return !config.disallowed_tools.includes(toolName);
	}
</script>

<div class="flex items-center gap-3 border-b border-[var(--border)] bg-[var(--card)] px-4 py-2">
	<!-- Model Picker -->
	<div class="flex items-center gap-1.5">
		<span class="text-xs text-[var(--muted-foreground)]">Model</span>
		<select
			value={config.model}
			onchange={setModel}
			class="rounded-md border border-[var(--border)] bg-[var(--secondary)] px-2 py-1 text-xs text-[var(--foreground)] focus:border-[var(--ring)] focus:outline-none"
		>
			{#each models as m}
				<option value={m.value} title={m.description}>{m.label}</option>
			{/each}
		</select>
	</div>

	<!-- Divider -->
	<div class="h-4 w-px bg-[var(--border)]"></div>

	<!-- Effort Level -->
	<div class="flex items-center gap-1">
		<span class="text-xs text-[var(--muted-foreground)]">Effort</span>
		<div class="flex rounded-md border border-[var(--border)] bg-[var(--secondary)]">
			{#each effortLevels as level}
				<button
					onclick={() => setEffort(level.value)}
					class="px-2 py-1 text-xs transition-colors first:rounded-l-md last:rounded-r-md
						{config.effort === level.value
						? 'bg-[var(--primary)] text-white'
						: 'text-[var(--muted-foreground)] hover:text-[var(--foreground)]'}"
				>
					{level.label}
				</button>
			{/each}
		</div>
	</div>

	<!-- Divider -->
	<div class="h-4 w-px bg-[var(--border)]"></div>

	<!-- Tools Toggle -->
	<button
		onclick={() => (showTools = !showTools)}
		class="flex items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--secondary)] px-2 py-1 text-xs text-[var(--muted-foreground)] transition-colors hover:text-[var(--foreground)]
			{showTools ? 'border-[var(--ring)] text-[var(--foreground)]' : ''}"
	>
		<svg
			class="h-3.5 w-3.5"
			viewBox="0 0 16 16"
			fill="none"
			stroke="currentColor"
			stroke-width="1.5"
		>
			<path d="M9.5 2.5L13.5 6.5L6 14H2V10L9.5 2.5Z" stroke-linejoin="round" />
		</svg>
		Tools
		<span class="rounded-full bg-[var(--card)] px-1 text-[10px]">
			{tools.length - config.disallowed_tools.length}/{tools.length}
		</span>
	</button>

	<!-- Spacer -->
	<div class="flex-1"></div>

	<!-- Save indicator -->
	{#if saving}
		<div
			class="h-3 w-3 animate-spin rounded-full border border-[var(--muted-foreground)] border-t-[var(--primary)]"
		></div>
	{/if}

	<!-- Model badge -->
	<span
		class="rounded-full bg-[var(--secondary)] px-2 py-0.5 text-[10px] text-[var(--muted-foreground)]"
	>
		{config.model} · {config.effort}
	</span>
</div>

<!-- Tools panel (expandable) -->
{#if showTools}
	<div class="border-b border-[var(--border)] bg-[var(--card)] px-4 py-3">
		<div class="flex flex-wrap gap-2">
			{#each tools as tool}
				{@const enabled = isToolEnabled(tool.name)}
				<button
					onclick={() => toggleTool(tool.name)}
					title={tool.description}
					class="flex items-center gap-1 rounded-lg border px-2.5 py-1 text-xs transition-colors
						{enabled
						? 'border-[var(--ring)] bg-brand-900/20 text-brand-300'
						: 'border-[var(--border)] bg-[var(--secondary)] text-[var(--muted-foreground)] line-through opacity-60'}"
				>
					{#if enabled}
						<svg class="h-3 w-3" viewBox="0 0 16 16" fill="currentColor"
							><path
								d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.75.75 0 011.06-1.06L6 10.94l6.72-6.72a.75.75 0 011.06 0z"
							/></svg
						>
					{:else}
						<svg class="h-3 w-3" viewBox="0 0 16 16" fill="currentColor"
							><path
								d="M3.72 3.72a.75.75 0 011.06 0L8 6.94l3.22-3.22a.75.75 0 111.06 1.06L9.06 8l3.22 3.22a.75.75 0 11-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 01-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 010-1.06z"
							/></svg
						>
					{/if}
					{tool.name}
				</button>
			{/each}
		</div>
	</div>
{/if}
