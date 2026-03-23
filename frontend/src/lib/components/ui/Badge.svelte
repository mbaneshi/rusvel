<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	type Variant = 'default' | 'brand' | 'success' | 'danger' | 'warning' | 'info';
	type Size = 'sm' | 'md';

	let {
		variant = 'default',
		size = 'md',
		dot = false,
		class: className,
		children,
		...rest
	}: {
		variant?: Variant;
		size?: Size;
		dot?: boolean;
		class?: string;
		children?: Snippet;
		[key: string]: unknown;
	} = $props();

	const base = 'inline-flex items-center font-medium rounded-[var(--radius-full)]';

	const variants: Record<Variant, string> = {
		default: 'bg-[var(--r-bg-raised)] text-[var(--r-fg-muted)]',
		brand:   'bg-brand-900/50 text-brand-300',
		success: 'bg-success-900/50 text-success-400',
		danger:  'bg-danger-900/50 text-danger-400',
		warning: 'bg-warning-900/50 text-warning-400',
		info:    'bg-info-900/50 text-info-400',
	};

	const dotColors: Record<Variant, string> = {
		default: 'bg-neutral-400',
		brand:   'bg-brand-400',
		success: 'bg-success-400',
		danger:  'bg-danger-400',
		warning: 'bg-warning-400',
		info:    'bg-info-400',
	};

	const sizes: Record<Size, string> = {
		sm: 'text-xs px-2 py-0.5 gap-1',
		md: 'text-xs px-2.5 py-1 gap-1.5',
	};
</script>

<span class={cn(base, variants[variant], sizes[size], className)} {...rest}>
	{#if dot}
		<span class={cn('h-1.5 w-1.5 rounded-full', dotColors[variant])}></span>
	{/if}
	{#if children}
		{@render children()}
	{/if}
</span>
