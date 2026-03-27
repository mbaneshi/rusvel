<script lang="ts">
	import { page } from '$app/state';
	import { activeSession } from '$lib/stores';
	import {
		getGtmContacts,
		getGtmDeals,
		postGtmContact,
		type GtmContactRow,
		type GtmDealRow
	} from '$lib/api';
	import { toast } from 'svelte-sonner';
	import { ArrowRight, Receipt } from 'lucide-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';

	let sessionId = $state<string | null>(null);
	activeSession.subscribe((s) => (sessionId = s?.id ?? null));

	let contacts: GtmContactRow[] = $state([]);
	let deals: GtmDealRow[] = $state([]);
	let loading = $state(true);
	let submitting = $state(false);
	let search = $state('');
	let tagFilter = $state('');

	let newName = $state('');
	let newEmail = $state('');
	let newCompany = $state('');
	let newRole = $state('');
	let newTags = $state('');

	let deptId = $derived(page.params.id);
	let isGtm = $derived(deptId === 'gtm');

	function dealsForContact(contactId: string): GtmDealRow[] {
		return deals.filter((d) => d.contact_id === contactId);
	}

	function matchesSearch(c: GtmContactRow, q: string): boolean {
		if (!q.trim()) return true;
		const t = q.toLowerCase();
		const emails = c.emails.join(' ').toLowerCase();
		return (
			c.name.toLowerCase().includes(t) ||
			emails.includes(t) ||
			(c.company?.toLowerCase().includes(t) ?? false) ||
			(c.role?.toLowerCase().includes(t) ?? false)
		);
	}

	function matchesTag(c: GtmContactRow, tag: string): boolean {
		if (!tag.trim()) return true;
		return c.tags.some((x) => x.toLowerCase() === tag.trim().toLowerCase());
	}

	let visibleContacts = $derived(
		contacts.filter((c) => matchesSearch(c, search) && matchesTag(c, tagFilter))
	);

	let allTags = $derived([...new Set(contacts.flatMap((c) => c.tags))].sort());

	async function load() {
		if (!sessionId || !isGtm) return;
		loading = true;
		try {
			const [c, d] = await Promise.all([getGtmContacts(sessionId), getGtmDeals(sessionId)]);
			contacts = c;
			deals = d;
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to load CRM');
			contacts = [];
			deals = [];
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (!sessionId || !isGtm) return;
		void load();
	});

	async function submitAdd() {
		if (!sessionId) return;
		const name = newName.trim();
		const email = newEmail.trim();
		if (!name || !email) {
			toast.error('Name and email are required.');
			return;
		}
		submitting = true;
		try {
			const tags = newTags
				.split(',')
				.map((s) => s.trim())
				.filter(Boolean);
			await postGtmContact(sessionId, {
				name,
				email,
				company: newCompany.trim() || null,
				role: newRole.trim() || null,
				tags
			});
			toast.success('Contact added');
			newName = '';
			newEmail = '';
			newCompany = '';
			newRole = '';
			newTags = '';
			await load();
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'Failed to add contact');
		} finally {
			submitting = false;
		}
	}
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
	<div class="flex flex-wrap items-start justify-between gap-2 border-b border-border px-4 py-3">
		<div>
			<h1 class="text-lg font-semibold">CRM contacts</h1>
			<p class="text-xs text-muted-foreground">
				Search and filter session contacts; deal rows link to the same session’s pipeline.
			</p>
		</div>
		{#if isGtm}
			<div class="flex flex-wrap items-center gap-2">
				<a
					href="/dept/gtm/deals"
					class="inline-flex items-center gap-1 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					Deal pipeline
					<ArrowRight class="h-3.5 w-3.5 opacity-70" strokeWidth={2} />
				</a>
				<a
					href="/dept/gtm/outreach"
					class="inline-flex items-center gap-1 rounded-md border border-border bg-secondary/60 px-2.5 py-1.5 text-[11px] font-medium text-foreground hover:bg-accent"
				>
					Outreach
					<ArrowRight class="h-3.5 w-3.5 opacity-70" strokeWidth={2} />
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
				Contacts are available under <span class="font-mono text-foreground">/dept/gtm/contacts</span>.
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
			<div class="mb-4 flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-end">
				<div class="min-w-[180px] flex-1">
					<Input
						label="Search"
						bind:value={search}
						size="sm"
						placeholder="Name, email, company…"
					/>
				</div>
				<div class="w-full min-w-[140px] sm:w-48">
					<label class="mb-1 block text-[11px] font-medium text-muted-foreground" for="tag-filter"
						>Filter by tag</label
					>
					<select
						id="tag-filter"
						bind:value={tagFilter}
						class="h-9 w-full rounded-md border border-input bg-background px-2 text-sm text-foreground"
					>
						<option value="">All tags</option>
						{#each allTags as t}
							<option value={t}>{t}</option>
						{/each}
					</select>
				</div>
				<p class="text-[11px] text-muted-foreground sm:pb-2">
					Showing {visibleContacts.length} of {contacts.length} contacts
				</p>
			</div>

			<div class="mb-6 rounded-lg border border-border bg-card p-4">
				<p class="mb-3 text-sm font-medium text-foreground">Add contact</p>
				<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<Input label="Name *" bind:value={newName} size="sm" placeholder="Jane Doe" />
					<Input label="Email *" bind:value={newEmail} size="sm" placeholder="jane@example.com" />
					<Input label="Company" bind:value={newCompany} size="sm" />
					<Input label="Role" bind:value={newRole} size="sm" />
					<div class="sm:col-span-2">
						<Input
							label="Tags"
							bind:value={newTags}
							size="sm"
							placeholder="comma, separated"
							hint="Stored on the contact; use filter above to narrow."
						/>
					</div>
				</div>
				<div class="mt-3">
					<Button
						size="sm"
						disabled={submitting}
						loading={submitting}
						onclick={() => submitAdd()}>Save contact</Button
					>
				</div>
			</div>

			<div class="overflow-x-auto rounded-lg border border-border">
				<table class="w-full min-w-[640px] border-collapse text-left text-sm">
					<thead>
						<tr class="border-b border-border bg-muted/40">
							<th class="px-3 py-2 font-medium">Name</th>
							<th class="px-3 py-2 font-medium">Email</th>
							<th class="px-3 py-2 font-medium">Company</th>
							<th class="px-3 py-2 font-medium">Role</th>
							<th class="px-3 py-2 font-medium">Tags</th>
							<th class="px-3 py-2 font-medium">Deals</th>
						</tr>
					</thead>
					<tbody>
						{#each visibleContacts as c (c.id)}
							<tr class="border-b border-border/80 hover:bg-muted/20">
								<td class="px-3 py-2 font-medium text-foreground">{c.name}</td>
								<td class="px-3 py-2 text-muted-foreground">{c.emails[0] ?? '—'}</td>
								<td class="px-3 py-2 text-muted-foreground">{c.company ?? '—'}</td>
								<td class="px-3 py-2 text-muted-foreground">{c.role ?? '—'}</td>
								<td class="px-3 py-2">
									{#if c.tags.length}
										<span class="flex flex-wrap gap-1">
											{#each c.tags as tg}
												<span
													class="rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-foreground"
													>{tg}</span
												>
											{/each}
										</span>
									{:else}
										<span class="text-muted-foreground">—</span>
									{/if}
								</td>
								<td class="px-3 py-2 align-top">
									{#each dealsForContact(c.id) as d (d.id)}
										<div class="text-xs">
											<span class="text-foreground">{d.title}</span>
											<span class="text-muted-foreground"> · {d.stage}</span>
											<span class="tabular-nums text-muted-foreground">
												· ${d.value.toFixed(0)}</span
											>
										</div>
									{:else}
										<span class="text-[11px] text-muted-foreground">—</span>
									{/each}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
				{#if visibleContacts.length === 0}
					<p class="p-4 text-center text-sm text-muted-foreground">No contacts match.</p>
				{/if}
			</div>
		</div>
	{/if}
</div>
