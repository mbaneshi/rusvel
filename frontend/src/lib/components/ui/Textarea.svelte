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
</script>

<div class={cn('flex flex-col gap-1.5', className)}>
	{#if label}
		<label for={textareaId} class="text-sm font-medium text-foreground">{label}</label>
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
			'w-full rounded-lg border bg-background text-foreground',
			'placeholder:text-muted-foreground text-sm px-3 py-2',
			'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring',
			'transition-colors resize-none',
			'disabled:opacity-50 disabled:pointer-events-none',
			error ? 'border-destructive' : 'border-input'
		)}
		{...rest}
	></textarea>

	{#if error}
		<p class="text-xs text-destructive">{error}</p>
	{:else if hint}
		<p class="text-xs text-muted-foreground">{hint}</p>
	{/if}
</div>
