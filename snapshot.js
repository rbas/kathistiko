const puppeteer = require("puppeteer");

async function main() {
  const url = process.env.URL;
  const outputFilename = process.env.OUTPUT_FILENAME;

  if (!url || !outputFilename) {
    throw new Error("URL and OUTPUT_FILENAME must be configured");
  }

  const browser = await puppeteer.launch({
    headless: true,
    args: ["--no-sandbox", "--disable-setuid-sandbox"],
  });

  try {
    const page = await browser.newPage();
    await page.setViewport({ width: 480, height: 800 });
    await page.goto(url, { waitUntil: "networkidle0", timeout: 60_000 });
    await page.screenshot({ path: `/app/output/${outputFilename}` });
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
