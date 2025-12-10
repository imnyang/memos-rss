export interface RssSource {
  channel: string;
  tag: string;
  emoji: string;
  body: string;
  rss: string;
  category_filter?: string[];
}
