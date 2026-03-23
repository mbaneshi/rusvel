<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	type Size = 'xs' | 'sm' | 'base' | 'lg';
	type Color = 'default' | 'muted' | 'subtle' | 'brand' | 'success' | 'danger' | 'warning';

	let {
		as = 'p',
		size = 'base',
		color = 'default',
		weight = 'normal',
		class: className,
		children,
		...rest
	}: {
		as?: string;
		size?: Size;
		color?: Color;
		weight?: 'normal' | 'medium' | 'semibold' | 'bold';
		class?: string;
		children: Snippet;
		[key: string]: unknown;
	} = $props();

	const sizeMap: Record<Size, string> = {
		xs: 'text-xs',
		sm: 'text-sm',
		base: 'text-base',
		lg: 'text-lg'
	};

	const colorMap: Record<Color, string> = {
		default: 'text-[var(--foreground)]',
		muted: 'text-[var(--muted-foreground)]',
		subtle: 'text-[var(--muted-foreground)]',
		brand: 'text-brand-400',
		success: 'text-success-400',
		danger: 'text-danger-400',
		warning: 'text-warning-400'
	};
</script>

<svelte:element
	this={as}
	class={cn(sizeMap[size], colorMap[color], `font-${weight}`, className)}
	{...rest}
>
	{@render children()}
</svelte:element>
