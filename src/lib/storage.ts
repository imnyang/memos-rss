import { readFileSync, writeFileSync, existsSync } from "fs";
import { PROCESSED_FILE } from "./config";

export function loadProcessedItems(): Record<string, string[]> {
  if (existsSync(PROCESSED_FILE)) {
    try {
      return JSON.parse(readFileSync(PROCESSED_FILE, "utf-8"));
    } catch (error) {
      console.error("처리된 항목 파일 읽기 실패:", error);
    }
  }
  return {};
}

export function saveProcessedItems(items: Record<string, string[]>): void {
  try {
    writeFileSync(PROCESSED_FILE, JSON.stringify(items, null, 2));
  } catch (error) {
    console.error("처리된 항목 파일 저장 실패:", error);
  }
}

// 처리된 항목 Map
const processedData = loadProcessedItems();
export const processedItems = new Map<string, Set<string>>(
  Object.entries(processedData).map(([k, v]) => [k, new Set(v)])
);

export function saveCurrentState(): void {
  saveProcessedItems(
    Object.fromEntries(
      Array.from(processedItems.entries()).map(([k, v]) => [k, Array.from(v)])
    )
  );
}
