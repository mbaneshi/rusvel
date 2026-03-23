<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	type Variant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'outline';
	type Size = 'xs' | 'sm' | 'md' | 'lg';

	let {
		variant = 'primary',
		size = 'md',
		disabled = false,
		loading = false,
		type = 'button',
		class: className,
		children,
		icon,
		onclick,
		...rest
	}: {
		variant?: Variant;
		size?: Size;
		disabled?: boolean;
		loading?: boolean;
		type?: 'button' | 'submit' | 'reset';
		class?: string;
		children?: Snippet;
		icon?: Snippet;
		onclick?: (e: MouseEvent) => void;
		[key: string]: unknown;
	} = $props();

	const base = 'inline-flex items-center justify-center font-medium transition-colors r-focus-ring cursor-pointer disabled:opacity-50 disabled:pointer-events-none';

	const variants: Record<Variant, string> = {
		primary:   'bg-[var(--r-brand-default)] text-[var(--r-fg-on-brand)] hover:bg-[var(--r-brand-hover)]',
		secondary: 'bg-[var(--r-bg-raised)] text-[var(--r-fg-default)] hover:bg-[var(--r-border-strong)] border border-[var(--r-border-default)]',
		ghost:     'text-[var(--r-fg-muted)] hover:text-[var(--r-fg-default)] hover:bg-[var(--r-bg-raised)]',
		danger:    'bg-danger-600 text-white hover:bg-danger-500',
		outline:   'border border-[var(--r-border-strong)] text-[var(--r-fg-default)] hover:bg-[var(--r-bg-raised)]',
	};

	const sizes: Record<Size, string> = {
		xs: 'text-xs px-2 py-1 gap-1 rounded-[var(--radius-sm)]',
		sm: 'text-sm px-3 py-1.5 gap-1.5 rounded-[var(--radius-md)]',
		md: 'text-sm px-4 py-2 gap-2 rounded-[var(--radius-lg)]',
		lg: 'text-base px-5 py-2.5 gap-2 rounded-[var(--radius-lg)]',
	};
</script>

<button
	{type}
	disabled={disabled || loading}
	class={cn(base, variants[variant], sizes[size], className)}
	{onclick}
	{...rest}
>
	{#if loading}
		<span class="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></span>
	{:else if icon}
		{@render icon()}
	{/if}
	{#if children}
		{@render children()}
	{/if}
</button>
