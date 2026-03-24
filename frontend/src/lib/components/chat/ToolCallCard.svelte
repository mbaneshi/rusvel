<script lang="ts">
	let {
		name,
		args,
		result = null,
		isError = false,
		expanded = false
	}: {
		name: string;
		args: Record<string, unknown>;
		result: string | null;
		isError: boolean;
		expanded?: boolean;
	} = $props();

	let open = $state(false);
	$effect(() => { open = expanded; });

	let borderColor = $derived(
		result === null ? 'border-blue-500' : isError ? 'border-red-500' : 'border-green-500'
	);
</script>

<div class="{borderColor} border-l-2 bg-zinc-900/50 rounded-r-md px-3 py-2 my-2 font-mono text-sm">
	<button onclick={() => (open = !open)} class="flex items-center gap-2 w-full text-left">
		{#if result === null}
			<span class="animate-spin text-blue-400 text-xs">&#x27F3;</span>
		{:else if isError}
			<span class="text-red-400 text-xs">&#x2717;</span>
		{:else}
			<span class="text-green-400 text-xs">&#x2713;</span>
		{/if}
		<span class="text-zinc-300 truncate">{name}</span>
		<span class="text-zinc-600 text-xs ml-auto flex-shrink-0">{open ? '\u25BC' : '\u25B6'}</span>
	</button>
	{#if open}
		<div class="mt-2 space-y-2 text-xs">
			<div class="text-zinc-500">Input:</div>
			<pre class="bg-zinc-950 p-2 rounded overflow-x-auto max-h-40 text-zinc-400">{JSON.stringify(args, null, 2)}</pre>
			{#if result !== null}
				<div class="text-zinc-500">Result:</div>
				<pre class="bg-zinc-950 p-2 rounded overflow-x-auto max-h-60 {isError ? 'text-red-400' : 'text-zinc-300'}">{result}</pre>
			{/if}
		</div>
	{/if}
</div>
