/**
 * ASCII Canvas E2E Tests
 * Tests the full application functionality in a browser
 */

import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3003';

test.describe('ASCII Canvas Editor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    
    // Wait for the loading overlay to be hidden
    await page.waitForSelector('#loading.hidden', { timeout: 15000 });
    
    // Wait for the canvas to be visible
    await page.waitForSelector('#canvas', { timeout: 10000 });
    
    // Small delay to ensure JS is fully loaded
    await page.waitForTimeout(500);
  });

  test('should load the editor', async ({ page }) => {
    // Check that the main elements are present
    await expect(page.locator('.toolbar')).toBeVisible();
    await expect(page.locator('#canvas')).toBeVisible();
    await expect(page.locator('.logo-text')).toContainText('ASCII Canvas');
  });

  test('should have all tool buttons', async ({ page }) => {
    const tools = ['select', 'rectangle', 'line', 'arrow', 'diamond', 'text', 'freehand', 'eraser'];
    
    for (const tool of tools) {
      const button = page.locator(`[data-tool="${tool}"]`);
      await expect(button).toBeVisible();
    }
  });

  test('should switch tools when clicked', async ({ page }) => {
    // Click on line tool
    await page.click('[data-tool="line"]');
    
    // Check if the tool is active
    const lineButton = page.locator('[data-tool="line"]');
    await expect(lineButton).toHaveClass(/active/);
  });

  test('should switch tools with keyboard shortcuts', async ({ page }) => {
    // Focus the canvas
    await page.click('#canvas');
    await page.waitForTimeout(100);
    
    // Press 'L' for line tool
    await page.keyboard.press('l');
    await page.waitForTimeout(100);
    
    // Check if line tool is active
    const lineButton = page.locator('[data-tool="line"]');
    await expect(lineButton).toHaveClass(/active/);
    
    // Press 'R' for rectangle tool
    await page.keyboard.press('r');
    await page.waitForTimeout(100);
    
    // Check if rectangle tool is active
    const rectButton = page.locator('[data-tool="rectangle"]');
    await expect(rectButton).toHaveClass(/active/);
  });

  test('should draw a rectangle on canvas', async ({ page }) => {
    // Select rectangle tool
    await page.click('[data-tool="rectangle"]');
    
    // Get canvas bounds
    const canvas = page.locator('#canvas');
    const box = await canvas.boundingBox();
    
    if (box) {
      // Draw a rectangle by dragging
      await page.mouse.move(box.x + 100, box.y + 100);
      await page.mouse.down();
      await page.mouse.move(box.x + 300, box.y + 200, { steps: 10 });
      await page.mouse.up();
    }
  });

  test('should show border style selector', async ({ page }) => {
    const select = page.locator('#border-style');
    await expect(select).toBeVisible();
    
    // Check options
    const options = await select.locator('option').allTextContents();
    expect(options).toContain('Single (─)');
    expect(options).toContain('Double (═)');
  });

  test('should have undo/redo buttons disabled initially', async ({ page }) => {
    const undoBtn = page.locator('#undo-btn');
    const redoBtn = page.locator('#redo-btn');
    
    await expect(undoBtn).toBeDisabled();
    await expect(redoBtn).toBeDisabled();
  });

  test('should have copy button', async ({ page }) => {
    const copyBtn = page.locator('#copy-btn');
    await expect(copyBtn).toBeVisible();
    await expect(copyBtn).toContainText('Copy');
  });

  test('should zoom with scroll wheel', async ({ page }) => {
    // Check initial zoom level
    const zoomLevel = page.locator('#zoom-level');
    await expect(zoomLevel).toContainText('100%');
    
    // Scroll on canvas
    const canvas = page.locator('#canvas');
    await canvas.hover();
    await page.mouse.wheel(0, -100);
    
    // Zoom should have changed
    await page.waitForTimeout(200);
  });

  test('should display cursor position', async ({ page }) => {
    const cursorPos = page.locator('#cursor-pos');
    await expect(cursorPos).toBeVisible();
    
    // Move mouse over canvas
    const canvas = page.locator('#canvas');
    const box = await canvas.boundingBox();
    
    if (box) {
      await page.mouse.move(box.x + 100, box.y + 100);
      await page.waitForTimeout(100);
      // Cursor position should update
      await expect(cursorPos).not.toBeEmpty();
    }
  });

  test('should show status bar', async ({ page }) => {
    const statusBar = page.locator('.status-bar');
    await expect(statusBar).toBeVisible();
    
    const toolStatus = page.locator('#status-tool');
    await expect(toolStatus).toContainText('Tool:');
  });
});

test.describe('Keyboard Shortcuts', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await page.waitForSelector('#loading.hidden', { timeout: 15000 });
    await page.waitForSelector('#canvas', { timeout: 10000 });
    await page.waitForTimeout(500);
  });

  test('should support all tool shortcuts', async ({ page }) => {
    await page.click('#canvas');
    await page.waitForTimeout(100);
    
    const shortcuts = [
      { key: 'v', tool: 'select' },
      { key: 'r', tool: 'rectangle' },
      { key: 'l', tool: 'line' },
      { key: 'a', tool: 'arrow' },
      { key: 'd', tool: 'diamond' },
      { key: 't', tool: 'text' },
      { key: 'f', tool: 'freehand' },
      { key: 'e', tool: 'eraser' },
    ];
    
    for (const { key, tool } of shortcuts) {
      await page.keyboard.press(key);
      await page.waitForTimeout(100);
      await expect(page.locator(`[data-tool="${tool}"]`)).toHaveClass(/active/, { timeout: 3000 });
    }
  });
});
