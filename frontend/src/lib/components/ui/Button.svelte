<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Variant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'outline' | 'link';
	type Size = 'xs' | 'sm' | 'md' | 'lg' | 'icon';

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
	}: HTMLButtonAttributes & {
		variant?: Variant;
		size?: Size;
		disabled?: boolean;
		loading?: boolean;
		class?: string;
		children?: Snippet;
		icon?: Snippet;
		onclick?: (e: MouseEvent) => void;
	} = $props();

	const variants: Record<Variant, string> = {
		primary: 'bg-primary text-primary-foreground shadow-sm hover:bg-primary/90',
		secondary:
			'bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80 border border-border',
		ghost: 'text-muted-foreground hover:text-foreground hover:bg-accent',
		danger: 'bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90',
		outline:
			'border border-border bg-background text-foreground shadow-sm hover:bg-accent hover:text-accent-foreground',
		link: 'text-primary underline-offset-4 hover:underline'
	};

	const sizes: Record<Size, string> = {
		xs: 'h-7 text-xs px-2 gap-1 rounded-md',
		sm: 'h-8 text-sm px-3 gap-1.5 rounded-md',
		md: 'h-9 text-sm px-4 gap-2 rounded-lg',
		lg: 'h-10 text-base px-5 gap-2 rounded-lg',
		icon: 'h-9 w-9 rounded-lg'
	};
</script>

<button
	{type}
	disabled={disabled || loading}
	class={cn(
		'inline-flex items-center justify-center font-medium transition-colors cursor-pointer',
		'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring',
		'disabled:opacity-50 disabled:pointer-events-none',
		variants[variant],
		sizes[size],
		className
	)}
	{onclick}
	{...rest}
>
	{#if loading}
		<span
			class="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
		></span>
	{:else if icon}
		{@render icon()}
	{/if}
	{#if children}
		{@render children()}
	{/if}
</button>
