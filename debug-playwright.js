const { chromium } = require("playwright");
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  // Enable console logging from the page
  page.on("console", msg => {
    console.log("PAGE LOG:", msg.type(), msg.text());
  });
  
  page.on("pageerror", error => {
    console.log("PAGE ERROR:", error.message);
  });
  
  try {
    await page.goto("http://localhost:3000");
    await page.waitForLoadState("networkidle");
    
    // Check page title and content
    const title = await page.title();
    console.log("Page title:", title);
    
    // Check if there are any canvas elements
    const canvasCount = await page.locator("canvas").count();
    console.log("Canvas elements found:", canvasCount);
    
    // Check for any error messages on screen
    const errorText = await page.locator("body").textContent();
    if (errorText && errorText.includes("Error")) {
      console.log("Error text found on page:", errorText);
    }
    
    // Take a screenshot for debugging
    await page.screenshot({ path: "debug-screenshot.png", fullPage: true });
    console.log("Screenshot saved as debug-screenshot.png");
    
    // Check network requests
    const responses = [];
    page.on("response", response => {
      responses.push({
        url: response.url(),
        status: response.status(),
        ok: response.ok()
      });
    });
    
    // Wait a bit to collect network activity
    await page.waitForTimeout(2000);
    
    console.log("Network requests:");
    responses.forEach(resp => {
      console.log(`  ${resp.status} ${resp.ok ? "OK" : "FAIL"} ${resp.url}`);
    });
    
  } catch (error) {
    console.error("Error navigating to page:", error.message);
  } finally {
    await browser.close();
  }
})()
