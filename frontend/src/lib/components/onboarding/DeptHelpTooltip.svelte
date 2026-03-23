<script lang="ts">
	let {
		dept,
		description,
		prompts = [],
	}: {
		dept: string;
		description: string;
		prompts: string[];
	} = $props();

	let showHelp = $state(false);

	function sendPrompt(prompt: string) {
		showHelp = false;
		document.dispatchEvent(new CustomEvent('dept-quick-action', { detail: { prompt }, bubbles: true }));
	}
</script>

<div class="relative">
	<button
		onclick={() => (showHelp = !showHelp)}
		class="rounded-md p-1 text-[var(--r-fg-subtle)] hover:bg-[var(--r-bg-raised)] hover:text-[var(--r-fg-default)] {showHelp ? 'bg-[var(--r-bg-raised)] text-[var(--r-fg-default)]' : ''}"
		title="Help"
	>
		<svg class="h-4 w-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
			<circle cx="8" cy="8" r="6" />
			<path d="M6 6.5a2 2 0 013.5 1.5c0 1-1.5 1.5-1.5 1.5" stroke-linecap="round" />
			<circle cx="8" cy="11.5" r="0.5" fill="currentColor" />
		</svg>
	</button>

	{#if showHelp}
		<div class="absolute left-0 top-full z-40 mt-1 w-64 rounded-lg border border-[var(--r-border-default)] bg-[var(--r-bg-surface)] p-3 shadow-xl">
			<p class="text-xs text-[var(--r-fg-muted)] leading-relaxed">{description}</p>
			{#if prompts.length > 0}
				<div class="mt-2 space-y-1">
					<p class="text-[10px] font-medium uppercase tracking-wider text-[var(--r-fg-subtle)]">Try asking</p>
					{#each prompts as prompt}
						<button
							onclick={() => sendPrompt(prompt)}
							class="block w-full rounded-md px-2 py-1 text-left text-[11px] text-indigo-400 hover:bg-[var(--r-bg-raised)]"
						>
							"{prompt}"
						</button>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>
