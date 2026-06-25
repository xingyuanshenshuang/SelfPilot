import { invoke } from "@tauri-apps/api/core";
import type {
  Stage,
  StageWithProgress,
  CreateStageInput,
  UpdateStageInput,
  DeleteStageInput,
} from "@/types";

export async function createStage(input: CreateStageInput): Promise<Stage> {
  return invoke("create_stage", { input });
}

export async function listStages(goalId: string): Promise<StageWithProgress[]> {
  return invoke("list_stages", { goalId });
}

export async function updateStage(input: UpdateStageInput): Promise<Stage> {
  return invoke("update_stage", { input });
}

export async function deleteStage(input: DeleteStageInput): Promise<void> {
  return invoke("delete_stage", { input });
}
