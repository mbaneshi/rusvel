<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import OnboardingChecklist from '$lib/components/onboarding/OnboardingChecklist.svelte';
	import ProductTour from '$lib/components/onboarding/ProductTour.svelte';
	import CommandPalette from '$lib/components/onboarding/CommandPalette.svelte';
	import TopBar from '$lib/components/shell/TopBar.svelte';
	import DeptBar from '$lib/components/shell/DeptBar.svelte';
	import { Toaster } from 'svelte-sonner';

	let { children }: { children: Snippet } = $props();
</script>

<div class="flex h-screen flex-col bg-background text-foreground">
	<TopBar />
	<DeptBar />
	<main class="min-h-0 flex-1 overflow-hidden">
		{@render children()}
	</main>
</div>

<Toaster richColors position="bottom-right" theme="dark" />
<CommandPalette />
<OnboardingChecklist />
<ProductTour />
