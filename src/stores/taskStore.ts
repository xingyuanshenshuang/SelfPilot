import { defineStore } from "pinia";
import { ref } from "vue";
import * as taskApi from "@/api/task";
import type { TodayTask, CompleteTaskInput } from "@/types";

export const useTaskStore = defineStore("task", () => {
  const todayTasks = ref<TodayTask[]>([]);
  const overdueTasks = ref<TodayTask[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try {
      const [today, overdue] = await Promise.all([
        taskApi.listTodayTasks(),
        taskApi.listOverdueTasks(),
      ]);
      todayTasks.value = today;
      overdueTasks.value = overdue;
    } finally {
      loading.value = false;
    }
  }

  async function completeTask(input: CompleteTaskInput) {
    const updated = await taskApi.completeTask(input);
    await fetchAll();
    return updated;
  }

  async function skipTask(taskId: string) {
    await taskApi.skipTask(taskId);
    await fetchAll();
  }

  return {
    todayTasks,
    overdueTasks,
    loading,
    fetchAll,
    completeTask,
    skipTask,
  };
});
