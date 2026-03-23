<script lang="ts">
	import { cn } from '$lib/utils/cn';

	type Tab = { id: string; label: string; disabled?: boolean };

	let {
		tabs = [],
		active = $bindable(''),
		variant = 'underline',
		class: className,
		onchange
	}: {
		tabs: Tab[];
		active?: string;
		variant?: 'underline' | 'pills';
		class?: string;
		onchange?: (id: string) => void;
	} = $props();

	function select(id: string) {
		active = id;
		onchange?.(id);
	}
</script>

<div
	class={cn('flex gap-1', variant === 'underline' && 'border-b border-border', className)}
	role="tablist"
>
	{#each tabs as tab}
		<button
			role="tab"
			aria-selected={active === tab.id}
			disabled={tab.disabled}
			onclick={() => select(tab.id)}
			class={cn(
				'text-sm font-medium transition-colors cursor-pointer',
				'disabled:opacity-50 disabled:pointer-events-none',
				variant === 'underline' && [
					'px-3 py-2 -mb-px border-b-2',
					active === tab.id
						? 'border-primary text-foreground'
						: 'border-transparent text-muted-foreground hover:text-foreground'
				],
				variant === 'pills' && [
					'px-3 py-1.5 rounded-lg',
					active === tab.id
						? 'bg-primary text-primary-foreground'
						: 'text-muted-foreground hover:text-foreground hover:bg-accent'
				]
			)}
		>
			{tab.label}
		</button>
	{/each}
</div>
