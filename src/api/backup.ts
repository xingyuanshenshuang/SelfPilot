import { invoke } from "@tauri-apps/api/core";
import type { ImportInput, ImportResult } from "@/types";

/** 导出全部数据为 JSON 字符串 */
export async function exportData(): Promise<string> {
  return invoke("export_data");
}

/** 导入数据 */
export async function importData(input: ImportInput): Promise<ImportResult> {
  return invoke("import_data", { input });
}
