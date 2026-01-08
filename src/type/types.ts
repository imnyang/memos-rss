export interface RssSource {
  channel: string;
  tag: string;
  emoji: string;
  rss: string;
  category_filter?: string[];
  setup?: {
    title?: string;
    link?: string;
    content?: string;
    author?: string;
    authorLink?: string;
    pubDate?: string;
  };
}
