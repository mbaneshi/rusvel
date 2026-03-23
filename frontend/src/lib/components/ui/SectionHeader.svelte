<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	let {
		title,
		description = '',
		actions,
		class: className = ''
	}: {
		title: string;
		description?: string;
		actions?: Snippet;
		class?: string;
	} = $props();
</script>

<div class={cn('mb-4 flex items-center justify-between', className)}>
	<div>
		<h3 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">{title}</h3>
		{#if description}
			<p class="mt-0.5 text-xs text-muted-foreground">{description}</p>
		{/if}
	</div>
	{#if actions}
		<div class="flex items-center gap-2">
			{@render actions()}
		</div>
	{/if}
</div>
