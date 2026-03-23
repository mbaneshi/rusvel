/**
 * Maps department color names to HSL values for CSS custom properties.
 * Usage: set --dept-hue, --dept-sat, --dept-light on the container,
 * then use dept-accent color in Tailwind via @theme inline.
 */
export const deptColorMap: Record<string, string> = {
	emerald: '160 84% 39%',
	purple: '271 91% 65%',
	amber: '38 92% 50%',
	cyan: '192 91% 36%',
	indigo: '239 84% 67%',
	rose: '347 77% 50%',
	sky: '199 89% 48%',
	orange: '25 95% 53%',
	lime: '84 85% 43%',
	pink: '330 81% 60%',
	teal: '173 80% 36%',
	violet: '263 70% 50%'
};

export function getDeptColor(color: string): string {
	return deptColorMap[color] ?? deptColorMap.indigo;
}
