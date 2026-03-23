<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	type Level = 1 | 2 | 3 | 4 | 5 | 6;

	let {
		level = 2,
		class: className,
		children,
		...rest
	}: {
		level?: Level;
		class?: string;
		children: Snippet;
		[key: string]: unknown;
	} = $props();

	const styles: Record<Level, string> = {
		1: 'text-3xl font-bold tracking-tight',
		2: 'text-2xl font-bold tracking-tight',
		3: 'text-xl font-semibold',
		4: 'text-lg font-semibold',
		5: 'text-base font-semibold',
		6: 'text-sm font-semibold uppercase tracking-wider text-[var(--muted-foreground)]'
	};

	let tag = $derived(`h${level}`);
</script>

<svelte:element
	this={tag}
	class={cn(styles[level], 'text-[var(--foreground)]', className)}
	{...rest}
>
	{@render children()}
</svelte:element>
