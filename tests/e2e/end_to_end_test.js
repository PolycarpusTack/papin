// E2E tests using Playwright
const { test, expect } = require('@playwright/test');

// Test conversation flow
test('conversation flow works correctly', async ({ page }) => {
  // Launch the application
  await page.goto('tauri://localhost');
  
  // Wait for the app to load
  await page.waitForSelector('#app-container', { state: 'visible' });
  
  // Type a message
  await page.fill('#message-input', 'Hello, I need some help with the MCP client.');
  
  // Click send button
  await page.click('#send-button');
  
  // Wait for response
  await page.waitForSelector('.assistant-message:nth-child(2)', { state: 'visible', timeout: 10000 });
  
  // Verify response is displayed
  const responseText = await page.textContent('.assistant-message:nth-child(2)');
  expect(responseText).not.toBeNull();
  expect(responseText.length).toBeGreaterThan(10);
  
  // Type a follow-up question
  await page.fill('#message-input', 'How do I enable offline mode?');
  
  // Click send button
  await page.click('#send-button');
  
  // Wait for second response
  await page.waitForSelector('.assistant-message:nth-child(4)', { state: 'visible', timeout: 10000 });
  
  // Verify second response is displayed and mentions offline mode
  const secondResponseText = await page.textContent('.assistant-message:nth-child(4)');
  expect(secondResponseText).toContain('offline');
});

// Test settings functionality
test('settings functionality works correctly', async ({ page }) => {
  // Launch the application
  await page.goto('tauri://localhost');
  
  // Wait for the app to load
  await page.waitForSelector('#app-container', { state: 'visible' });
  
  // Open settings
  await page.click('#settings-button');
  
  // Wait for settings modal
  await page.waitForSelector('#settings-modal', { state: 'visible' });
  
  // Navigate to appearance settings
  await page.click('#appearance-tab');
  
  // Change theme
  await page.click('#theme-selector-dark');
  
  // Wait for theme change
  await page.waitForSelector('body.dark-theme');
  
  // Verify theme was applied
  const isDarkTheme = await page.evaluate(() => {
    return document.body.classList.contains('dark-theme');
  });
  expect(isDarkTheme).toBe(true);
  
  // Navigate to offline settings
  await page.click('#offline-tab');
  
  // Toggle offline mode
  const initialOfflineState = await page.isChecked('#offline-mode-toggle');
  await page.click('#offline-mode-toggle');
  
  // Verify toggle changed state
  const newOfflineState = await page.isChecked('#offline-mode-toggle');
  expect(newOfflineState).not.toEqual(initialOfflineState);
  
  // Close settings
  await page.click('#close-settings-button');
  
  // Verify settings were closed
  await expect(page.locator('#settings-modal')).toBeHidden();
});

// Test offline capabilities
test('offline capabilities work correctly', async ({ page }) => {
  // Launch the application
  await page.goto('tauri://localhost');
  
  // Wait for the app to load
  await page.waitForSelector('#app-container', { state: 'visible' });
  
  // Enable offline mode through settings
  await page.click('#settings-button');
  await page.waitForSelector('#settings-modal', { state: 'visible' });
  await page.click('#offline-tab');
  
  // Ensure offline mode is enabled
  if (!await page.isChecked('#offline-mode-toggle')) {
    await page.click('#offline-mode-toggle');
    // Wait for offline mode to be activated
    await page.waitForSelector('#connection-status:has-text("Offline")', { timeout: 5000 });
  }
  
  // Close settings
  await page.click('#close-settings-button');
  
  // Verify offline indicator is displayed
  const statusText = await page.textContent('#connection-status');
  expect(statusText).toBe('Offline');
  
  // Send a message in offline mode
  await page.fill('#message-input', 'Can you help me with something while offline?');
  await page.click('#send-button');
  
  // Wait for response
  await page.waitForSelector('.assistant-message:nth-child(2)', { state: 'visible', timeout: 10000 });
  
  // Verify response mentions offline mode
  const responseText = await page.textContent('.assistant-message:nth-child(2)');
  expect(responseText).toContain('offline') || expect(responseText).toContain('local');
  
  // Try to create a new conversation (should work even offline)
  await page.click('#new-conversation-button');
  
  // Verify new conversation was created
  await expect(page.locator('#message-input')).toBeEmpty();
  await expect(page.locator('.message-container')).toBeEmpty();
});

// Test performance monitoring
test('performance monitoring dashboard works correctly', async ({ page }) => {
  // Launch the application
  await page.goto('tauri://localhost');
  
  // Wait for the app to load
  await page.waitForSelector('#app-container', { state: 'visible' });
  
  // Open resource dashboard
  await page.click('#tools-menu');
  await page.click('#resource-dashboard');
  
  // Wait for dashboard to load
  await page.waitForSelector('#resource-dashboard-container', { state: 'visible' });
  
  // Verify charts are displayed
  await expect(page.locator('#memory-usage-chart')).toBeVisible();
  await expect(page.locator('#api-latency-chart')).toBeVisible();
  await expect(page.locator('#token-usage-chart')).toBeVisible();
  
  // Change timeframe
  await page.click('#timeframe-selector-24h');
  
  // Wait for charts to update
  await page.waitForTimeout(500);
  
  // Verify charts contain data (by checking for canvas elements with content)
  const memoryChartEmpty = await page.evaluate(() => {
    const canvas = document.querySelector('#memory-usage-chart canvas');
    const context = canvas.getContext('2d');
    const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
    const data = imageData.data;
    
    // Check if canvas is empty (all pixels are transparent)
    for (let i = 3; i < data.length; i += 4) {
      if (data[i] !== 0) return false;
    }
    return true;
  });
  
  expect(memoryChartEmpty).toBe(false);
  
  // Close dashboard
  await page.click('#close-dashboard-button');
  
  // Verify dashboard was closed
  await expect(page.locator('#resource-dashboard-container')).toBeHidden();
});

// Test auto-update functionality (mock)
test('auto-update notification works correctly', async ({ page }) => {
  // Launch the application
  await page.goto('tauri://localhost');
  
  // Wait for the app to load
  await page.waitForSelector('#app-container', { state: 'visible' });
  
  // Inject mock update event (this would normally come from Tauri)
  await page.evaluate(() => {
    window.dispatchEvent(new CustomEvent('tauri://update-available', { 
      detail: { version: '1.1.0', body: 'New features and bug fixes' } 
    }));
  });
  
  // Verify update notification is displayed
  await page.waitForSelector('#update-notification', { state: 'visible' });
  
  const notificationText = await page.textContent('#update-notification');
  expect(notificationText).toContain('1.1.0');
  
  // Click "Update Now" button
  await page.click('#update-now-button');
  
  // Verify update progress is displayed
  await page.waitForSelector('#update-progress', { state: 'visible' });
  
  // Inject mock update completion event
  await page.evaluate(() => {
    window.dispatchEvent(new CustomEvent('tauri://update-downloaded'));
  });
  
  // Verify restart notification is displayed
  await page.waitForSelector('#restart-notification', { state: 'visible' });
});