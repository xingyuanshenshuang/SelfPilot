import { invoke } from '@tauri-apps/api/core'
import type { Task, CreateTaskInput, CompleteTaskInput, TodayTask } from '@/types'

export async function createTask(input: CreateTaskInput): Promise<Task> {
  return invoke('create_task', { input })
}

export async function completeTask(input: CompleteTaskInput): Promise<Task> {
  return invoke('complete_task', { input })
}

export async function skipTask(taskId: string): Promise<Task> {
  return invoke('skip_task', { taskId })
}

export async function listTodayTasks(): Promise<TodayTask[]> {
  return invoke('list_today_tasks')
}

export async function listOverdueTasks(): Promise<TodayTask[]> {
  return invoke('list_overdue_tasks')
}

export async function listTasksByGoal(goalId: string): Promise<Task[]> {
  return invoke('list_tasks_by_goal', { goalId })
}
