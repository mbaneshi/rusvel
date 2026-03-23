<script lang="ts">
	import { Handle, Position } from '@xyflow/svelte';

	let { data }: { data: { label: string; prompt: string; index: number } } = $props();
</script>

<div class="rounded-lg border border-border bg-card px-3 py-2 shadow-sm min-w-[140px]">
	<Handle type="target" position={Position.Top} class="!bg-primary !w-2 !h-2" />
	<div class="flex items-center gap-2">
		<span
			class="flex h-5 w-5 items-center justify-center rounded-full bg-primary/20 text-[9px] font-bold text-primary"
			>{data.index + 1}</span
		>
		<div>
			<p class="text-[10px] font-medium text-foreground">{data.label}</p>
			<p class="text-[8px] text-muted-foreground truncate max-w-[120px]">{data.prompt}</p>
		</div>
	</div>
	<Handle type="source" position={Position.Bottom} class="!bg-primary !w-2 !h-2" />
</div>
