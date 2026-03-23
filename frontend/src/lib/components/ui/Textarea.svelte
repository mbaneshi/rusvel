<script lang="ts">
	import { cn } from '$lib/utils/cn';

	let {
		value = $bindable(''),
		label,
		error,
		hint,
		placeholder,
		rows = 3,
		autogrow = false,
		disabled = false,
		class: className,
		...rest
	}: {
		value?: string;
		label?: string;
		error?: string;
		hint?: string;
		placeholder?: string;
		rows?: number;
		autogrow?: boolean;
		disabled?: boolean;
		class?: string;
		[key: string]: unknown;
	} = $props();

	let textareaEl: HTMLTextAreaElement | undefined = $state();

	function handleInput() {
		if (autogrow && textareaEl) {
			textareaEl.style.height = 'auto';
			textareaEl.style.height = textareaEl.scrollHeight + 'px';
		}
	}

	const textareaId = `textarea-${Math.random().toString(36).slice(2, 9)}`;

	const base = cn(
		'w-full rounded-[var(--radius-lg)] border bg-[var(--r-bg-raised)] text-[var(--r-fg-default)]',
		'placeholder:text-[var(--r-fg-subtle)] text-sm px-3 py-2',
		'r-focus-ring transition-colors resize-none',
		'disabled:opacity-50 disabled:pointer-events-none',
	);
</script>

<div class={cn('flex flex-col gap-1.5', className)}>
	{#if label}
		<label for={textareaId} class="text-sm font-medium text-[var(--r-fg-muted)]">{label}</label>
	{/if}

	<textarea
		id={textareaId}
		bind:this={textareaEl}
		bind:value
		{placeholder}
		{rows}
		{disabled}
		oninput={handleInput}
		class={cn(
			base,
			error
				? 'border-danger-500 focus:border-danger-500'
				: 'border-[var(--r-border-default)] focus:border-[var(--r-border-brand)]',
		)}
		{...rest}
	></textarea>

	{#if error}
		<p class="text-xs text-danger-400">{error}</p>
	{:else if hint}
		<p class="text-xs text-[var(--r-fg-subtle)]">{hint}</p>
	{/if}
</div>
