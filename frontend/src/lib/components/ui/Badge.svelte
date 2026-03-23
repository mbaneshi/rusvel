<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	type Variant = 'default' | 'brand' | 'success' | 'danger' | 'warning' | 'info' | 'outline';

	let {
		variant = 'default',
		size = 'md',
		dot = false,
		class: className,
		children,
		...rest
	}: {
		variant?: Variant;
		size?: 'sm' | 'md';
		dot?: boolean;
		class?: string;
		children?: Snippet;
		[key: string]: unknown;
	} = $props();

	const variants: Record<Variant, string> = {
		default: 'bg-secondary text-secondary-foreground',
		brand: 'bg-primary/15 text-primary',
		success: 'bg-success-500/15 text-success-400',
		danger: 'bg-destructive/15 text-destructive',
		warning: 'bg-warning-500/15 text-warning-400',
		info: 'bg-info-500/15 text-info-400',
		outline: 'border border-border text-foreground'
	};

	const dotColors: Record<Variant, string> = {
		default: 'bg-muted-foreground',
		brand: 'bg-primary',
		success: 'bg-success-400',
		danger: 'bg-destructive',
		warning: 'bg-warning-400',
		info: 'bg-info-400',
		outline: 'bg-foreground'
	};

	const sizes: Record<string, string> = {
		sm: 'text-xs px-2 py-0.5 gap-1',
		md: 'text-xs px-2.5 py-1 gap-1.5'
	};
</script>

<span
	class={cn(
		'inline-flex items-center font-medium rounded-full',
		variants[variant],
		sizes[size],
		className
	)}
	{...rest}
>
	{#if dot}
		<span class={cn('h-1.5 w-1.5 rounded-full', dotColors[variant])}></span>
	{/if}
	{#if children}
		{@render children()}
	{/if}
</span>
