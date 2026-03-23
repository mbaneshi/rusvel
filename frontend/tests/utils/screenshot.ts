/**
 * Screenshot utilities for visual testing and analysis.
 *
 * Used by both Playwright tests and the analysis script.
 */

import fs from 'fs';
import path from 'path';

export interface ScreenshotManifestEntry {
	route: string;
	name: string;
	baselinePath: string | null;
	actualPath: string | null;
	diffPath: string | null;
	status: 'pass' | 'fail' | 'new';
	diffPixelRatio: number | null;
	timestamp: string;
}

export interface ScreenshotManifest {
	runId: string;
	timestamp: string;
	entries: ScreenshotManifestEntry[];
	summary: {
		total: number;
		passed: number;
		failed: number;
		new: number;
	};
}

/**
 * Parse Playwright test results JSON and build a screenshot manifest
 * for the visual analysis script.
 */
export function buildManifest(resultsDir: string): ScreenshotManifest {
	const resultsFile = path.join(resultsDir, 'results.json');
	const entries: ScreenshotManifestEntry[] = [];
	const runId = `run-${Date.now()}`;

	if (!fs.existsSync(resultsFile)) {
		return {
			runId,
			timestamp: new Date().toISOString(),
			entries: [],
			summary: { total: 0, passed: 0, failed: 0, new: 0 }
		};
	}

	const results = JSON.parse(fs.readFileSync(resultsFile, 'utf-8'));

	for (const suite of results.suites ?? []) {
		for (const spec of suite.specs ?? []) {
			for (const test of spec.tests ?? []) {
				for (const result of test.results ?? []) {
					for (const attachment of result.attachments ?? []) {
						if (attachment.name?.endsWith('-actual.png')) {
							const baseName = attachment.name.replace('-actual.png', '');
							const route = routeFromScreenshotName(baseName);

							entries.push({
								route,
								name: baseName,
								baselinePath: findFile(resultsDir, `${baseName}-expected.png`),
								actualPath: attachment.path ?? null,
								diffPath: findFile(resultsDir, `${baseName}-diff.png`),
								status: result.status === 'passed' ? 'pass' : 'fail',
								diffPixelRatio: null,
								timestamp: new Date().toISOString()
							});
						}
					}
				}
			}
		}
	}

	// Also scan for diff images directly in test-results
	const diffFiles = findDiffFiles(resultsDir);
	for (const diffFile of diffFiles) {
		const baseName = path.basename(diffFile).replace('-diff.png', '');
		if (!entries.find((e) => e.name === baseName)) {
			entries.push({
				route: routeFromScreenshotName(baseName),
				name: baseName,
				baselinePath: findFile(resultsDir, `${baseName}-expected.png`),
				actualPath: findFile(resultsDir, `${baseName}-actual.png`),
				diffPath: diffFile,
				status: 'fail',
				diffPixelRatio: null,
				timestamp: new Date().toISOString()
			});
		}
	}

	const summary = {
		total: entries.length,
		passed: entries.filter((e) => e.status === 'pass').length,
		failed: entries.filter((e) => e.status === 'fail').length,
		new: entries.filter((e) => e.status === 'new').length
	};

	return { runId, timestamp: new Date().toISOString(), entries, summary };
}

function routeFromScreenshotName(name: string): string {
	if (name === 'dashboard') return '/';
	if (name === 'settings') return '/settings';
	if (name === 'knowledge') return '/knowledge';
	if (name === 'chat') return '/chat';
	if (name.startsWith('dept-')) return `/dept/${name.replace('dept-', '')}`;
	return `/${name}`;
}

function findFile(dir: string, filename: string): string | null {
	const full = path.join(dir, filename);
	if (fs.existsSync(full)) return full;

	// Search subdirectories
	try {
		for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
			if (entry.isDirectory()) {
				const found = findFile(path.join(dir, entry.name), filename);
				if (found) return found;
			}
		}
	} catch {
		/* ignore */
	}
	return null;
}

function findDiffFiles(dir: string): string[] {
	const results: string[] = [];
	try {
		for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
			const fullPath = path.join(dir, entry.name);
			if (entry.isDirectory()) {
				results.push(...findDiffFiles(fullPath));
			} else if (entry.name.endsWith('-diff.png')) {
				results.push(fullPath);
			}
		}
	} catch {
		/* ignore */
	}
	return results;
}
