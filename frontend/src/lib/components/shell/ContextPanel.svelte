<script lang="ts">
	import { MessageSquare, Braces, Cpu, PanelRightClose } from 'lucide-svelte';
	import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
	import {
		activeSession,
		contextPanelMode,
		contextPanelOpen,
		contextPanelProperties
	} from '$lib/stores';
	import type { ContextPanelMode } from '$lib/stores';

	let {
		deptId,
		deptTitle
	}: {
		deptId: string;
		deptTitle: string;
	} = $props();

	const modes: { id: ContextPanelMode; label: string; icon: typeof MessageSquare }[] = [
		{ id: 'chat', label: 'Chat', icon: MessageSquare },
		{ id: 'properties', label: 'Props', icon: Braces },
		{ id: 'execution', label: 'Output', icon: Cpu }
	];

	function setMode(m: ContextPanelMode) {
		contextPanelMode.set(m);
	}

	function close() {
		contextPanelOpen.set(false);
	}
</script>

<aside
	class="flex h-full w-[320px] shrink-0 flex-col border-l border-border bg-card"
	aria-label="Context panel"
>
	<div class="flex items-center justify-between gap-1 border-b border-border px-2 py-1.5">
		<div class="flex min-w-0 flex-1 gap-0.5">
			{#each modes as m}
				{@const Icon = m.icon}
				<button
					type="button"
					title={m.label}
					onclick={() => setMode(m.id)}
					class="flex shrink-0 items-center gap-1 rounded-md px-2 py-1 text-[10px] font-medium transition-colors
						{$contextPanelMode === m.id
						? 'bg-sidebar-primary/20 text-sidebar-primary'
						: 'text-muted-foreground hover:bg-accent hover:text-foreground'}"
				>
					<Icon size={12} strokeWidth={2} class="shrink-0" />
					<span class="hidden sm:inline">{m.label}</span>
				</button>
			{/each}
		</div>
		<button
			type="button"
			onclick={close}
			class="shrink-0 rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-foreground"
			title="Close panel (⌘J / Ctrl+J)"
		>
			<PanelRightClose size={16} strokeWidth={1.75} />
		</button>
	</div>

	<div class="min-h-0 flex-1 overflow-hidden">
		{#if $contextPanelMode === 'chat'}
			{#if !$activeSession}
				<p class="p-3 text-xs text-muted-foreground">Select a session in the top bar to chat.</p>
			{:else}
				{#key deptId}
					<DepartmentChat dept={deptId} title={deptTitle} compact />
				{/key}
			{/if}
		{:else if $contextPanelMode === 'properties'}
			<div class="flex h-full flex-col">
				<p class="border-b border-border px-3 py-2 text-[10px] text-muted-foreground">
					Set <code class="rounded bg-muted px-1">contextPanelProperties</code> from a page to inspect JSON.
				</p>
				<pre
					class="min-h-0 flex-1 overflow-auto p-3 font-mono text-[10px] leading-relaxed text-foreground"
				>{JSON.stringify($contextPanelProperties, null, 2)}</pre>
			</div>
		{:else}
			<div class="space-y-2 p-3 text-xs text-muted-foreground">
				<p>
					Agent tool calls and job output stream in <strong class="text-foreground">Chat</strong> mode.
				</p>
				<p>
					For full-width chat and history, open
					<a class="text-primary underline" href="/dept/{deptId}/chat">Department chat</a>.
				</p>
			</div>
		{/if}
	</div>
</aside>
