declare module "*.toml" {
  import { RssSource } from "./types";
  const value: Record<string, RssSource>;
  export default value;
}
