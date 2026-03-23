<script lang="ts">
	import { cn } from '$lib/utils/cn';

	let {
		orientation = 'horizontal',
		class: className,
		label,
	}: {
		orientation?: 'horizontal' | 'vertical';
		class?: string;
		label?: string;
	} = $props();
</script>

{#if label}
	<div class={cn('flex items-center gap-3', className)}>
		<div class="flex-1 h-px bg-[var(--r-border-default)]"></div>
		<span class="text-xs text-[var(--r-fg-subtle)] shrink-0">{label}</span>
		<div class="flex-1 h-px bg-[var(--r-border-default)]"></div>
	</div>
{:else if orientation === 'horizontal'}
	<hr class={cn('border-0 h-px bg-[var(--r-border-default)]', className)} />
{:else}
	<div class={cn('w-px self-stretch bg-[var(--r-border-default)]', className)}></div>
{/if}
