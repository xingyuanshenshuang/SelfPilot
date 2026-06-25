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
  NDataTable,
  NProgress,
  useMessage,
  useDialog,
} from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { Icon } from "@iconify/vue";
import { useGoalStore } from "@/stores/goalStore";
import * as taskApi from "@/api/task";
import * as stageApi from "@/api/stage";
import * as goalApi from "@/api/goal";
import type { Goal, Task, StageWithProgress, ReplanPreview } from "@/types";
import { STATUS_META } from "@/types";
import { format, parseISO, differenceInCalendarDays } from "date-fns";

const goalStore = useGoalStore();
const message = useMessage();
const dialog = useDialog();

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
const expandedStages = ref<Set<string>>(new Set());
const tasksByGoal = ref<Record<string, Task[]>>({});
const stagesByGoal = ref<Record<string, StageWithProgress[]>>({});

// 阶段相关状态
const showAddStage = ref<string | null>(null);
const newStageName = ref("");

// 重新规划相关状态
const showReplanModal = ref(false);
const replanPreview = ref<ReplanPreview | null>(null);
const replanGoalId = ref("");

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
    await loadGoalData(goalId);
  }
}

async function loadGoalData(goalId: string) {
  const [tasks, stages] = await Promise.all([
    taskApi.listTasksByGoal(goalId),
    stageApi.listStages(goalId),
  ]);
  tasksByGoal.value[goalId] = tasks;
  stagesByGoal.value[goalId] = stages;
}

function toggleStage(stageId: string) {
  if (expandedStages.value.has(stageId)) {
    expandedStages.value.delete(stageId);
  } else {
    expandedStages.value.add(stageId);
  }
}

async function handleAddStage(goal: Goal) {
  if (!newStageName.value.trim()) {
    message.warning("请输入阶段名称");
    return;
  }
  try {
    await stageApi.createStage({
      goal_id: goal.id,
      name: newStageName.value,
    });
    message.success("阶段创建成功");
    newStageName.value = "";
    showAddStage.value = null;
    await loadGoalData(goal.id);
  } catch (e) {
    message.error(String(e));
  }
}

async function handleDeleteStage(stage: StageWithProgress, goalId: string) {
  dialog.warning({
    title: "删除阶段",
    content: `阶段"${stage.name}"下有 ${stage.task_count} 个任务。请选择处理方式：`,
    positiveText: "子任务转独立",
    negativeText: "级联删除",
    onPositiveClick: async () => {
      try {
        await stageApi.deleteStage({ id: stage.id, mode: "detach" });
        message.success("阶段已删除，子任务转为独立任务");
        await loadGoalData(goalId);
      } catch (e) {
        message.error(String(e));
      }
    },
    onNegativeClick: async () => {
      try {
        await stageApi.deleteStage({ id: stage.id, mode: "cascade" });
        message.success("阶段及子任务已删除");
        await loadGoalData(goalId);
      } catch (e) {
        message.error(String(e));
      }
    },
  });
}

async function handleDeleteGoal(goal: Goal) {
  try {
    await goalStore.deleteGoal(goal.id);
    delete tasksByGoal.value[goal.id];
    delete stagesByGoal.value[goal.id];
    expandedGoals.value.delete(goal.id);
    message.success("已删除目标");
  } catch (e) {
    message.error(String(e));
  }
}

/** 获取阶段下的任务 */
function getStageTasks(goalId: string, stageId: string): Task[] {
  return (tasksByGoal.value[goalId] || []).filter(
    (t) => t.stage_id === stageId,
  );
}

/** 获取不属于任何阶段的独立任务 */
function getStandaloneTasks(goalId: string): Task[] {
  return (tasksByGoal.value[goalId] || []).filter((t) => !t.stage_id);
}

