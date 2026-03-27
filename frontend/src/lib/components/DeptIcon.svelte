<script lang="ts">
	import {
		Hammer,
		Code,
		Search,
		PenTool,
		Rocket,
		DollarSign,
		Package,
		TrendingUp,
		Share2,
		Scale,
		HeadphonesIcon,
		Server,
		Workflow,
		Layers
	} from 'lucide-svelte';

	type IconComponent = typeof Hammer;

	const MAP: Record<string, IconComponent> = {
		forge: Hammer,
		code: Code,
		harvest: Search,
		content: PenTool,
		gtm: Rocket,
		finance: DollarSign,
		product: Package,
		growth: TrendingUp,
		distro: Share2,
		legal: Scale,
		support: HeadphonesIcon,
		infra: Server,
		flow: Workflow
	};

	let {
		deptId,
		size = 16,
		strokeWidth = 1.75,
		class: className = ''
	}: {
		deptId: string;
		size?: number;
		strokeWidth?: number;
		class?: string;
	} = $props();

	let Icon = $derived(MAP[deptId] ?? Layers);
</script>

<svelte:component this={Icon} {size} {strokeWidth} class={className} />
