/**
 * Simple class name merge utility.
 * Filters falsy values, flattens arrays, and joins with space.
 */
type ClassValue = string | number | false | null | undefined;

export function cn(...classes: (ClassValue | ClassValue[])[]): string {
	return classes.flat().filter(Boolean).join(' ');
}