/** 重新规划预览 */
async function handleReplanPreview(goal: Goal) {
  try {
    replanPreview.value = await goalApi.replanPreview(goal.id);
    replanGoalId.value = goal.id;
    showReplanModal.value = true;
  } catch (e) {
    message.error(String(e));
  }
}

/** 执行重新规划 */
async function handleReplanConfirm() {
  try {
    const result = await goalApi.replanGoal(replanGoalId.value);
    message.success(
      `已重新规划：更新 ${result.updated_count} 个任务，保留 ${result.retained_count} 个手动任务`,
    );
    showReplanModal.value = false;
    replanPreview.value = null;
    await loadGoalData(replanGoalId.value);
    await goalStore.fetchProgresses();
  } catch (e) {
    message.error(String(e));
  }
}

/** 重新规划预览表格列 */
const replanColumns: DataTableColumns<ReplanPreview["items"][0]> = [
  {
    title: "日期",
    key: "plan_date",
    width: 110,
  },
  {
    title: "任务名",
    key: "name",
    ellipsis: { tooltip: true },
  },
  {
    title: "原计划",
    key: "old_plan_qty",
    width: 90,
    render: (row) => `${row.old_plan_qty}`,
  },
  {
    title: "新计划",
    key: "new_plan_qty",
    width: 90,
    render: (row) =>
      row.retained
        ? `${row.new_plan_qty} (保留)`
        : Math.abs(row.new_plan_qty - row.old_plan_qty) > 0.01
          ? `${row.new_plan_qty}`
          : `${row.new_plan_qty}`,
  },
];

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
            <NButton
              size="tiny"
              type="warning"
              ghost
              :disabled="!goal.deadline"
              @click="handleReplanPreview(goal)"
            >
              <template #icon><Icon icon="mdi:refresh-circle" /></template>
              重新规划
            </NButton>
            <NButton size="tiny" quaternary @click="showAddStage = goal.id">
              <template #icon><Icon icon="mdi:folder-plus-outline" /></template>
              添加阶段
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

        <!-- 添加阶段输入框 -->
        <div
          v-if="showAddStage === goal.id"
          class="mt-2 ml-8 flex gap-2"
          @click.stop
        >
          <NInput
            v-model:value="newStageName"
            size="small"
            placeholder="阶段名称，如：第一阶段"
            @keyup.enter="handleAddStage(goal)"
          />
          <NButton size="small" type="primary" @click="handleAddStage(goal)">
            确认
          </NButton>
          <NButton size="small" @click="showAddStage = null">取消</NButton>
        </div>

        <!-- 目标展开内容：阶段 + 独立任务 -->
        <div v-if="expandedGoals.has(goal.id)" class="mt-3 ml-8 space-y-2">
          <!-- 阶段列表 -->
          <div
            v-for="stage in stagesByGoal[goal.id] || []"
            :key="stage.id"
            class="border border-gray-100 rounded-lg"
          >
            <!-- 阶段头部 -->
            <div
              class="flex items-center gap-2 px-3 py-1.5 cursor-pointer hover:bg-gray-50 rounded-lg"
              @click="toggleStage(stage.id)"
            >
              <Icon
                :icon="
                  expandedStages.has(stage.id)
                    ? 'mdi:chevron-down'
                    : 'mdi:chevron-right'
                "
                width="16"
                class="text-gray-400"
              />
              <Icon
                icon="mdi:folder-outline"
                width="16"
                class="text-blue-500"
              />
              <span class="flex-1 text-sm font-medium">{{ stage.name }}</span>
              <NProgress
                type="line"
                :percentage="Math.round(stage.percentage * 100)"
                :show-indicator="false"
                :height="4"
                style="width: 80px"
              />
              <span class="text-xs text-gray-500">
                {{ Math.round(stage.percentage * 100) }}% ({{
                  stage.task_count
                }}任务)
              </span>
              <NPopconfirm @positive-click="handleDeleteStage(stage, goal.id)">
                <template #trigger>
                  <NButton size="tiny" quaternary type="error" @click.stop>
                    <Icon icon="mdi:close" width="14" />
                  </NButton>
                </template>
                确定删除阶段"{{ stage.name }}"？
              </NPopconfirm>
            </div>
            <!-- 阶段下任务 -->
            <div
              v-if="expandedStages.has(stage.id)"
              class="ml-6 pb-1 space-y-0.5"
            >
              <div
                v-for="task in getStageTasks(goal.id, stage.id)"
                :key="task.id"
                class="flex items-center gap-2 px-3 py-1 rounded hover:bg-gray-50 text-sm"
              >
                <Icon
                  :icon="STATUS_META[task.status].icon"
                  :color="STATUS_META[task.status].color"
                  width="14"
                />
                <span
                  class="flex-1 truncate"
                  :class="{
                    'line-through text-gray-400': task.status === 'done',
                  }"
                >
                  {{ task.name }}
                </span>
                <span class="text-xs text-gray-500">{{ task.plan_date }}</span>
                <span class="text-xs text-gray-500">
                  {{ task.actual_qty }}/{{ task.plan_qty }}{{ task.unit }}
                </span>
                <NTag
                  v-if="task.is_manual"
                  size="tiny"
                  :bordered="false"
                  type="warning"
                >
                  手动
                </NTag>
              </div>
              <div
                v-if="getStageTasks(goal.id, stage.id).length === 0"
                class="text-xs text-gray-400 py-1 px-3"
              >
                暂无任务
              </div>
            </div>
          </div>

          <!-- 独立任务（不属于任何阶段） -->
          <div
            v-if="getStandaloneTasks(goal.id).length > 0"
            class="space-y-0.5"
          >
            <div
              v-if="(stagesByGoal[goal.id] || []).length > 0"
              class="text-xs text-gray-400 px-3 py-1"
            >
              独立任务
            </div>
            <div
              v-for="task in getStandaloneTasks(goal.id)"
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
                :class="{
                  'line-through text-gray-400': task.status === 'done',
                }"
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
              <NTag
                v-if="task.is_manual"
                size="tiny"
                :bordered="false"
                type="warning"
              >
                手动
              </NTag>
            </div>
          </div>

          <!-- 空状态 -->
          <div
            v-if="
              (tasksByGoal[goal.id] || []).length === 0 &&
              (stagesByGoal[goal.id] || []).length === 0
            "
            class="text-sm text-gray-400 py-2"
          >
            暂无任务，点击"自动拆解"生成每日任务
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

    <!-- 重新规划预览弹窗 -->
    <NModal
      v-model:show="showReplanModal"
      preset="card"
      title="重新规划预览"
      style="width: 720px"
    >
      <div v-if="replanPreview" class="space-y-3">
        <div class="text-sm text-gray-600 space-y-1">
          <div>
            目标：<strong>{{ replanPreview.goal_name }}</strong>
          </div>
          <div>
            剩余天数：<strong>{{ replanPreview.remaining_days }} 天</strong> ｜
            剩余总量：<strong>{{ replanPreview.remaining_qty }}</strong>
          </div>
          <div>
            每日基础量：{{ replanPreview.daily_base }} ｜ 余数分前
            {{ replanPreview.remainder }} 天
          </div>
        </div>
        <div class="text-xs text-orange-500 bg-orange-50 px-3 py-2 rounded">
          <Icon icon="mdi:information" class="inline-block mr-1" />
          已跳过的任务不参与重新规划；手动修改过的任务（标记"手动"）将保留原计划数量
        </div>
        <NDataTable
          :columns="replanColumns"
          :data="replanPreview.items"
          :max-height="320"
          size="small"
          :bordered="false"
        />
      </div>
      <template #footer>
        <NSpace justify="end">
          <NButton @click="showReplanModal = false">取消</NButton>
          <NButton type="warning" @click="handleReplanConfirm">
            确认重新规划
          </NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>
