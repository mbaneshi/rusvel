<script lang="ts">
	import { onMount } from 'svelte';
	import { driver } from 'driver.js';
	import 'driver.js/dist/driver.css';
	import { onboarding } from '$lib/stores';
	import type { OnboardingState } from '$lib/stores';

	let ob: OnboardingState = $state({
		sessionCreated: false,
		goalAdded: false,
		planGenerated: false,
		deptChatUsed: false,
		agentCreated: false,
		dismissed: false,
		tourCompleted: false
	});

	onboarding.subscribe((v) => (ob = v));

	onMount(() => {
		if (ob.tourCompleted) return;

		// Small delay to let layout render
		const timeout = setTimeout(() => {
			const driverObj = driver({
				showProgress: true,
				animate: true,
				overlayColor: 'rgba(0, 0, 0, 0.7)',
				popoverClass: 'rusvel-tour-popover',
				steps: [
					{
						element: '[data-tour="sidebar-logo"]',
						popover: {
							title: 'Welcome to RUSVEL',
							description:
								'Your AI-powered virtual agency. Every department has its own AI agent ready to help.',
							side: 'bottom',
							align: 'start'
						}
					},
					{
						element: '[data-tour="session-switcher"]',
						popover: {
							title: 'Sessions',
							description:
								'Sessions are your workspaces. Create one for each project, lead, or campaign.',
							side: 'bottom',
							align: 'start'
						}
					},
					{
						element: '[data-tour="nav-forge"]',
						popover: {
							title: 'Forge Department',
							description:
								'Your mission control. Set goals, generate daily plans, and run reviews here.',
							side: 'bottom',
							align: 'start'
						}
					},
					{
						element: '[data-tour="nav-chat"]',
						popover: {
							title: 'God Agent Chat',
							description: 'Chat with the central AI that has authority over all departments.',
							side: 'bottom',
							align: 'start'
						}
					},
					{
						element: '[data-tour="nav-settings"]',
						popover: {
							title: 'Settings',
							description: 'Check system health, engine status, and configure your instance.',
							side: 'bottom',
							align: 'start'
						}
					}
				],
				onDestroyed: () => {
					onboarding.complete('tourCompleted');
				}
			});

			driverObj.drive();
		}, 1000);

		return () => clearTimeout(timeout);
	});
</script>

<style>
	:global(.rusvel-tour-popover) {
		background: #0f172a !important;
		color: #f1f5f9 !important;
		border: 1px solid #334155 !important;
		border-radius: 12px !important;
		box-shadow: 0 25px 50px rgba(0, 0, 0, 0.5) !important;
	}
	:global(.rusvel-tour-popover .driver-popover-title) {
		color: #f1f5f9 !important;
		font-size: 14px !important;
	}
	:global(.rusvel-tour-popover .driver-popover-description) {
		color: #94a3b8 !important;
		font-size: 13px !important;
	}
	:global(.rusvel-tour-popover .driver-popover-progress-text) {
		color: #64748b !important;
	}
	:global(.rusvel-tour-popover button) {
		background: #4f46e5 !important;
		color: white !important;
		border: none !important;
		border-radius: 6px !important;
		font-size: 12px !important;
		padding: 6px 14px !important;
	}
	:global(.rusvel-tour-popover .driver-popover-close-btn) {
		background: transparent !important;
		color: #64748b !important;
	}
</style>
