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
		xs: { container: 'h-6 w-6',   text: 'text-xs' },
		sm: { container: 'h-8 w-8',   text: 'text-xs' },
		md: { container: 'h-10 w-10', text: 'text-sm' },
		lg: { container: 'h-12 w-12', text: 'text-base' },
		xl: { container: 'h-16 w-16', text: 'text-lg' },
	};

	const variants: Record<string, string> = {
		brand:   'bg-brand-900/50 text-brand-300',
		neutral: 'bg-[var(--r-bg-raised)] text-[var(--r-fg-muted)]',
		success: 'bg-success-900/50 text-success-300',
		danger:  'bg-danger-900/50 text-danger-300',
	};
</script>

<div
	class={cn(
		'inline-flex items-center justify-center rounded-full font-semibold shrink-0 overflow-hidden',
		sizes[size].container,
		!src && variants[variant],
		className,
	)}
	{...rest}
>
	{#if src}
		<img {src} {alt} class="h-full w-full object-cover" />
	{:else if initials}
		<span class={sizes[size].text}>{initials}</span>
	{/if}
</div>
