<script lang="ts">
	import { cn } from '$lib/utils/cn';
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		title,
		description,
		class: className,
		children,
		footer
	}: {
		open?: boolean;
		title?: string;
		description?: string;
		class?: string;
		children?: Snippet;
		footer?: Snippet;
	} = $props();

	function close() {
		open = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}

	function handleBackdrop(e: MouseEvent) {
		if (e.target === e.currentTarget) close();
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center"
		onkeydown={handleKeydown}
		role="presentation"
	>
		<!-- Backdrop -->
		<div
			class="fixed inset-0 bg-black/60 backdrop-blur-sm"
			onclick={handleBackdrop}
			role="presentation"
		></div>

		<!-- Dialog -->
		<div
			class={cn(
				'relative z-50 w-full max-w-lg rounded-xl border border-border bg-card p-6 shadow-xl',
				'animate-in fade-in-0 zoom-in-95',
				className
			)}
			role="dialog"
			aria-modal="true"
			aria-label={title}
		>
			<!-- Close button -->
			<button
				onclick={close}
				class="absolute right-4 top-4 rounded-sm text-muted-foreground hover:text-foreground transition-colors"
				aria-label="Close"
			>
				<svg
					class="h-4 w-4"
					viewBox="0 0 16 16"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path d="M4 4l8 8M12 4l-8 8" stroke-linecap="round" />
				</svg>
			</button>

			{#if title}
				<h2 class="text-lg font-semibold text-foreground">{title}</h2>
			{/if}
			{#if description}
				<p class="mt-1 text-sm text-muted-foreground">{description}</p>
			{/if}

			{#if children}
				<div class="mt-4">
					{@render children()}
				</div>
			{/if}

			{#if footer}
				<div class="mt-6 flex justify-end gap-2">
					{@render footer()}
				</div>
			{/if}
		</div>
	</div>
{/if}
