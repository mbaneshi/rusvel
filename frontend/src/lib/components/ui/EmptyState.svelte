<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';
	import type { IconName } from '../icons/icons';
	import Icon from '../icons/Icon.svelte';

	let {
		icon,
		title,
		description,
		class: className,
		actions
	}: {
		icon?: IconName;
		title: string;
		description?: string;
		class?: string;
		actions?: Snippet;
	} = $props();
</script>

<div class={cn('flex flex-col items-center justify-center py-12 px-4 text-center', className)}>
	{#if icon}
		<div class="mb-4 rounded-full bg-secondary p-4">
			<Icon name={icon} size={32} class="text-muted-foreground" />
		</div>
	{/if}
	<h3 class="text-lg font-semibold text-foreground">{title}</h3>
	{#if description}
		<p class="mt-1 text-sm text-muted-foreground max-w-sm">{description}</p>
	{/if}
	{#if actions}
		<div class="mt-4">
			{@render actions()}
		</div>
	{/if}
</div>
