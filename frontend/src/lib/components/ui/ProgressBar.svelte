<script lang="ts">
	import { cn } from '$lib/utils/cn';

	let {
		value = 0,
		max = 100,
		size = 'md',
		variant = 'brand',
		label,
		showValue = false,
		class: className
	}: {
		value?: number;
		max?: number;
		size?: 'sm' | 'md' | 'lg';
		variant?: 'brand' | 'success' | 'danger' | 'warning';
		label?: string;
		showValue?: boolean;
		class?: string;
	} = $props();

	let pct = $derived(Math.min(100, Math.max(0, (value / max) * 100)));

	const sizes: Record<string, string> = {
		sm: 'h-1',
		md: 'h-1.5',
		lg: 'h-2.5'
	};

	const fills: Record<string, string> = {
		brand: 'bg-primary',
		success: 'bg-success-500',
		danger: 'bg-destructive',
		warning: 'bg-warning-500'
	};
</script>

<div class={cn('flex flex-col gap-1', className)}>
	{#if label || showValue}
		<div class="flex justify-between text-xs">
			{#if label}
				<span class="text-muted-foreground">{label}</span>
			{/if}
			{#if showValue}
				<span class="text-muted-foreground">{Math.round(pct)}%</span>
			{/if}
		</div>
	{/if}
	<div class={cn('w-full rounded-full bg-secondary', sizes[size])}>
		<div
			class={cn('rounded-full transition-all duration-300', sizes[size], fills[variant])}
			style="width: {pct}%"
			role="progressbar"
			aria-valuenow={value}
			aria-valuemin={0}
			aria-valuemax={max}
		></div>
	</div>
</div>
