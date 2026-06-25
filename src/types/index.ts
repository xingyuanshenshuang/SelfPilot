/// 全局 TypeScript 类型定义（与 Rust 侧 models.rs 对应）

export interface Goal {
  id: string;
  name: string;
  deadline: string | null;
  total_qty: number;
  unit: string;
  created_at: string;
}

export interface CreateGoalInput {
  name: string;
  deadline?: string | null;
  total_qty?: number;
  unit?: string;
}

export interface Task {
  id: string;
  goal_id: string;
  stage_id: string | null;
  parent_id: string | null;
  path: string;
  name: string;
  plan_date: string | null;
  plan_qty: number;
  actual_qty: number;
  unit: string;
  status: TaskStatus;
  is_manual: number;
  source: "auto" | "manual";
  sort_order: number;
  created_at: string;
}

export interface CreateTaskInput {
  goal_id: string;
  stage_id?: string | null;
  name: string;
  plan_date?: string | null;
  plan_qty?: number;
  unit?: string;
}

export interface CompleteTaskInput {
  task_id: string;
  actual_qty: number;
}

export interface ProgressInfo {
  id: string;
  name: string;
  total_plan: number;
  total_actual: number;
  /** 完成百分比 0.0 ~ 1.0 */
  percentage: number;
}

export interface TodayTask {
  id: string;
  goal_id: string;
  goal_name: string;
  stage_id: string | null;
  name: string;
  plan_date: string | null;
  plan_qty: number;
  actual_qty: number;
  unit: string;
  status: TaskStatus;
  source: "auto" | "manual";
}

export type TaskStatus = "pending" | "partial" | "done" | "skipped";

/** 任务状态对应的显示信息 */
export const STATUS_META: Record<
  TaskStatus,
  { icon: string; label: string; color: string }
> = {
  pending: {
    icon: "mdi:checkbox-blank-outline",
    label: "未完成",
    color: "#909399",
  },
  partial: {
    icon: "mdi:checkbox-intermediate",
    label: "部分完成",
    color: "#e6a23c",
  },
  done: { icon: "mdi:check-circle", label: "已完成", color: "#67c23a" },
  skipped: {
    icon: "mdi:skip-next-circle-outline",
    label: "已跳过",
    color: "#909399",
  },
};
