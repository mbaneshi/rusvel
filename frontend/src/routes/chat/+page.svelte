<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { streamChat, getConversations, getChatHistory } from '$lib/api';
	import type { ChatMessage, Conversation } from '$lib/api';

	interface DisplayMessage {
		role: 'user' | 'assistant' | 'system';
		content: string;
		streaming?: boolean;
	}

	let messages: DisplayMessage[] = $state([]);
	let conversations: Conversation[] = $state([]);
	let conversationId: string | undefined = $state(undefined);
	let inputText = $state('');
	let sending = $state(false);
	let error = $state('');
	let messagesContainer: HTMLDivElement | undefined = $state(undefined);

	onMount(async () => {
		try {
			conversations = await getConversations();
		} catch {
			// No conversations yet — that's fine
		}
	});

	async function scrollToBottom() {
		await tick();
		if (messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight;
		}
	}

	async function loadConversation(id: string) {
		try {
			const history = await getChatHistory(id);
			messages = history.map((m) => ({ role: m.role, content: m.content }));
			conversationId = id;
			await scrollToBottom();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load conversation';
		}
	}

	function newConversation() {
		messages = [];
		conversationId = undefined;
		inputText = '';
		error = '';
	}

	async function sendMessage() {
		const text = inputText.trim();
		if (!text || sending) return;

		inputText = '';
		sending = true;
		error = '';

		// Add user message
		messages = [...messages, { role: 'user', content: text }];
		await scrollToBottom();

		// Add placeholder for assistant response
		messages = [...messages, { role: 'assistant', content: '', streaming: true }];
		await scrollToBottom();

		try {
			await streamChat(
				text,
				conversationId,
				// onDelta
				(deltaText, convId) => {
					conversationId = convId;
					const last = messages[messages.length - 1];
					if (last?.role === 'assistant') {
						messages = [
							...messages.slice(0, -1),
							{ ...last, content: last.content + deltaText }
						];
					}
					scrollToBottom();
				},
				// onDone
				(fullText, convId) => {
					conversationId = convId;
					const last = messages[messages.length - 1];
					if (last?.role === 'assistant') {
						messages = [
							...messages.slice(0, -1),
							{ role: 'assistant', content: fullText, streaming: false }
						];
					}
					sending = false;
					// Refresh conversation list
					getConversations().then((c) => (conversations = c)).catch(() => {});
				},
				// onError
				(msg) => {
					error = msg;
					// Remove the empty assistant placeholder
					if (messages[messages.length - 1]?.content === '') {
						messages = messages.slice(0, -1);
					}
					sending = false;
				}
			);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to send message';
			if (messages[messages.length - 1]?.content === '') {
				messages = messages.slice(0, -1);
			}
			sending = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			sendMessage();
		}
	}
</script>

<div class="flex h-full">
	<!-- Conversation sidebar -->
	<div class="flex w-56 flex-shrink-0 flex-col border-r border-gray-800 bg-gray-900/50">
		<div class="border-b border-gray-800 p-3">
			<button
				onclick={newConversation}
				class="w-full rounded-lg bg-indigo-600 px-3 py-2 text-sm font-medium hover:bg-indigo-500"
			>
				+ New Chat
			</button>
		</div>
		<div class="flex-1 overflow-y-auto p-2">
			{#if conversations.length === 0}
				<p class="p-2 text-xs text-gray-600">No conversations yet</p>
			{:else}
				{#each conversations as conv}
					<button
						onclick={() => loadConversation(conv.id)}
						class="mb-1 w-full rounded-lg px-3 py-2 text-left text-sm transition-colors
							{conversationId === conv.id
							? 'bg-gray-800 text-gray-100'
							: 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
					>
						<p class="truncate">{conv.title}</p>
						<p class="text-xs text-gray-600">{conv.message_count} messages</p>
					</button>
				{/each}
			{/if}
		</div>
	</div>

	<!-- Chat area -->
	<div class="flex flex-1 flex-col">
		<!-- Messages -->
		<div bind:this={messagesContainer} class="flex-1 overflow-y-auto p-6">
			{#if messages.length === 0}
				<div class="flex h-full items-center justify-center">
					<div class="text-center">
						<div
							class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-indigo-600/20"
						>
							<span class="text-2xl font-bold text-indigo-400">R</span>
						</div>
						<h2 class="text-lg font-semibold text-gray-200">RUSVEL Assistant</h2>
						<p class="mt-2 max-w-md text-sm text-gray-500">
							Your AI companion. Ask about your products, plan your day, draft content, or
							strategize.
						</p>
					</div>
				</div>
			{:else}
				<div class="mx-auto max-w-3xl space-y-4">
					{#each messages as msg}
						<div
							class="flex gap-3 {msg.role === 'user' ? 'justify-end' : 'justify-start'}"
						>
							{#if msg.role === 'assistant'}
								<div
									class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-lg bg-indigo-600/30 text-xs font-bold text-indigo-300"
								>
									R
								</div>
							{/if}
							<div
								class="max-w-[80%] rounded-xl px-4 py-3 text-sm {msg.role === 'user'
									? 'bg-indigo-600 text-white'
									: 'bg-gray-800 text-gray-200'}"
							>
								<p class="whitespace-pre-wrap">{msg.content}{#if msg.streaming}<span
											class="inline-block h-4 w-1 animate-pulse bg-indigo-400"
										></span>{/if}</p>
							</div>
							{#if msg.role === 'user'}
								<div
									class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-lg bg-gray-700 text-xs font-bold text-gray-300"
								>
									M
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Error -->
		{#if error}
			<div class="mx-6 mb-2 rounded-lg border border-red-900 bg-red-950 px-4 py-2 text-sm text-red-400">
				{error}
			</div>
		{/if}

		<!-- Input -->
		<div class="border-t border-gray-800 p-4">
			<div class="mx-auto flex max-w-3xl gap-3">
				<textarea
					bind:value={inputText}
					onkeydown={handleKeydown}
					placeholder="Message RUSVEL..."
					rows="1"
					disabled={sending}
					class="flex-1 resize-none rounded-xl border border-gray-700 bg-gray-800 px-4 py-3 text-sm text-gray-200 placeholder-gray-500 focus:border-indigo-500 focus:outline-none disabled:opacity-50"
				></textarea>
				<button
					onclick={sendMessage}
					disabled={sending || !inputText.trim()}
					class="rounded-xl bg-indigo-600 px-5 py-3 text-sm font-medium hover:bg-indigo-500 disabled:opacity-50"
				>
					{sending ? '...' : 'Send'}
				</button>
			</div>
		</div>
	</div>
</div>
