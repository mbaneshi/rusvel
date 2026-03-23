<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import { icons, type IconName } from './icons';

	let {
		name,
		size = 20,
		strokeWidth = 1.5,
		class: className,
		...rest
	}: {
		name: IconName;
		size?: number;
		strokeWidth?: number;
		class?: string;
		[key: string]: unknown;
	} = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width={strokeWidth}
	stroke-linecap="round"
	stroke-linejoin="round"
	class={cn('shrink-0', className)}
	aria-hidden="true"
	{...rest}
>
	<path d={icons[name]} />
</svg>
