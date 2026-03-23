<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	type Size = 'sm' | 'md' | 'lg';

	let {
		value = $bindable(''),
		label,
		error,
		hint,
		placeholder,
		size = 'md',
		disabled = false,
		type = 'text',
		class: className,
		icon,
		...rest
	}: {
		value?: string;
		label?: string;
		error?: string;
		hint?: string;
		placeholder?: string;
		size?: Size;
		disabled?: boolean;
		type?: string;
		class?: string;
		icon?: Snippet;
		[key: string]: unknown;
	} = $props();

	const sizes: Record<Size, string> = {
		sm: 'h-8 text-sm px-2.5',
		md: 'h-9 text-sm px-3',
		lg: 'h-10 text-base px-4'
	};

	const inputId = `input-${Math.random().toString(36).slice(2, 9)}`;
</script>

<div class={cn('flex flex-col gap-1.5', className)}>
	{#if label}
		<label for={inputId} class="text-sm font-medium text-foreground">{label}</label>
	{/if}

	<div class="relative">
		{#if icon}
			<div class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">
				{@render icon()}
			</div>
		{/if}

		<input
			id={inputId}
			{type}
			{placeholder}
			{disabled}
			bind:value
			class={cn(
				'w-full rounded-lg border bg-background text-foreground',
				'placeholder:text-muted-foreground',
				'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring',
				'transition-colors',
				'disabled:opacity-50 disabled:pointer-events-none',
				sizes[size],
				icon && 'pl-10',
				error ? 'border-destructive focus-visible:outline-destructive' : 'border-input'
			)}
			{...rest}
		/>
	</div>

	{#if error}
		<p class="text-xs text-destructive">{error}</p>
	{:else if hint}
		<p class="text-xs text-muted-foreground">{hint}</p>
	{/if}
</div>
