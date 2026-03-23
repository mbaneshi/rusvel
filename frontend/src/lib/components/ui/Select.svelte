<script lang="ts">
	import { cn } from '$lib/utils/cn';

	type Option = { value: string; label: string; disabled?: boolean };

	let {
		value = $bindable(''),
		options = [],
		label,
		error,
		placeholder,
		size = 'md',
		disabled = false,
		class: className,
		...rest
	}: {
		value?: string;
		options?: Option[];
		label?: string;
		error?: string;
		placeholder?: string;
		size?: 'sm' | 'md' | 'lg';
		disabled?: boolean;
		class?: string;
		[key: string]: unknown;
	} = $props();

	const sizes: Record<string, string> = {
		sm: 'text-sm px-2.5 py-1.5',
		md: 'text-sm px-3 py-2',
		lg: 'text-base px-4 py-2.5',
	};

	const base = cn(
		'w-full rounded-[var(--radius-lg)] border bg-[var(--r-bg-raised)] text-[var(--r-fg-default)]',
		'r-focus-ring transition-colors appearance-none cursor-pointer',
		'disabled:opacity-50 disabled:pointer-events-none',
	);
</script>

<div class={cn('flex flex-col gap-1.5', className)}>
	{#if label}
		<label class="text-sm font-medium text-[var(--r-fg-muted)]">{label}</label>
	{/if}

	<div class="relative">
		<select
			bind:value
			{disabled}
			class={cn(
				base,
				sizes[size],
				'pr-8',
				error
					? 'border-danger-500'
					: 'border-[var(--r-border-default)] focus:border-[var(--r-border-brand)]',
			)}
			{...rest}
		>
			{#if placeholder}
				<option value="" disabled>{placeholder}</option>
			{/if}
			{#each options as opt}
				<option value={opt.value} disabled={opt.disabled}>{opt.label}</option>
			{/each}
		</select>

		<div class="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-[var(--r-fg-subtle)]">
			<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
				<path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
			</svg>
		</div>
	</div>

	{#if error}
		<p class="text-xs text-danger-400">{error}</p>
	{/if}
</div>
