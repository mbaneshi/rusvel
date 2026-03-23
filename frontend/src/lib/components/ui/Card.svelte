<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	type Variant = 'default' | 'bordered' | 'raised' | 'ghost';

	let {
		variant = 'default',
		padding = 'md',
		class: className,
		children,
		header,
		footer,
		...rest
	}: {
		variant?: Variant;
		padding?: 'none' | 'sm' | 'md' | 'lg';
		class?: string;
		children?: Snippet;
		header?: Snippet;
		footer?: Snippet;
		[key: string]: unknown;
	} = $props();

	const variants: Record<Variant, string> = {
		default: 'bg-card text-card-foreground border border-border shadow-sm',
		bordered: 'bg-transparent border border-border',
		raised: 'bg-card text-card-foreground shadow-lg',
		ghost: 'bg-transparent'
	};

	const paddings: Record<string, string> = {
		none: '',
		sm: 'p-3',
		md: 'p-5',
		lg: 'p-6'
	};
</script>

<div
	class={cn('rounded-xl', variants[variant], !header && !footer && paddings[padding], className)}
	{...rest}
>
	{#if header}
		<div class={cn('border-b border-border', paddings[padding])}>
			{@render header()}
		</div>
	{/if}

	{#if children}
		<div class={cn((header || footer) && paddings[padding])}>
			{@render children()}
		</div>
	{/if}

	{#if footer}
		<div class={cn('border-t border-border', paddings[padding])}>
			{@render footer()}
		</div>
	{/if}
</div>
