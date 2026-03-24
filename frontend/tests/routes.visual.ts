/**
 * Visual regression tests for all RUSVEL frontend routes.
 *
 * Takes full-page screenshots and compares against baselines.
 * Run `pnpm test:e2e:update` to regenerate baselines.
 */

import { expect } from '@playwright/test';
import { test, setupSession, navigateAndWait } from './fixtures';

const DEPARTMENTS = [
	'forge',
	'code',
	'harvest',
	'content',
	'gtm',
	'finance',
	'product',
	'growth',
	'distro',
	'legal',
	'support',
	'infra'
];

test.describe('Visual Regression — Core Routes', () => {
	test.beforeEach(async ({ page }) => {
		await setupSession(page);
	});

	test('dashboard /', async ({ page }) => {
		await navigateAndWait(page, '/');
		await expect(page).toHaveScreenshot('dashboard.png', {
			fullPage: true,
			maxDiffPixelRatio: 0.05
		});
	});

	test('settings /settings', async ({ page }) => {
		await navigateAndWait(page, '/settings');
		await expect(page).toHaveScreenshot('settings.png', {
			fullPage: true,
			maxDiffPixelRatio: 0.05
		});
	});

	test('knowledge /knowledge', async ({ page }) => {
		await navigateAndWait(page, '/knowledge');
		await expect(page).toHaveScreenshot('knowledge.png', {
			fullPage: true,
			maxDiffPixelRatio: 0.05
		});
	});

	test('chat /chat', async ({ page }) => {
		await navigateAndWait(page, '/chat');
		await expect(page).toHaveScreenshot('chat.png', {
			fullPage: true,
			maxDiffPixelRatio: 0.05
		});
	});
});

test.describe('Visual Regression — RusvelBase / Database', () => {
	test.beforeEach(async ({ page }) => {
		await setupSession(page);
	});

	test('database schema /database/schema', async ({ page }) => {
		await navigateAndWait(page, '/database/schema');
		await expect(page).toHaveScreenshot('database-schema.png', {
			fullPage: true,
			maxDiffPixelRatio: 0.05
		});
	});

	test('database tables /database/tables', async ({ page }) => {
		await navigateAndWait(page, '/database/tables');
		await expect(page).toHaveScreenshot('database-tables.png', {
			fullPage: true,
			maxDiffPixelRatio: 0.05
		});
	});

	test('database sql /database/sql', async ({ page }) => {
		await navigateAndWait(page, '/database/sql');
		await expect(page).toHaveScreenshot('database-sql.png', {
			fullPage: true,
			maxDiffPixelRatio: 0.05
		});
	});
});

test.describe('Visual Regression — Department Pages', () => {
	test.beforeEach(async ({ page }) => {
		await setupSession(page);
	});

	for (const dept of DEPARTMENTS) {
		test(`department ${dept}`, async ({ page }) => {
			await navigateAndWait(page, `/dept/${dept}`);
			await expect(page).toHaveScreenshot(`dept-${dept}.png`, {
				fullPage: true,
				maxDiffPixelRatio: 0.05
			});
		});
	}
});
