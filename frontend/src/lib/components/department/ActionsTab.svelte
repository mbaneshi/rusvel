<script lang="ts">
	import { pendingCommand } from '$lib/stores';
	import { streamCapability } from '$lib/api';

	let {
		dept,
		quickActions = [],
		deptHsl
	}: {
		dept: string;
		quickActions: { label: string; prompt: string }[];
		deptHsl: string;
	} = $props();

	let showCapability = $state(false);
	let capabilityInput = $state('');

	function sendQuickAction(prompt: string) {
		pendingCommand.set({ prompt });
	}
</script>

<div class="p-3 space-y-2">
	<!-- Build Capability -->
	<button
		onclick={() => (showCapability = !showCapability)}
		class="w-full rounded-lg border border-dashed px-3 py-2 text-left transition-colors hover:opacity-80"
		style="border-color: hsl({deptHsl} / 0.3); background: hsl({deptHsl} / 0.1)"
	>
		<p class="text-xs font-medium" style="color: hsl({deptHsl})">Build Capability</p>
		<p class="text-[10px] text-muted-foreground">
			Describe what you need — AI discovers & installs
		</p>
	</button>

	{#if showCapability}
		<div class="rounded-lg bg-secondary p-3 space-y-2">
			<textarea
				bind:value={capabilityInput}
				placeholder="e.g. I need to scrape job postings and score them"
				rows="3"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground focus:outline-none resize-none"
			></textarea>
			<button
				onclick={() => {
					if (capabilityInput.trim()) {
						sendQuickAction('!capability ' + capabilityInput.trim());
						capabilityInput = '';
						showCapability = false;
					}
				}}
				disabled={!capabilityInput.trim()}
				class="w-full rounded-md py-1 text-xs font-medium text-white disabled:opacity-40 disabled:cursor-not-allowed"
				style="background: hsl({deptHsl})">Search & Build</button
			>
		</div>
	{/if}

	<!-- Quick Actions -->
	{#each quickActions as action}
		<button
			onclick={() => sendQuickAction(action.prompt)}
			class="w-full rounded-lg bg-secondary px-3 py-2 text-left transition-colors hover:opacity-80 group"
		>
			<p
				class="text-xs font-medium text-foreground group-hover:opacity-90"
				style="--accent: hsl({deptHsl})"
			>
				{action.label}
			</p>
		</button>
	{/each}

	{#if quickActions.length === 0}
		<p class="text-center text-[10px] text-muted-foreground py-2">No quick actions configured.</p>
	{/if}
</div>
