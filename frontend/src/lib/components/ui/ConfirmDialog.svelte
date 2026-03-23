<script lang="ts">
	import Dialog from './Dialog.svelte';
	import Button from './Button.svelte';

	let {
		open = $bindable(false),
		title = 'Confirm',
		description = 'Are you sure?',
		confirmLabel = 'Confirm',
		cancelLabel = 'Cancel',
		variant = 'danger' as 'primary' | 'danger',
		onconfirm
	}: {
		open?: boolean;
		title?: string;
		description?: string;
		confirmLabel?: string;
		cancelLabel?: string;
		variant?: 'primary' | 'danger';
		onconfirm?: () => void;
	} = $props();

	function handleConfirm() {
		onconfirm?.();
		open = false;
	}
</script>

<Dialog bind:open {title} {description}>
	{#snippet footer()}
		<Button variant="ghost" onclick={() => (open = false)}>{cancelLabel}</Button>
		<Button {variant} onclick={handleConfirm}>{confirmLabel}</Button>
	{/snippet}
</Dialog>
