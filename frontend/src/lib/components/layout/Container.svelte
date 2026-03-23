<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	let {
		size = 'lg',
		class: className,
		children,
		...rest
	}: {
		size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
		class?: string;
		children: Snippet;
		[key: string]: unknown;
	} = $props();

	const sizes: Record<string, string> = {
		sm: 'max-w-2xl',
		md: 'max-w-4xl',
		lg: 'max-w-6xl',
		xl: 'max-w-7xl',
		full: 'max-w-full'
	};
</script>

<div class={cn('mx-auto w-full px-4 sm:px-6', sizes[size], className)} {...rest}>
	{@render children()}
</div>
