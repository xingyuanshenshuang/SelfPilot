<script setup lang="ts">
import { onMounted, ref, reactive } from "vue";
import {
  NCard,
  NButton,
  NSpace,
  NInput,
  NDatePicker,
  NSelect,
  NInputNumber,
  NForm,
  NFormItem,
  NModal,
  NEmpty,
  NTag,
  NPopconfirm,
  useMessage,
} from "naive-ui";
import { Icon } from "@iconify/vue";
import { useGoalStore } from "@/stores/goalStore";
import * as taskApi from "@/api/task";
import type { Goal, Task } from "@/types";
import { STATUS_META } from "@/types";
import { format, parseISO, differenceInCalendarDays } from "date-fns";

const goalStore = useGoalStore();
const message = useMessage();

const showCreate = ref(false);
const form = reactive({
  name: "",
  deadline: null as number | null,
  total_qty: 10,
  unit: "个",
});

const unitOptions = [
  { label: "个", value: "个" },
  { label: "页", value: "页" },
  { label: "小时", value: "小时" },
];

const expandedGoals = ref<Set<string>>(new Set());
const tasksByGoal = ref<Record<string, Task[]>>({});

onMounted(async () => {
  await goalStore.fetchGoals();
  await goalStore.fetchProgresses();
});

async function handleCreate() {
  if (!form.name.trim()) {
    message.warning("请输入目标名称");
    return;
  }
  const deadline = form.deadline
    ? format(new Date(form.deadline), "yyyy-MM-dd")
    : null;
  try {
    await goalStore.createGoal({
      name: form.name,
      deadline,
      total_qty: form.total_qty,
      unit: form.unit,
    });
    message.success("目标创建成功");
    showCreate.value = false;
    form.name = "";
    form.deadline = null;
    form.total_qty = 10;
    form.unit = "个";
  } catch (e) {
    message.error(String(e));
  }
}

async function handleAutoSplit(goal: Goal) {
  try {
    const tasks = await goalStore.autoSplit(goal.id);
    tasksByGoal.value[goal.id] = tasks;
    expandedGoals.value.add(goal.id);
    message.success(`已拆解为 ${tasks.length} 个每日任务`);
  } catch (e) {
    message.error(String(e));
  }
}

async function toggleGoal(goalId: string) {
  if (expandedGoals.value.has(goalId)) {
    expandedGoals.value.delete(goalId);
  } else {
    expandedGoals.value.add(goalId);
    if (!tasksByGoal.value[goalId]) {
      tasksByGoal.value[goalId] = await taskApi.listTasksByGoal(goalId);
    }
  }
}

async function handleDeleteGoal(goal: Goal) {
  try {
    await goalStore.deleteGoal(goal.id);
    delete tasksByGoal.value[goal.id];
    expandedGoals.value.delete(goal.id);
    message.success("已删除目标");
  } catch (e) {
    message.error(String(e));
  }
}

