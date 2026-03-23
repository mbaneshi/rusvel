<script lang="ts">
	import { page } from '$app/state';
</script>

<div class="flex h-screen items-center justify-center bg-background text-foreground">
	<div class="text-center max-w-md px-6">
		<div
			class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-destructive/15"
		>
			<svg
				class="h-8 w-8 text-destructive"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<circle cx="12" cy="12" r="10" />
				<path d="M12 8v4M12 16h.01" stroke-linecap="round" />
			</svg>
		</div>
		<h1 class="text-2xl font-bold">Something went wrong</h1>
		<p class="mt-2 text-sm text-muted-foreground">
			{page.error?.message ?? 'An unexpected error occurred.'}
		</p>
		<div class="mt-6 flex justify-center gap-3">
			<a
				href="/"
				class="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
			>
				Go to Dashboard
			</a>
			<button
				onclick={() => location.reload()}
				class="rounded-lg border border-border bg-secondary px-4 py-2 text-sm font-medium text-foreground hover:bg-accent"
			>
				Try Again
			</button>
		</div>
	</div>
</div>
