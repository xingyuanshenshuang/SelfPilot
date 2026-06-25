import { invoke } from "@tauri-apps/api/core";
import type { SetSettingInput, Setting } from "@/types";

/** 获取所有设置项 */
export async function getAllSettings(): Promise<Setting[]> {
  return invoke("get_all_settings");
}

/** 获取单个设置项 */
export async function getSetting(key: string): Promise<string | null> {
  return invoke("get_setting", { key });
}

/** 设置某个值（upsert） */
export async function setSetting(input: SetSettingInput): Promise<void> {
  return invoke("set_setting", { input });
}
