import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as encApi from "@/api/encouragement";
import type { Encouragement, StreakInfo, EncouragementLevel } from "@/types";

/** 鼓励语 Store - 管理鼓励语库和连续天数 */
export const useEncouragementStore = defineStore("encouragement", () => {
  const list = ref<Encouragement[]>([]);
  const streak = ref<StreakInfo>({
    current_streak: 0,
    longest_streak: 0,
    completed_today: false,
  });
  const loaded = ref(false);

  const presetList = computed(() =>
    list.value.filter((e) => e.category === "preset"),
  );
  const customList = computed(() =>
    list.value.filter((e) => e.category === "custom"),
  );

  /** 按等级分组 */
  const byLevel = computed(() => {
    const groups: Record<EncouragementLevel, Encouragement[]> = {
      normal: [],
      advanced: [],
      highlight: [],
      celebration: [],
    };
    for (const e of list.value) {
      if (groups[e.level]) {
        groups[e.level].push(e);
      }
    }
    return groups;
  });

  /** 加载鼓励语列表 */
  async function fetchAll() {
    try {
      list.value = await encApi.listEncouragements();
      loaded.value = true;
    } catch {
      list.value = [];
    }
  }

  /** 加载连续天数 */
  async function fetchStreak() {
    try {
      streak.value = await encApi.getStreak();
    } catch {
      // 忽略
    }
  }

  /** 添加自定义鼓励语 */
  async function add(text: string, level: EncouragementLevel = "normal") {
    const item = await encApi.addEncouragement({ text, level });
    list.value.push(item);
    return item;
  }

  /** 删除鼓励语 */
  async function remove(id: string) {
    await encApi.deleteEncouragement(id);
    list.value = list.value.filter((e) => e.id !== id);
  }

  /** 随机抽取一句鼓励语（全等级） */
  async function random(): Promise<Encouragement | null> {
    if (list.value.length === 0) {
      await fetchAll();
    }
    if (list.value.length === 0) return null;
    try {
      return await encApi.randomEncouragement();
    } catch {
      const idx = Math.floor(Math.random() * list.value.length);
      return list.value[idx] ?? null;
    }
  }

  /** 根据连续天数智能抽取鼓励语（Sprint 5 个性化规则） */
  async function randomByStreak(
    streakDays: number,
  ): Promise<Encouragement | null> {
    try {
      return await encApi.randomEncouragementByStreak(streakDays);
    } catch {
      // 后端不可用时降级为全等级随机
      return random();
    }
  }

  /** 抽取庆祝鼓励语（全部目标完成） */
  async function randomCelebration(): Promise<Encouragement | null> {
    try {
      return await encApi.randomCelebrationEncouragement();
    } catch {
      return random();
    }
  }

  return {
    list,
    streak,
    loaded,
    presetList,
    customList,
    byLevel,
    fetchAll,
    fetchStreak,
    add,
    remove,
    random,
    randomByStreak,
    randomCelebration,
  };
});
