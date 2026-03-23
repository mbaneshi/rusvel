<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	let {
		text,
		position = 'top',
		children,
		class: className
	}: {
		text: string;
		position?: 'top' | 'bottom' | 'left' | 'right';
		children: Snippet;
		class?: string;
	} = $props();

	const positions: Record<string, string> = {
		top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
		bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
		left: 'right-full top-1/2 -translate-y-1/2 mr-2',
		right: 'left-full top-1/2 -translate-y-1/2 ml-2'
	};
</script>

<div class={cn('relative group inline-flex', className)}>
	{@render children()}
	<div
		class={cn(
			'absolute z-50 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity',
			'whitespace-nowrap rounded-md bg-popover border border-border',
			'px-2 py-1 text-xs text-popover-foreground shadow-lg',
			positions[position]
		)}
		role="tooltip"
	>
		{text}
	</div>
</div>
