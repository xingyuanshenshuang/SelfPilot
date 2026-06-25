import { invoke } from "@tauri-apps/api/core";
import type { ProgressInfo } from "@/types";

export async function getGoalProgress(goalId: string): Promise<ProgressInfo> {
  return invoke("get_goal_progress", { goalId });
}

export async function getAllGoalsProgress(): Promise<ProgressInfo[]> {
  return invoke("get_all_goals_progress");
}