function getDaysLeft(deadline: string | null): string {
  if (!deadline) return "未设置截止日期";
  const days = differenceInCalendarDays(parseISO(deadline), new Date());
  if (days < 0) return `已逾期 ${-days} 天`;
  if (days === 0) return "今天截止";
  return `剩余 ${days} 天`;
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-semibold flex items-center gap-2">
        <Icon icon="mdi:file-tree-outline" width="22" class="text-brand-500" />
        目标树
      </h2>
      <NButton type="primary" @click="showCreate = true">
        <template #icon><Icon icon="mdi:plus" /></template>
        创建目标
      </NButton>
    </div>

    <!-- 目标列表 -->
    <div v-if="goalStore.goals.length > 0" class="space-y-2">
      <NCard v-for="goal in goalStore.goals" :key="goal.id" size="small">
        <!-- 目标头部 -->
        <div
          class="flex items-center gap-3 cursor-pointer"
          @click="toggleGoal(goal.id)"
        >
          <Icon
            :icon="
              expandedGoals.has(goal.id)
                ? 'mdi:chevron-down'
                : 'mdi:chevron-right'
            "
            width="20"
            class="text-gray-400"
          />
          <Icon icon="mdi:target" width="20" class="text-brand-500" />
          <div class="flex-1">
            <div class="font-medium">{{ goal.name }}</div>
            <div class="text-xs text-gray-500 flex items-center gap-3 mt-0.5">
              <span
                :class="{
                  'text-red-500': getDaysLeft(goal.deadline).includes('逾期'),
                }"
              >
                {{ getDaysLeft(goal.deadline) }}
              </span>
              <span>总量：{{ goal.total_qty }}{{ goal.unit }}</span>
              <span>
                进度：{{
                  Math.round(
                    (goalStore.getProgress(goal.id)?.percentage ?? 0) * 100,
                  )
                }}%
              </span>
            </div>
          </div>
          <NSpace :size="4" @click.stop>
            <NButton
              size="tiny"
              type="primary"
              ghost
              :disabled="!goal.deadline || goal.total_qty <= 0"
              @click="handleAutoSplit(goal)"
            >
              <template #icon><Icon icon="mdi:auto-fix" /></template>
              自动拆解
            </NButton>
            <NPopconfirm @positive-click="handleDeleteGoal(goal)">
              <template #trigger>
                <NButton size="tiny" quaternary type="error">
                  <Icon icon="mdi:delete" />
                </NButton>
              </template>
              确定删除目标"{{ goal.name }}"？所有关联任务将一并删除。
            </NPopconfirm>
          </NSpace>
        </div>

        <!-- 任务列表 -->
        <div v-if="expandedGoals.has(goal.id)" class="mt-3 ml-8 space-y-1">
          <div
            v-if="(tasksByGoal[goal.id] || []).length === 0"
            class="text-sm text-gray-400 py-2"
          >
            暂无任务，点击"自动拆解"生成每日任务
          </div>
          <div
            v-for="task in tasksByGoal[goal.id] || []"
            :key="task.id"
            class="flex items-center gap-2 px-3 py-1.5 rounded hover:bg-gray-50 text-sm"
          >
            <Icon
              :icon="STATUS_META[task.status].icon"
              :color="STATUS_META[task.status].color"
              width="16"
            />
            <span
              class="flex-1 truncate"
              :class="{ 'line-through text-gray-400': task.status === 'done' }"
            >
              {{ task.name }}
            </span>
            <span class="text-xs text-gray-500">{{ task.plan_date }}</span>
            <NTag
              size="tiny"
              :bordered="false"
              :type="task.source === 'auto' ? 'info' : 'warning'"
            >
              {{ task.source === "auto" ? "自动" : "手动" }}
            </NTag>
            <span class="text-xs text-gray-500">
              {{ task.actual_qty }}/{{ task.plan_qty }}{{ task.unit }}
            </span>
          </div>
        </div>
      </NCard>
    </div>

    <NEmpty v-else description="还没有目标，点击右上角创建第一个目标吧" />

    <!-- 创建目标弹窗 -->
    <NModal
      v-model:show="showCreate"
      preset="card"
      title="创建目标"
      style="width: 480px"
    >
      <NForm label-placement="top">
        <NFormItem label="目标名称" required>
          <NInput
            v-model:value="form.name"
            placeholder="如：完成 Vue 视频学习"
          />
        </NFormItem>
        <NFormItem label="截止日期">
          <NDatePicker
            v-model:value="form.deadline"
            type="date"
            clearable
            :is-date-disabled="(ts: number) => ts < Date.now() - 86400000"
          />
        </NFormItem>
        <NSpace>
          <NFormItem label="总量">
            <NInputNumber v-model:value="form.total_qty" :min="0" />
          </NFormItem>
          <NFormItem label="单位">
            <NSelect
              v-model:value="form.unit"
              :options="unitOptions"
              style="width: 100px"
            />
          </NFormItem>
        </NSpace>
      </NForm>
      <template #footer>
        <NSpace justify="end">
          <NButton @click="showCreate = false">取消</NButton>
          <NButton type="primary" @click="handleCreate">创建</NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>
