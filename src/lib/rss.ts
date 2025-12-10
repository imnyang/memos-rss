import { ForumChannel, ThreadAutoArchiveDuration, Client } from "discord.js";
import type Parser from "rss-parser";
import { parser } from "./config";
import { processedItems, saveCurrentState } from "./storage";
import type { RssSource } from "../type/types";
import rssConfig from "@/../rss.toml";

export async function checkRssFeeds(client: Client): Promise<void> {
  console.log(`[${new Date().toISOString()}] Checking RSS feeds...`);

  for (const [name, config] of Object.entries(rssConfig) as [string, RssSource][]) {
    try {
      await processRssFeed(name, config, client);
    } catch (error) {
      console.error(`[${name}] Failed to check RSS feed:`, error);
    }
  }
}

async function processRssFeed(
  name: string,
  config: RssSource,
  client: Client
): Promise<void> {
  const feed = await parser.parseURL(config.rss);

  if (!processedItems.has(name)) {
    processedItems.set(name, new Set());
  }

  const processed = processedItems.get(name)!;

  // 현재 RSS 피드에 있는 항목들의 ID 수집
  const currentFeedIds = new Set(
    feed.items.map((item) => item.link || item.guid || "").filter(Boolean)
  );

  // RSS 피드에서 사라진 항목들을 처리된 목록에서 제거
  const removedItems: string[] = [];
  for (const itemId of processed) {
    if (!currentFeedIds.has(itemId)) {
      removedItems.push(itemId);
    }
  }

  if (removedItems.length > 0) {
    for (const itemId of removedItems) {
      processed.delete(itemId);
    }
    console.log(`[${name}] Cleaned up ${removedItems.length} old items from processed list`);
    saveCurrentState();
  }

  for (const item of feed.items) {
    const itemId = item.link || item.guid || "";

    if (!itemId || processed.has(itemId)) continue;

    // 카테고리 필터링
    if (shouldFilterByCategory(config, item)) {
      console.log(
        `[${name}] Filtered by category: ${item.title} (${item.categories?.join(", ")})`
      );
      processed.add(itemId);
      continue;
    }

    console.log(`[${name}] New item found: ${item.title}`);

    await postToForum(config, item, client);
    processed.add(itemId);
    saveCurrentState();
  }
}

function shouldFilterByCategory(config: RssSource, item: Parser.Item): boolean {
  if (!config.category_filter?.length) return false;

  const categories = item.categories || [];
  return categories.some((cat) => config.category_filter!.includes(cat));
}

async function postToForum(
  config: RssSource,
  item: Parser.Item,
  client: Client
): Promise<void> {
  const startTime = performance.now();
  
  try {
    const channel = await client.channels.fetch(config.channel);

    if (!channel || !(channel instanceof ForumChannel)) {
      console.error(`Channel ${config.channel} not found or is not a forum channel.`);
      return;
    }

    const title = (item.title || "Untitled").slice(0, 100);
    const content = buildContent(config, item);

    await channel.threads.create({
      name: title,
      autoArchiveDuration: ThreadAutoArchiveDuration.OneWeek,
      message: { content },
      appliedTags: [config.tag],
    });

    const endTime = performance.now();
    console.log(`[Forum] Post completed: ${title} (${(endTime - startTime).toFixed(2)}ms)`);
  } catch (error) {
    const endTime = performance.now();
    console.error(`[Forum] Post failed (${(endTime - startTime).toFixed(2)}ms):`, error);
  }
}

function buildContent(config: RssSource, item: Parser.Item): string {
  const parts: string[] = [];

  if (item.link) {
    parts.push(`# ${config.emoji} | [${item.title}](<${item.link}>)`);
  }

  if (item.contentSnippet) {
    parts.push(`\n${item.content}\n`);
  }

  if (item.pubDate) {
    const timestamp = Math.floor(new Date(item.pubDate).getTime() / 1000);
    parts.push(`\n-# 🕐 <t:${timestamp}:f>`);
  }

  return parts.join("\n") || item.link || "No content available.";
}
