import Parser from "rss-parser";

export const PROCESSED_FILE = "./processed.json";
export const INTERVAL_MS = 
  process.env.INTERVAL_MINUTES
    ? parseInt(process.env.INTERVAL_MINUTES) * 60 * 1000
    : 5 * 60 * 1000;

export const parser = new Parser({
  headers: {
    "User-Agent": process.env.USER_AGENT || "NekoRSS/1.0 (+abuse@imnya.ng)",
  },
});

export const token = process.env.DISCORD_TOKEN;

if (!token) {
  console.error("Discord Token is not set in environment variables.");
  process.exit(1);
}
