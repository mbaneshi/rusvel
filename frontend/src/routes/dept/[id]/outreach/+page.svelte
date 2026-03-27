<script lang="ts">
	import { page } from '$app/state';
	import { activeSession } from '$lib/stores';
	import {
		getGtmContacts,
		getGtmOutreachSequences,
		postGtmOutreachExecute,
		postGtmOutreachSequence,
		postGtmOutreachSequenceActivate,
		type GtmContactRow,
		type GtmOutreachSequenceRow,
		type GtmSequenceStep
	} from '$lib/api';
	import { toast } from 'svelte-sonner';
	import { ArrowRight, Kanban, Receipt, Users } from 'lucide-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	let sequences: GtmOutreachSequenceRow[] = $state([]);
	let contacts: GtmContactRow[] = $state([]);
	let loading = $state(true);
	let submitting = $state(false);
	let newName = $state('');
	let stepsJson = $state(
		JSON.stringify(
			[
				{
					delay_days: 0,
					channel: 'email',
					template: 'Hi — quick note about our last conversation.'
				},
				{
					delay_days: 3,
					channel: 'email',
					template: 'Following up on my previous message.'
				}
			] satisfies GtmSequenceStep[],
			null,
			2
		)
	);
	let runContactId = $state('');

	let deptId = $derived(page.params.id);
	let isGtm = $derived(deptId === 'gtm');

	async function load() {
		if (!sessionId || !isGtm) return;
		loading = true;
		try {
			const [s, c] = await Promise.all([
				getGtmOutreachSequences(sessionId),
				getGtmContacts(sessionId)
			]);
			sequences = s;
			contacts = c;
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load outreach');
			sequences = [];
			contacts = [];
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (!sessionId || !isGtm) return;
		void load();
	});

	async function onCreate() {
		if (!sessionId) return;
		let steps: GtmSequenceStep[];
		try {
			const parsed = JSON.parse(stepsJson) as unknown;
			if (!Array.isArray(parsed) || parsed.length === 0) {
				throw new Error('steps must be a non-empty array');
			}
			steps = parsed as GtmSequenceStep[];
		} catch {
			toast.error('Invalid JSON for steps');
			return;
		}
		const trimmed = newName.trim();
		if (!trimmed) {
			toast.error('Name is required');
			return;
		}
		submitting = true;
		try {
			await postGtmOutreachSequence(sessionId, { name: trimmed, steps });
			newName = '';
			toast.success('Sequence created (draft)');
			await load();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Create failed');
		} finally {
			submitting = false;
		}
	}

	async function onActivate(id: string) {
		if (!sessionId) return;
		try {
			await postGtmOutreachSequenceActivate(sessionId, id);
			toast.success('Sequence activated');
			await load();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Activate failed');
		}
	}

	async function onRun(seqId: string) {
		if (!sessionId) {
			toast.error('No session');
			return;
		}
		if (!runContactId.trim()) {
			toast.error('Pick a contact to run the sequence');
			return;
		}
		try {
			const r = await postGtmOutreachExecute(sessionId, seqId, runContactId.trim());
			toast.success(`Queued job ${r.job_id}`);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Execute failed');
		}
	}
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
	<div class="flex flex-wrap items-start justify-between gap-2 border-b border-border px-4 py-3">
		<div>
			<h1 class="text-lg font-semibold">Outreach sequences</h1>
			<p class="text-xs text-muted-foreground">
				Create multi-step email sequences, activate them, then enqueue sends for a contact. Each step
				uses approval + your configured email adapter before delivery.
			</p>
		</div>
		{#if isGtm}
			<div class="flex flex-wrap items-center gap-2">
				<a
					href="/dept/gtm/contacts"
					class="inline-flex items-center gap-1.5 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					<Users class="h-3.5 w-3.5 opacity-80" strokeWidth={2} />
					Contacts
				</a>
				<a
					href="/dept/gtm/deals"
					class="inline-flex items-center gap-1.5 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					<Kanban class="h-3.5 w-3.5 opacity-80" strokeWidth={2} />
					Deals
				</a>
				<a
					href="/dept/gtm/invoices"
					class="inline-flex items-center gap-1.5 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					<Receipt class="h-3.5 w-3.5 opacity-80" strokeWidth={2} />
					Invoices
				</a>
			</div>
		{/if}
	</div>

	{#if !isGtm}
		<div class="flex flex-1 items-center justify-center p-6">
			<p class="text-sm text-muted-foreground">
				Outreach sequences are available under <span class="font-mono text-foreground"
					>/dept/gtm/outreach</span
				>.
			</p>
		</div>
	{:else if !sessionId}
		<div class="flex flex-1 items-center justify-center p-6">
			<p class="text-sm text-muted-foreground">Select a session in the top bar.</p>
		</div>
	{:else if loading}
		<div class="p-6 text-sm text-muted-foreground">Loading…</div>
	{:else}
		<div class="min-h-0 flex-1 overflow-auto p-4">
			<div class="mb-6 rounded-lg border border-border bg-card p-4">
				<p class="mb-3 text-sm font-medium text-foreground">New sequence (draft)</p>
				<div class="grid gap-3 lg:grid-cols-2">
					<Input label="Name *" bind:value={newName} size="sm" placeholder="Q1 nurture" />
					<div class="lg:col-span-2">
						<label class="mb-1 block text-[11px] font-medium text-muted-foreground" for="steps-json"
							>Steps (JSON)</label
						>
						<textarea
							id="steps-json"
							bind:value={stepsJson}
							rows="10"
							class="w-full rounded-md border border-input bg-background px-2 py-1.5 font-mono text-xs text-foreground"
							spellcheck="false"
						></textarea>
						<p class="mt-1 text-[11px] text-muted-foreground">
							Array of <span class="font-mono">delay_days</span> (from previous send),
							<span class="font-mono">channel</span> (email), and
							<span class="font-mono">template</span> (passed to the draft step).
						</p>
					</div>
				</div>
				<div class="mt-3 flex justify-end">
					<Button size="sm" disabled={submitting} onclick={() => void onCreate()}>Create draft</Button>
				</div>
			</div>

			<div class="mb-4 flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-end">
				<div class="w-full min-w-[200px] sm:max-w-sm">
					<label class="mb-1 block text-[11px] font-medium text-muted-foreground" for="run-contact"
						>Contact for “Run”</label
					>
					<select
						id="run-contact"
						bind:value={runContactId}
						class="h-9 w-full rounded-md border border-input bg-background px-2 text-sm text-foreground"
					>
						<option value="">Select…</option>
						{#each contacts as c}
							<option value={c.id}>{c.name} — {c.emails[0] ?? c.id}</option>
						{/each}
					</select>
				</div>
				<p class="text-[11px] text-muted-foreground sm:pb-2">
					Requires an active sequence. First job may be delayed per step 0
					<span class="font-mono">delay_days</span>.
				</p>
			</div>

			<div class="overflow-x-auto rounded-lg border border-border">
				<table class="w-full min-w-[640px] border-collapse text-left text-sm">
					<thead class="border-b border-border bg-muted/40 text-[11px] uppercase tracking-wide text-muted-foreground">
						<tr>
							<th class="px-3 py-2 font-medium">Name</th>
							<th class="px-3 py-2 font-medium">Status</th>
							<th class="px-3 py-2 font-medium">Steps</th>
							<th class="px-3 py-2 font-medium text-right">Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each sequences as s}
							<tr class="border-b border-border/80 hover:bg-muted/20">
								<td class="px-3 py-2 font-medium text-foreground">{s.name}</td>
								<td class="px-3 py-2 text-muted-foreground">{s.status}</td>
								<td class="px-3 py-2 text-muted-foreground">{s.steps.length}</td>
								<td class="px-3 py-2 text-right">
									<div class="flex flex-wrap justify-end gap-2">
										{#if s.status === 'Draft'}
											<Button size="sm" variant="secondary" onclick={() => void onActivate(s.id)}>
												Activate
											</Button>
										{/if}
										{#if s.status === 'Active'}
											<Button
												size="sm"
												disabled={!runContactId}
												onclick={() => void onRun(s.id)}
											>
												Run
												<ArrowRight class="ml-1 h-3.5 w-3.5" strokeWidth={2} />
											</Button>
										{/if}
									</div>
								</td>
							</tr>
						{:else}
							<tr>
								<td colspan="4" class="px-3 py-6 text-center text-sm text-muted-foreground">
									No sequences yet — create a draft above.
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
</div>
