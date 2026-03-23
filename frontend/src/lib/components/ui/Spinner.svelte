<script lang="ts">
	import { cn } from '$lib/utils/cn';

	let {
		size = 'md',
		class: className,
		...rest
	}: {
		size?: 'sm' | 'md' | 'lg';
		class?: string;
		[key: string]: unknown;
	} = $props();

	const sizes: Record<string, string> = {
		sm: 'h-4 w-4 border-2',
		md: 'h-6 w-6 border-2',
		lg: 'h-8 w-8 border-3'
	};
</script>

<span
	role="status"
	aria-label="Loading"
	class={cn(
		'inline-block animate-spin rounded-full border-current border-t-transparent',
		sizes[size],
		'text-primary',
		className
	)}
	{...rest}
></span>
