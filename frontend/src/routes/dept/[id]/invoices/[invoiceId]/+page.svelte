<script lang="ts">
	import { page } from '$app/state';
	import { activeSession } from '$lib/stores';
	import {
		getGtmInvoice,
		postGtmInvoiceStatus,
		type GtmInvoiceDetail,
		type GtmInvoiceStatus
	} from '$lib/api';
	import { toast } from 'svelte-sonner';
	import Button from '$lib/components/ui/Button.svelte';

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	let inv = $state<GtmInvoiceDetail | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let statusPick = $state<GtmInvoiceStatus>('Draft');

	const STATUSES: GtmInvoiceStatus[] = [
		'Draft',
		'Sent',
		'Paid',
		'Overdue',
		'Cancelled'
	];

	let deptId = $derived(page.params.id);
	let invoiceId = $derived(page.params.invoiceId ?? '');
	let isGtm = $derived(deptId === 'gtm');

	const money = new Intl.NumberFormat(undefined, {
		style: 'currency',
		currency: 'USD',
		maximumFractionDigits: 2
	});

	async function load() {
		if (!sessionId || !isGtm || !invoiceId) return;
		loading = true;
		try {
			const r = await getGtmInvoice(sessionId, invoiceId);
			inv = r;
			statusPick = r.status;
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load invoice');
			inv = null;
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (!sessionId || !isGtm || !invoiceId) return;
		void load();
	});

	async function saveStatus() {
		if (!sessionId || !inv) return;
		saving = true;
		try {
			await postGtmInvoiceStatus(sessionId, inv.id, statusPick);
			toast.success('Status updated');
			await load();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Update failed');
		} finally {
			saving = false;
		}
	}

	function printPage() {
		window.print();
	}

	function fmtWhen(iso: string): string {
		try {
			return new Date(iso).toLocaleString(undefined, { dateStyle: 'long', timeStyle: 'short' });
		} catch {
			return iso;
		}
	}
</script>

<div class="flex h-full min-h-0 flex-col overflow-y-auto print:overflow-visible">
	<div class="no-print border-b border-border px-4 py-3">
		<div class="mx-auto flex max-w-3xl flex-wrap items-center justify-between gap-2">
			<div>
				<a
					class="text-[11px] text-muted-foreground hover:text-foreground"
					href="/dept/{deptId}/invoices">← Invoices</a
				>
				<h1 class="mt-1 text-lg font-semibold">Invoice</h1>
			</div>
			<div class="flex flex-wrap gap-2">
				<label class="flex items-center gap-2 text-xs">
					<span class="text-muted-foreground">Status</span>
					<select
						class="rounded-md border border-border bg-background px-2 py-1.5 text-xs"
						bind:value={statusPick}
						disabled={!inv || saving}
					>
						{#each STATUSES as s}
							<option value={s}>{s}</option>
						{/each}
					</select>
				</label>
				<Button
					variant="secondary"
					size="sm"
					disabled={!inv || saving || statusPick === inv?.status}
					loading={saving}
					onclick={saveStatus}>Save status</Button
				>
				<Button variant="outline" size="sm" onclick={printPage}>Print / PDF</Button>
			</div>
		</div>
	</div>

	{#if !isGtm}
		<p class="p-6 text-sm text-muted-foreground">Open from GTM department.</p>
	{:else if !sessionId}
		<p class="p-6 text-sm text-muted-foreground">Select a session.</p>
	{:else if loading}
		<p class="p-6 text-sm text-muted-foreground">Loading…</p>
	{:else if !inv}
		<p class="p-6 text-sm text-muted-foreground">Invoice not found.</p>
	{:else}
		<div class="mx-auto max-w-3xl px-4 py-8 print:max-w-none print:px-8 print:py-10">
			<section
				id="invoice-print"
				class="rounded-xl border border-border bg-card p-8 shadow-sm print:border-0 print:shadow-none print:bg-white"
			>
				<header class="border-b border-border pb-6 print:border-slate-300">
					<p class="text-[10px] font-semibold uppercase tracking-[0.2em] text-muted-foreground">
						Invoice
					</p>
					<h2 class="mt-1 font-mono text-sm text-foreground">#{inv.id}</h2>
					<div class="mt-4 grid gap-1 text-sm">
						<p>
							<span class="text-muted-foreground">Bill to:</span>
							<strong class="ml-2 text-foreground">{inv.contact_name ?? 'Contact'}</strong>
						</p>
						<p class="text-xs text-muted-foreground">Contact ID {inv.contact_id}</p>
					</div>
				</header>

				<div class="mt-6 overflow-x-auto">
					<table class="w-full text-left text-sm">
						<thead>
							<tr class="border-b border-border text-[10px] uppercase tracking-wide text-muted-foreground">
								<th class="py-2 pr-2">Description</th>
								<th class="w-20 py-2 pr-2 text-right">Qty</th>
								<th class="w-28 py-2 pr-2 text-right">Unit</th>
								<th class="w-28 py-2 text-right">Subtotal</th>
							</tr>
						</thead>
						<tbody>
							{#each inv.items as item}
								<tr class="border-b border-border/60">
									<td class="py-2 pr-2">{item.description}</td>
									<td class="py-2 pr-2 text-right tabular-nums">{item.quantity}</td>
									<td class="py-2 pr-2 text-right tabular-nums">{money.format(item.unit_price)}</td>
									<td class="py-2 text-right tabular-nums">{money.format(item.quantity * item.unit_price)}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				<footer class="mt-8 flex flex-col gap-4 border-t border-border pt-6 print:border-slate-300">
					<div class="flex justify-end">
						<p class="text-lg font-semibold tabular-nums">
							Total: {money.format(inv.total)}
						</p>
					</div>
					<div class="grid gap-1 text-xs text-muted-foreground sm:grid-cols-2">
						<p>
							<span class="font-medium text-foreground">Due:</span>
							{fmtWhen(inv.due_date)}
						</p>
						<p>
							<span class="font-medium text-foreground">Status:</span>
							{inv.status}
						</p>
						{#if inv.paid_at}
							<p class="sm:col-span-2">
								<span class="font-medium text-foreground">Paid at:</span>
								{fmtWhen(inv.paid_at)}
							</p>
						{/if}
					</div>
				</footer>
			</section>
		</div>
	{/if}
</div>

<style>
	@media print {
		.no-print {
			display: none !important;
		}
	}
</style>
