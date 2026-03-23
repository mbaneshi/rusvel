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
		sm: 'h-8 text-sm px-2.5',
		md: 'h-9 text-sm px-3',
		lg: 'h-10 text-base px-4'
	};

	const selectId = `select-${Math.random().toString(36).slice(2, 9)}`;
</script>

<div class={cn('flex flex-col gap-1.5', className)}>
	{#if label}
		<label for={selectId} class="text-sm font-medium text-foreground">{label}</label>
	{/if}

	<div class="relative">
		<select
			id={selectId}
			bind:value
			{disabled}
			class={cn(
				'w-full rounded-lg border bg-background text-foreground',
				'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring',
				'transition-colors appearance-none cursor-pointer',
				'disabled:opacity-50 disabled:pointer-events-none',
				sizes[size],
				'pr-8',
				error ? 'border-destructive' : 'border-input'
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

		<div
			class="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground"
		>
			<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
				<path
					d="M4 6l4 4 4-4"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			</svg>
		</div>
	</div>

	{#if error}
		<p class="text-xs text-destructive">{error}</p>
	{/if}
</div>
