<script lang="ts">
	import { cn } from '$lib/utils/cn';

	let {
		class: className,
		...rest
	}: {
		class?: string;
		[key: string]: unknown;
	} = $props();
</script>

<div class={cn('animate-pulse rounded-md bg-muted', className)} {...rest}></div>
