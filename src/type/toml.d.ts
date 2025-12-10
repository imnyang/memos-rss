declare module "*.toml" {
  const value: Record<string, {
    channel: string;
    tag: string;
    emoji: string;
    body: string;
    rss: string;
    category_filter?: string[];
  }>;
  export default value;
}
