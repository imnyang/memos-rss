import { ForumChannel, ThreadAutoArchiveDuration, Client } from "discord.js";
import type Parser from "rss-parser";
import TurndownService from "turndown";
import { parser } from "./config";
import { processedItems, saveCurrentState } from "./storage";
import type { RssSource } from "../type/types";

export async function checkRssFeeds(client: Client): Promise<void> {
  console.log(`[${new Date().toISOString()}] Checking RSS feeds...`);

  let rssConfig: Record<string, RssSource>;
  try {
    const configFile = Bun.file("./rss.toml");
    const content = await configFile.text();
    rssConfig = Bun.TOML.parse(content) as Record<string, RssSource>;
  } catch (error) {
    console.error("Failed to load rss.toml:", error);
    return;
  }

  for (const [name, config] of Object.entries(rssConfig)) {
    try {
      await processRssFeed(name, config, client);
    } catch (error) {
      console.error(`[${name}] Failed to check RSS feed:`, error);
    }
  }
}

function getValueByPath(obj: any, path: string | undefined): any {
  if (!path) return undefined;
  if (obj[path] !== undefined) return obj[path];
  return path.split(".").reduce((acc, part) => acc && acc[part], obj);
}

function getFieldValue(
  item: Parser.Item,
  config: RssSource,
  field: keyof NonNullable<RssSource["setup"]>
): any {
  const path = config.setup?.[field];
  const itemany = item as any;
  if (path) {
    return getValueByPath(itemany, path);
  }

  switch (field) {
    case "title":
      return item.title;
    case "link":
      return item.link;
    case "content":
      return item.content || item.contentSnippet;
    case "pubDate":
      return item.pubDate;
    case "author":
      return (
        itemany.creator ||
        (typeof itemany.author === "string"
          ? itemany.author
          : itemany.author?.name)
      );
    default:
      return undefined;
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
    feed.items
      .map((item) => getFieldValue(item, config, "link") || item.guid || "")
      .filter(Boolean)
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
    console.log(
      `[${name}] Cleaned up ${removedItems.length} old items from processed list`
    );
    saveCurrentState();
  }

  for (const item of feed.items) {
    const itemId = getFieldValue(item, config, "link") || item.guid || "";

    if (!itemId || processed.has(itemId)) continue;

    // 카테고리 필터링
    if (shouldFilterByCategory(config, item)) {
      const title = getFieldValue(item, config, "title") || "Untitled";
      console.log(
        `[${name}] Filtered by category: ${title} (${item.categories?.join(
          ", "
        )})`
      );
      processed.add(itemId);
      continue;
    }

    const title = getFieldValue(item, config, "title") || "Untitled";
    console.log(`[${name}] New item found: ${title}`);

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
      console.error(
        `Channel ${config.channel} not found or is not a forum channel.`
      );
      return;
    }

    const title = (getFieldValue(item, config, "title") || "Untitled").slice(
      0,
      100
    );
    const content = buildContent(config, item);

    await channel.threads.create({
      name: title,
      autoArchiveDuration: ThreadAutoArchiveDuration.OneWeek,
      message: { content },
      appliedTags: [config.tag],
    });

    const endTime = performance.now();
    console.log(
      `[Forum] Post completed: ${title} (${(endTime - startTime).toFixed(2)}ms)`
    );
  } catch (error) {
    const endTime = performance.now();
    console.error(
      `[Forum] Post failed (${(endTime - startTime).toFixed(2)}ms):`,
      error
    );
  }
}

function buildContent(config: RssSource, item: Parser.Item): string {
  const parts: string[] = [];
  const turndownService = new TurndownService();

  const title = getFieldValue(item, config, "title") || "Untitled";
  const link = getFieldValue(item, config, "link");
  const content = getFieldValue(item, config, "content");
  const author = getFieldValue(item, config, "author");
  const authorLink = getFieldValue(item, config, "authorLink");
  const pubDate = getFieldValue(item, config, "pubDate");

  if (link) {
    parts.push(`# ${config.emoji} | [${title}](<${link}>)`);
  } else {
    parts.push(`# ${config.emoji} | ${title}`);
  }

  if (author) {
    if (authorLink) {
      parts.push(`-# 🖊️ [${author}](<${authorLink}>)`);
    } else {
      parts.push(`-# 🖊️ ${author}`);
    }
  }

  if (content) {
    const markdown = turndownService.turndown(content);
    parts.push(`\n${markdown.trim()}\n`);
  }

  if (pubDate) {
    const timestamp = Math.floor(new Date(pubDate).getTime() / 1000);
    if (!isNaN(timestamp)) {
      parts.push(`\n-# 🕐 <t:${timestamp}:f>`);
    }
  }

  return parts.join("\n") || link || "No content available.";
}
