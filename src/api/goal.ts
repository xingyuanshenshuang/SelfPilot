import { invoke } from "@tauri-apps/api/core";
import type {
  Goal,
  CreateGoalInput,
  Task,
  ReplanPreview,
  ReplanResult,
} from "@/types";

export async function createGoal(input: CreateGoalInput): Promise<Goal> {
  return invoke("create_goal", { input });
}

export async function listGoals(): Promise<Goal[]> {
  return invoke("list_goals");
}

export async function getGoal(id: string): Promise<Goal> {
  return invoke("get_goal", { id });
}

export async function deleteGoal(id: string): Promise<void> {
  return invoke("delete_goal", { id });
}

export async function autoSplit(goalId: string): Promise<Task[]> {
  return invoke("auto_split", { goalId });
}

/** 重新规划预览 */
export async function replanPreview(goalId: string): Promise<ReplanPreview> {
  return invoke("replan_preview", { goalId });
}

/** 执行重新规划 */
export async function replanGoal(goalId: string): Promise<ReplanResult> {
  return invoke("replan_goal", { goalId });
}
