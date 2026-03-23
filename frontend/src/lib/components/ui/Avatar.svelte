<script lang="ts">
	import { cn } from '$lib/utils/cn';

	let {
		initials,
		src,
		alt = '',
		size = 'md',
		variant = 'brand',
		class: className,
		...rest
	}: {
		initials?: string;
		src?: string;
		alt?: string;
		size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
		variant?: 'brand' | 'neutral' | 'success' | 'danger';
		class?: string;
		[key: string]: unknown;
	} = $props();

	const sizes: Record<string, { container: string; text: string }> = {
		xs: { container: 'h-6 w-6', text: 'text-xs' },
		sm: { container: 'h-8 w-8', text: 'text-xs' },
		md: { container: 'h-10 w-10', text: 'text-sm' },
		lg: { container: 'h-12 w-12', text: 'text-base' },
		xl: { container: 'h-16 w-16', text: 'text-lg' }
	};

	const variants: Record<string, string> = {
		brand: 'bg-primary/15 text-primary',
		neutral: 'bg-secondary text-muted-foreground',
		success: 'bg-success-500/15 text-success-400',
		danger: 'bg-destructive/15 text-destructive'
	};
</script>

<div
	class={cn(
		'inline-flex items-center justify-center rounded-full font-semibold shrink-0 overflow-hidden',
		sizes[size].container,
		!src && variants[variant],
		className
	)}
	{...rest}
>
	{#if src}
		<img {src} {alt} class="h-full w-full object-cover" />
	{:else if initials}
		<span class={sizes[size].text}>{initials}</span>
	{/if}
</div>
