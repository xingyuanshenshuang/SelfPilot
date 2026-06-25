import { defineStore } from "pinia";
import { ref } from "vue";
import * as goalApi from "@/api/goal";
import * as progressApi from "@/api/progress";
import type { Goal, CreateGoalInput, ProgressInfo } from "@/types";

export const useGoalStore = defineStore("goal", () => {
  const goals = ref<Goal[]>([]);
  const progresses = ref<ProgressInfo[]>([]);
  const loading = ref(false);

  async function fetchGoals() {
    loading.value = true;
    try {
      goals.value = await goalApi.listGoals();
    } finally {
      loading.value = false;
    }
  }

  async function fetchProgresses() {
    progresses.value = await progressApi.getAllGoalsProgress();
  }

  async function createGoal(input: CreateGoalInput) {
    const goal = await goalApi.createGoal(input);
    goals.value.push(goal);
    return goal;
  }

  async function deleteGoal(id: string) {
    await goalApi.deleteGoal(id);
    goals.value = goals.value.filter((g) => g.id !== id);
    progresses.value = progresses.value.filter((p) => p.id !== id);
  }

  async function autoSplit(goalId: string) {
    const tasks = await goalApi.autoSplit(goalId);
    await fetchProgresses();
    return tasks;
  }

  function getProgress(goalId: string): ProgressInfo | undefined {
    return progresses.value.find((p) => p.id === goalId);
  }

  return {
    goals,
    progresses,
    loading,
    fetchGoals,
    fetchProgresses,
    createGoal,
    deleteGoal,
    autoSplit,
    getProgress,
  };
});
