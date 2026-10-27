import { test, expect } from '@playwright/test';

test.describe('Unscii Image Converter', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to initialize
    await page.waitForFunction(() => window.UnsciiConverter !== undefined);
  });

  test('loads and shows main UI', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Unscii Image Converter');
    await expect(page.locator('text=Drop an image here')).toBeVisible();
    await expect(page.locator('text=Convert')).toBeVisible();
    await expect(page.locator('text=Export')).toBeVisible();
  });

  test('tabs are navigable', async ({ page }) => {
    await page.click('text=Dither');
    await expect(page.locator('text=Algorithm')).toBeVisible();
    
    await page.click('text=Font');
    await expect(page.locator('text=Font Variant')).toBeVisible();
    
    await page.click('text=Render');
    await expect(page.locator('text=Color Mode')).toBeVisible();
    
    await page.click('text=Color');
    await expect(page.locator('text=Preset Palettes')).toBeVisible();
    
    await page.click('text=Advanced');
    await expect(page.locator('text=CRT Effects')).toBeVisible();
  });

  test('loads image via file input', async ({ page }) => {
    // Create a test image file
    const testImage = Buffer.from([
      0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
      0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
      0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08,
      0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
      0xDE, 0x00, 0x00, 0x00, 0x1C, 0x49, 0x44, 0x41,
      0x54, 0x08, 0xD7, 0x63, 0x60, 0x60, 0x60, 0xF0,
      0x00, 0x00, 0x02, 0xE8, 0x01, 0x0B, 0x1C, 0x3F,
      0x2D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
      0x44, 0xAE, 0x42, 0x60, 0x82
    ]);
    
    await page.setInputFiles('input[type="file"]', {
      name: 'test.png',
      mimeType: 'image/png',
      buffer: testImage,
    });
    
    // Wait for image to load
    await expect(page.locator('text=test.png')).toBeVisible({ timeout: 10000 });
  });

  test('conversion works', async ({ page }) => {
    // First load an image
    const testImage = Buffer.from([
      0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
      0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
      0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08,
      0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
      0xDE, 0x00, 0x00, 0x00, 0x1C, 0x49, 0x44, 0x41,
      0x54, 0x08, 0xD7, 0x63, 0x60, 0x60, 0x60, 0xF0,
      0x00, 0x00, 0x02, 0xE8, 0x01, 0x0B, 0x1C, 0x3F,
      0x2D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
      0x44, 0xAE, 0x42, 0x60, 0x82
    ]);
    
    await page.setInputFiles('input[type="file"]', {
      name: 'test.png',
      mimeType: 'image/png',
      buffer: testImage,
    });
    
    await expect(page.locator('text=test.png')).toBeVisible({ timeout: 10000 });
    
    // Click convert
    await page.click('text=Convert');
    
    // Wait for conversion to complete (canvas should update)
    await page.waitForTimeout(1000);
    
    // Canvas should have content
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();
  });

  test('export modal opens', async ({ page }) => {
    // Load an image first
    const testImage = Buffer.from([
      0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
      0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
      0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08,
      0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
      0xDE, 0x00, 0x00, 0x00, 0x1C, 0x49, 0x44, 0x41,
      0x54, 0x08, 0xD7, 0x63, 0x60, 0x60, 0x60, 0xF0,
      0x00, 0x00, 0x02, 0xE8, 0x01, 0x0B, 0x1C, 0x3F,
      0x2D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
      0x44, 0xAE, 0x42, 0x60, 0x82
    ]);
    
    await page.setInputFiles('input[type="file"]', {
      name: 'test.png',
      mimeType: 'image/png',
      buffer: testImage,
    });
    
    await expect(page.locator('text=test.png')).toBeVisible({ timeout: 10000 });
    await page.click('text=Convert');
    await page.waitForTimeout(1000);
    
    // Open export modal
    await page.click('text=Export');
    
    await expect(page.locator('text=Export Options')).toBeVisible();
    await expect(page.locator('text=WebP')).toBeVisible();
    await expect(page.locator('text=HTML')).toBeVisible();
    await expect(page.locator('text=ANSI')).toBeVisible();
  });

  test('parameter changes update UI', async ({ page }) => {
    // Test dithering algorithm select
    await page.selectOption('select >> nth=0', '1'); // Floyd-Steinberg
    
    // Test cell size slider
    const cellSizeSlider = page.locator('input[type="range"] >> nth=1');
    await cellSizeSlider.fill('16');
    
    // Test color mode select
    await page.click('text=Render');
    await page.selectOption('select >> nth=0', '2'); // Palette mode
  });

  test('palette preset selection works', async ({ page }) => {
    await page.click('text=Color');
    
    // Click a palette preset
    await page.click('text=gameboy');
    
    // Should switch to palette mode
    await page.click('text=Render');
    await expect(page.locator('select >> nth=0')).toHaveValue('2');
  });
});