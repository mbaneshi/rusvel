<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { activeSession } from '$lib/stores';
	import {
		getGtmContacts,
		getGtmInvoices,
		postGtmInvoice,
		type GtmContactRow,
		type GtmInvoiceRow,
		type GtmInvoiceStatus,
		type GtmLineItem
	} from '$lib/api';
	import { toast } from 'svelte-sonner';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';

	const STATUS_OPTS: (GtmInvoiceStatus | '')[] = [
		'',
		'Draft',
		'Sent',
		'Paid',
		'Overdue',
		'Cancelled'
	];

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	let rows: GtmInvoiceRow[] = $state([]);
	let contacts: GtmContactRow[] = $state([]);
	let loading = $state(true);
	let submitting = $state(false);
	let statusFilter = $state<GtmInvoiceStatus | ''>('');

	let newContactId = $state('');
	let dueLocal = $state('');
	let lines = $state<{ description: string; quantity: string; unit_price: string }[]>([
		{ description: '', quantity: '1', unit_price: '0' }
	]);

	let deptId = $derived(page.params.id);
	let isGtm = $derived(deptId === 'gtm');

	const money = new Intl.NumberFormat(undefined, {
		style: 'currency',
		currency: 'USD',
		maximumFractionDigits: 2
	});

	function toIsoFromLocal(local: string): string {
		const d = new Date(local);
		if (Number.isNaN(d.getTime())) throw new Error('Invalid due date');
		return d.toISOString();
	}

	function parseLines(): GtmLineItem[] {
		const out: GtmLineItem[] = [];
		for (const row of lines) {
			const q = Number.parseFloat(row.quantity);
			const p = Number.parseFloat(row.unit_price);
			if (!row.description.trim()) continue;
			if (Number.isNaN(q) || Number.isNaN(p)) {
				throw new Error('Line items need numeric quantity and unit price');
			}
			out.push({ description: row.description.trim(), quantity: q, unit_price: p });
		}
		return out;
	}

	async function load() {
		if (!sessionId || !isGtm) return;
		loading = true;
		try {
			const [inv, c] = await Promise.all([
				getGtmInvoices(sessionId, statusFilter || undefined),
				getGtmContacts(sessionId)
			]);
			rows = inv;
			contacts = c;
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load invoices');
			rows = [];
			contacts = [];
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (!sessionId || !isGtm) return;
		void load();
	});

	function addLine() {
		lines = [...lines, { description: '', quantity: '1', unit_price: '0' }];
	}

	function removeLine(i: number) {
		lines = lines.filter((_, j) => j !== i);
		if (lines.length === 0) lines = [{ description: '', quantity: '1', unit_price: '0' }];
	}

	async function createInvoice() {
		if (!sessionId) return;
		if (!newContactId.trim()) {
			toast.error('Select a contact');
			return;
		}
		if (!dueLocal.trim()) {
			toast.error('Set a due date');
			return;
		}
		let items: GtmLineItem[];
		try {
			items = parseLines();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Invalid line items');
			return;
		}
		if (items.length === 0) {
			toast.error('Add at least one line with a description');
			return;
		}
		submitting = true;
		try {
			let due_date: string;
			try {
				due_date = toIsoFromLocal(dueLocal);
			} catch {
				toast.error('Invalid due date');
				return;
			}
			const { id } = await postGtmInvoice({
				session_id: sessionId,
				contact_id: newContactId,
				items,
				due_date
			});
			toast.success('Invoice created');
			goto(`/dept/${deptId}/invoices/${id}`);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Create failed');
		} finally {
			submitting = false;
		}
	}

	function dueLabel(iso: string): string {
		try {
			return new Date(iso).toLocaleString(undefined, {
				dateStyle: 'medium',
				timeStyle: 'short'
			});
		} catch {
			return iso;
		}
	}

	function isPastDue(r: GtmInvoiceRow): boolean {
		if (r.status === 'Paid' || r.status === 'Cancelled') return false;
		try {
			return new Date(r.due_date).getTime() < Date.now();
		} catch {
			return false;
		}
	}

	let sortedRows = $derived(
		[...rows].sort((a, b) => new Date(b.due_date).getTime() - new Date(a.due_date).getTime())
	);
</script>

