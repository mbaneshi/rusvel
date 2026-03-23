<script lang="ts">
	import { cn } from '$lib/utils/cn';

	let {
		orientation = 'horizontal',
		class: className,
		...rest
	}: {
		orientation?: 'horizontal' | 'vertical';
		class?: string;
		[key: string]: unknown;
	} = $props();
</script>

<div
	role="separator"
	aria-orientation={orientation}
	class={cn(
		'shrink-0 bg-border',
		orientation === 'horizontal' ? 'h-px w-full' : 'h-full w-px',
		className
	)}
	{...rest}
></div>
