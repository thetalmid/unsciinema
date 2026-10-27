# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: unscii.spec.ts >> Unscii Image Converter >> palette preset selection works
- Location: e2e\unscii.spec.ts:137:3

# Error details

```
Test timeout of 30000ms exceeded while running "beforeEach" hook.
```

```
Error: page.waitForFunction: Test timeout of 30000ms exceeded.
```

# Page snapshot

```yaml
- generic [ref=e3]:
  - banner [ref=e4]:
    - heading "Unscii Image Converter" [level=1] [ref=e5]
    - paragraph [ref=e6]: Interactive ASCII art with unscii font
  - main [ref=e7]:
    - generic [ref=e10]:
      - generic [ref=e11]:
        - button "Dither" [ref=e12] [cursor=pointer]
        - button "Font" [ref=e13] [cursor=pointer]
        - button "Render" [ref=e14] [cursor=pointer]
        - button "Color" [ref=e15] [cursor=pointer]
        - button "Advanced" [ref=e16] [cursor=pointer]
      - generic [ref=e18]:
        - generic [ref=e19]:
          - generic [ref=e20]: Algorithm
          - combobox "Algorithm" [disabled] [ref=e21]:
            - option "None" [selected]
            - option "Floyd-Steinberg"
            - option "Atkinson"
            - option "Bayer 4×4"
            - option "Bayer 8×8"
            - option "Blue Noise"
            - option "Sierra"
            - option "Jarvis-Judice-Ninke"
            - option "Stucki"
            - option "Burkes"
        - generic [ref=e22]:
          - generic [ref=e23]: "Strength: 1.00"
          - 'slider "Strength: 1.00" [disabled] [ref=e24]': "1"
        - generic [ref=e25]:
          - generic [ref=e26]: "Threshold: 0.50"
          - 'slider "Threshold: 0.50" [disabled] [ref=e27]': "0.5"
      - generic [ref=e28]:
        - button "Convert" [disabled] [ref=e29]
        - button "Export" [disabled] [ref=e30]
    - generic [ref=e32] [cursor=pointer]:
      - paragraph [ref=e36]: Drop an image here or click to browse
      - generic [ref=e37]: PNG, JPEG, GIF, WebP
    - generic [ref=e38]:
      - generic [ref=e39]: Drop an image or click to load
      - generic [ref=e40]: "WebGL Error:"
      - generic [ref=e41]: Unscii font by Viznut • Rust/WASM + SolidJS + WebGL2
```

# Test source

```ts
  1   | import { test, expect } from '@playwright/test';
  2   | 
  3   | test.describe('Unscii Image Converter', () => {
  4   |   test.beforeEach(async ({ page }) => {
  5   |     await page.goto('/');
  6   |     // Wait for WASM to initialize
> 7   |     await page.waitForFunction(() => window.UnsciiConverter !== undefined);
      |                ^ Error: page.waitForFunction: Test timeout of 30000ms exceeded.
  8   |   });
  9   | 
  10  |   test('loads and shows main UI', async ({ page }) => {
  11  |     await expect(page.locator('h1')).toContainText('Unscii Image Converter');
  12  |     await expect(page.locator('text=Drop an image here')).toBeVisible();
  13  |     await expect(page.locator('text=Convert')).toBeVisible();
  14  |     await expect(page.locator('text=Export')).toBeVisible();
  15  |   });
  16  | 
  17  |   test('tabs are navigable', async ({ page }) => {
  18  |     await page.click('text=Dither');
  19  |     await expect(page.locator('text=Algorithm')).toBeVisible();
  20  |     
  21  |     await page.click('text=Font');
  22  |     await expect(page.locator('text=Font Variant')).toBeVisible();
  23  |     
  24  |     await page.click('text=Render');
  25  |     await expect(page.locator('text=Color Mode')).toBeVisible();
  26  |     
  27  |     await page.click('text=Color');
  28  |     await expect(page.locator('text=Preset Palettes')).toBeVisible();
  29  |     
  30  |     await page.click('text=Advanced');
  31  |     await expect(page.locator('text=CRT Effects')).toBeVisible();
  32  |   });
  33  | 
  34  |   test('loads image via file input', async ({ page }) => {
  35  |     // Create a test image file
  36  |     const testImage = Buffer.from([
  37  |       0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
  38  |       0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
  39  |       0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08,
  40  |       0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
  41  |       0xDE, 0x00, 0x00, 0x00, 0x1C, 0x49, 0x44, 0x41,
  42  |       0x54, 0x08, 0xD7, 0x63, 0x60, 0x60, 0x60, 0xF0,
  43  |       0x00, 0x00, 0x02, 0xE8, 0x01, 0x0B, 0x1C, 0x3F,
  44  |       0x2D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
  45  |       0x44, 0xAE, 0x42, 0x60, 0x82
  46  |     ]);
  47  |     
  48  |     await page.setInputFiles('input[type="file"]', {
  49  |       name: 'test.png',
  50  |       mimeType: 'image/png',
  51  |       buffer: testImage,
  52  |     });
  53  |     
  54  |     // Wait for image to load
  55  |     await expect(page.locator('text=test.png')).toBeVisible({ timeout: 10000 });
  56  |   });
  57  | 
  58  |   test('conversion works', async ({ page }) => {
  59  |     // First load an image
  60  |     const testImage = Buffer.from([
  61  |       0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
  62  |       0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
  63  |       0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08,
  64  |       0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
  65  |       0xDE, 0x00, 0x00, 0x00, 0x1C, 0x49, 0x44, 0x41,
  66  |       0x54, 0x08, 0xD7, 0x63, 0x60, 0x60, 0x60, 0xF0,
  67  |       0x00, 0x00, 0x02, 0xE8, 0x01, 0x0B, 0x1C, 0x3F,
  68  |       0x2D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
  69  |       0x44, 0xAE, 0x42, 0x60, 0x82
  70  |     ]);
  71  |     
  72  |     await page.setInputFiles('input[type="file"]', {
  73  |       name: 'test.png',
  74  |       mimeType: 'image/png',
  75  |       buffer: testImage,
  76  |     });
  77  |     
  78  |     await expect(page.locator('text=test.png')).toBeVisible({ timeout: 10000 });
  79  |     
  80  |     // Click convert
  81  |     await page.click('text=Convert');
  82  |     
  83  |     // Wait for conversion to complete (canvas should update)
  84  |     await page.waitForTimeout(1000);
  85  |     
  86  |     // Canvas should have content
  87  |     const canvas = page.locator('canvas');
  88  |     await expect(canvas).toBeVisible();
  89  |   });
  90  | 
  91  |   test('export modal opens', async ({ page }) => {
  92  |     // Load an image first
  93  |     const testImage = Buffer.from([
  94  |       0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
  95  |       0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
  96  |       0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08,
  97  |       0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
  98  |       0xDE, 0x00, 0x00, 0x00, 0x1C, 0x49, 0x44, 0x41,
  99  |       0x54, 0x08, 0xD7, 0x63, 0x60, 0x60, 0x60, 0xF0,
  100 |       0x00, 0x00, 0x02, 0xE8, 0x01, 0x0B, 0x1C, 0x3F,
  101 |       0x2D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
  102 |       0x44, 0xAE, 0x42, 0x60, 0x82
  103 |     ]);
  104 |     
  105 |     await page.setInputFiles('input[type="file"]', {
  106 |       name: 'test.png',
  107 |       mimeType: 'image/png',
```