<div class="flex h-full min-h-0 flex-col overflow-y-auto">
	<div class="flex flex-wrap items-start justify-between gap-2 border-b border-border px-4 py-3">
		<div>
			<h1 class="text-lg font-semibold">Invoices</h1>
			<p class="text-xs text-muted-foreground">
				Create and track invoices (Draft → Sent → Paid / Overdue). Detail view supports print-to-PDF from
				the browser.
			</p>
		</div>
		{#if isGtm}
			<div class="flex flex-wrap items-center gap-2">
				<a
					href="/dept/gtm/contacts"
					class="inline-flex items-center gap-1.5 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					Contacts
				</a>
				<a
					href="/dept/gtm/deals"
					class="inline-flex items-center gap-1.5 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					Deals
				</a>
				<a
					href="/dept/gtm/outreach"
					class="inline-flex items-center gap-1.5 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					Outreach
				</a>
			</div>
		{/if}
	</div>

	{#if !isGtm}
		<div class="flex flex-1 items-center justify-center p-6">
			<p class="text-sm text-muted-foreground">
				Invoices live under <span class="font-mono text-foreground">/dept/gtm/invoices</span>.
			</p>
		</div>
	{:else if !sessionId}
		<div class="flex flex-1 items-center justify-center p-6">
			<p class="text-sm text-muted-foreground">Select a session in the top bar.</p>
		</div>
	{:else}
		<div class="mx-auto max-w-5xl space-y-8 p-4 pb-10">
			<section class="rounded-lg border border-border bg-card p-4 shadow-sm">
				<h2 class="text-sm font-medium text-foreground">New invoice</h2>
				<p class="mt-0.5 text-[11px] text-muted-foreground">
					Line totals are summed on the server. Currency is USD for display.
				</p>
				<div class="mt-4 grid gap-4 sm:grid-cols-2">
					<div class="sm:col-span-2">
						<label class="mb-1 block text-[11px] font-medium text-muted-foreground" for="inv-contact"
							>Contact</label
						>
						<select
							id="inv-contact"
							class="w-full rounded-md border border-border bg-background px-2 py-2 text-sm text-foreground"
							bind:value={newContactId}
						>
							<option value="">Select contact…</option>
							{#each contacts as c}
								<option value={c.id}>{c.name} ({c.emails[0] ?? 'no email'})</option>
							{/each}
						</select>
					</div>
					<div>
						<label class="mb-1 block text-[11px] font-medium text-muted-foreground" for="inv-due"
							>Due</label
						>
						<input
							id="inv-due"
							type="datetime-local"
							class="w-full rounded-md border border-border bg-background px-2 py-2 text-sm"
							bind:value={dueLocal}
						/>
					</div>
				</div>
				<div class="mt-4 space-y-2">
					<div class="flex items-center justify-between">
						<span class="text-[11px] font-medium text-muted-foreground">Line items</span>
						<Button type="button" variant="secondary" size="sm" onclick={addLine}>Add line</Button>
					</div>
					{#each lines as _, i}
						<div class="flex flex-wrap items-end gap-2 rounded-md border border-border bg-muted/20 p-2">
							<div class="min-w-[180px] flex-1">
								<Input
									label="Description"
									bind:value={lines[i].description}
									placeholder="Service"
								/>
							</div>
							<div class="w-24">
								<Input label="Qty" bind:value={lines[i].quantity} placeholder="1" />
							</div>
							<div class="w-28">
								<Input label="Unit $" bind:value={lines[i].unit_price} placeholder="0" />
							</div>
							<Button
								type="button"
								variant="ghost"
								size="sm"
								class="!h-8"
								onclick={() => removeLine(i)}
								disabled={lines.length <= 1}>Remove</Button
							>
						</div>
					{/each}
				</div>
				<div class="mt-4 flex justify-end">
					<Button
						variant="primary"
						loading={submitting}
						disabled={submitting}
						onclick={createInvoice}>Create invoice</Button
					>
				</div>
			</section>

			<section>
				<div class="mb-3 flex flex-wrap items-center justify-between gap-2">
					<h2 class="text-sm font-medium">All invoices</h2>
					<div class="flex items-center gap-2">
						<label class="text-[11px] text-muted-foreground" for="st-f">Status</label>
						<select
							id="st-f"
							class="rounded-md border border-border bg-background px-2 py-1.5 text-xs"
							bind:value={statusFilter}
							onchange={() => load()}
						>
							{#each STATUS_OPTS as s}
								<option value={s}>{s === '' ? 'All' : s}</option>
							{/each}
						</select>
						<Button variant="secondary" size="sm" onclick={() => load()}>Refresh</Button>
					</div>
				</div>

				{#if loading}
					<p class="text-sm text-muted-foreground">Loading…</p>
				{:else if sortedRows.length === 0}
					<p class="text-sm text-muted-foreground">No invoices for this session yet.</p>
				{:else}
					<div class="overflow-x-auto rounded-lg border border-border">
						<table class="w-full text-left text-xs">
							<thead class="border-b border-border bg-muted/40 text-[10px] uppercase tracking-wide text-muted-foreground">
								<tr>
									<th class="px-3 py-2">Invoice</th>
									<th class="px-3 py-2">Contact</th>
									<th class="px-3 py-2">Total</th>
									<th class="px-3 py-2">Status</th>
									<th class="px-3 py-2">Due</th>
									<th class="px-3 py-2"></th>
								</tr>
							</thead>
							<tbody>
								{#each sortedRows as r}
									<tr class="border-b border-border/80 hover:bg-muted/20">
										<td class="px-3 py-2 font-mono text-[11px] text-foreground"
											>{r.id.slice(0, 8)}…</td
										>
										<td class="px-3 py-2">{r.contact_name ?? '—'}</td>
										<td class="px-3 py-2 tabular-nums">{money.format(r.total)}</td>
										<td class="px-3 py-2">
											<span
												class="rounded-md px-1.5 py-0.5 text-[10px] font-medium {r.status === 'Paid'
													? 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400'
													: r.status === 'Overdue' || isPastDue(r)
														? 'bg-amber-500/15 text-amber-600 dark:text-amber-400'
														: 'bg-secondary text-muted-foreground'}"
											>
												{r.status}
												{#if isPastDue(r) && r.status !== 'Overdue' && r.status !== 'Paid'}
													<span class="text-[9px]">(past due)</span>
												{/if}
											</span>
										</td>
										<td class="px-3 py-2 text-muted-foreground">{dueLabel(r.due_date)}</td>
										<td class="px-3 py-2 text-right">
											<a
												class="text-primary underline-offset-2 hover:underline"
												href="/dept/{deptId}/invoices/{r.id}">Open</a
											>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</section>
		</div>
	{/if}
</div>
