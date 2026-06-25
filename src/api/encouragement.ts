import { invoke } from "@tauri-apps/api/core";
import type { Encouragement, AddEncouragementInput, StreakInfo } from "@/types";

/** 列出所有鼓励语 */
export async function listEncouragements(): Promise<Encouragement[]> {
  return invoke("list_encouragements");
}

/** 添加自定义鼓励语 */
export async function addEncouragement(
  input: AddEncouragementInput,
): Promise<Encouragement> {
  return invoke("add_encouragement", { input });
}

/** 删除鼓励语（预设不允许删除） */
export async function deleteEncouragement(id: string): Promise<void> {
  return invoke("delete_encouragement", { id });
}

/** 随机抽取一句鼓励语 */
export async function randomEncouragement(): Promise<Encouragement | null> {
  return invoke("random_encouragement");
}

/** 根据连续天数智能选择鼓励语等级（1天普通/3天进阶/7天高亮） */
export async function randomEncouragementByStreak(
  streak: number,
): Promise<Encouragement | null> {
  return invoke("random_encouragement_by_streak", { streak });
}

/** 抽取庆祝鼓励语（全部目标完成时使用） */
export async function randomCelebrationEncouragement(): Promise<Encouragement | null> {
  return invoke("random_celebration_encouragement");
}

/** 获取连续完成天数统计 */
export async function getStreak(): Promise<StreakInfo> {
  return invoke("get_streak");
}
