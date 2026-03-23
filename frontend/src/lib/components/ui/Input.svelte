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
		sm: 'text-sm px-2.5 py-1.5',
		md: 'text-sm px-3 py-2',
		lg: 'text-base px-4 py-2.5',
	};

	const inputId = `input-${Math.random().toString(36).slice(2, 9)}`;

	const inputBase = cn(
		'w-full rounded-[var(--radius-lg)] border bg-[var(--r-bg-raised)] text-[var(--r-fg-default)]',
		'placeholder:text-[var(--r-fg-subtle)]',
		'r-focus-ring transition-colors',
		'disabled:opacity-50 disabled:pointer-events-none',
	);
</script>

<div class={cn('flex flex-col gap-1.5', className)}>
	{#if label}
		<label for={inputId} class="text-sm font-medium text-[var(--r-fg-muted)]">{label}</label>
	{/if}

	<div class="relative">
		{#if icon}
			<div class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--r-fg-subtle)]">
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
				inputBase,
				sizes[size],
				icon && 'pl-10',
				error
					? 'border-danger-500 focus:border-danger-500'
					: 'border-[var(--r-border-default)] focus:border-[var(--r-border-brand)]',
			)}
			{...rest}
		/>
	</div>

	{#if error}
		<p class="text-xs text-danger-400">{error}</p>
	{:else if hint}
		<p class="text-xs text-[var(--r-fg-subtle)]">{hint}</p>
	{/if}
</div>
