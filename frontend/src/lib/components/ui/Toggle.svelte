<script lang="ts">
	import { cn } from '$lib/utils/cn';

	let {
		checked = $bindable(false),
		label,
		size = 'md',
		disabled = false,
		class: className,
		onchange,
		...rest
	}: {
		checked?: boolean;
		label?: string;
		size?: 'sm' | 'md';
		disabled?: boolean;
		class?: string;
		onchange?: (checked: boolean) => void;
		[key: string]: unknown;
	} = $props();

	const sizes = {
		sm: { track: 'h-5 w-9', thumb: 'h-3.5 w-3.5', translate: 'translate-x-4' },
		md: { track: 'h-6 w-11', thumb: 'h-4.5 w-4.5', translate: 'translate-x-5' }
	};

	function toggle() {
		if (disabled) return;
		checked = !checked;
		onchange?.(checked);
	}
</script>

<button
	role="switch"
	aria-checked={checked}
	type="button"
	{disabled}
	onclick={toggle}
	class={cn(
		'inline-flex items-center gap-2 cursor-pointer',
		disabled && 'opacity-50 pointer-events-none',
		className
	)}
	{...rest}
>
	<span
		class={cn(
			'relative inline-flex shrink-0 rounded-full transition-colors',
			sizes[size].track,
			checked ? 'bg-primary' : 'bg-secondary'
		)}
	>
		<span
			class={cn(
				'inline-block rounded-full bg-white shadow transition-transform',
				sizes[size].thumb,
				'absolute top-0.5 left-0.5',
				checked && sizes[size].translate
			)}
		></span>
	</span>
	{#if label}
		<span class="text-sm text-foreground">{label}</span>
	{/if}
</button>
