<script setup lang="ts">
import { ref, computed } from "vue";
import {
  NButton,
  NInputNumber,
  NTag,
  NSpace,
  NModal,
  NForm,
  NFormItem,
  useMessage,
} from "naive-ui";
import { Icon } from "@iconify/vue";
import type { TodayTask } from "@/types";
import { STATUS_META } from "@/types";
import { useTaskStore } from "@/stores/taskStore";
import { useGoalStore } from "@/stores/goalStore";
import { randomEncouragement } from "@/constants/encouragements";
import * as taskApi from "@/api/task";

const props = defineProps<{
  task: TodayTask;
  overdue?: boolean;
}>();

const emit = defineEmits<{ (e: "completed"): void }>();

const taskStore = useTaskStore();
const goalStore = useGoalStore();
const message = useMessage();

const statusMeta = computed(() => STATUS_META[props.task.status]);
const isQtyTask = computed(() => props.task.plan_qty > 1);
/** 是否可补完成（已跳过或逾期的未完成任务） */
const canBackfill = computed(
  () => props.overdue || props.task.status === "skipped",
);

const showCompleteModal = ref(false);
const showBackfillModal = ref(false);
const actualQty = ref(0);
const backfillQty = ref(0);

const completionText = computed(() => {
  if (props.task.status === "done") return "已完成";
  if (props.task.status === "partial") {
    return `${props.task.actual_qty}/${props.task.plan_qty}${props.task.unit}`;
  }
  if (props.task.status === "skipped") return "已跳过（可补完成）";
  return `计划 ${props.task.plan_qty}${props.task.unit}`;
});

function openCompleteModal() {
  if (isQtyTask.value) {
    actualQty.value = props.task.actual_qty;
    showCompleteModal.value = true;
  } else {
    // 布尔型任务（plan_qty=1）直接完成
    doComplete(props.task.plan_qty);
  }
}

function openBackfillModal() {
  backfillQty.value = props.task.actual_qty;
  showBackfillModal.value = true;
}

async function doComplete(qty: number) {
  try {
    await taskStore.completeTask({
      task_id: props.task.id,
      actual_qty: qty,
    });
    await goalStore.fetchProgresses();
    message.success(randomEncouragement());
    emit("completed");
  } catch (e) {
    message.error(String(e));
  }
}

/** 补完成：只更新历史完成记录，不触发重新规划 */
async function doBackfill() {
  try {
    await taskApi.backfillTask({
      task_id: props.task.id,
      actual_qty: backfillQty.value,
    });
    await goalStore.fetchProgresses();
    message.success("已补完成");
    showBackfillModal.value = false;
    emit("completed");
  } catch (e) {
    message.error(String(e));
  }
}

function confirmComplete() {
  showCompleteModal.value = false;
  doComplete(actualQty.value);
}

async function handleSkip() {
  try {
    await taskStore.skipTask(props.task.id);
    await goalStore.fetchProgresses();
    message.info("已跳过");
  } catch (e) {
    message.error(String(e));
  }
}
</script>

<template>
  <div
    class="task-item flex items-center gap-3 px-3 py-2 rounded transition-colors hover:bg-gray-50"
    :class="{ 'bg-red-50': overdue }"
  >
    <Icon :icon="statusMeta.icon" :color="statusMeta.color" width="20" />
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        <span
          class="text-sm font-medium truncate"
          :class="{ 'line-through text-gray-400': task.status === 'done' }"
        >
          {{ task.name }}
        </span>
        <NTag size="tiny" :bordered="false" type="info">{{
          task.goal_name
        }}</NTag>
      </div>
      <div class="text-xs text-gray-500 mt-0.5">{{ completionText }}</div>
    </div>

    <NSpace
      v-if="task.status !== 'done' && task.status !== 'skipped'"
      :size="4"
    >
      <NButton size="tiny" type="primary" @click="openCompleteModal">
        <template #icon><Icon icon="mdi:check" /></template>
        完成
      </NButton>
      <NButton size="tiny" quaternary @click="handleSkip">
        <template #icon><Icon icon="mdi:skip-next" /></template>
        跳过
      </NButton>
    </NSpace>
    <!-- 补完成按钮（逾期/已跳过的任务） -->
    <NButton
      v-else-if="canBackfill && task.status !== 'done'"
      size="tiny"
      type="warning"
      ghost
      @click="openBackfillModal"
    >
      <template #icon><Icon icon="mdi:history" /></template>
      补完成
    </NButton>
    <NTag
      v-else-if="task.status === 'done'"
      size="tiny"
      type="success"
      :bordered="false"
    >
      已完成
    </NTag>
    <NTag v-else size="tiny" type="default" :bordered="false">已跳过</NTag>

    <!-- 数量型任务完成弹窗 -->
    <NModal
      v-model:show="showCompleteModal"
      preset="card"
      title="完成任务"
      style="width: 360px"
    >
      <NForm label-placement="top">
        <NFormItem :label="`实际完成量 (0 ~ ${task.plan_qty}${task.unit})`">
          <NInputNumber
            v-model:value="actualQty"
            :min="0"
            :max="task.plan_qty"
            :step="1"
            style="width: 100%"
          />
        </NFormItem>
      </NForm>
      <template #footer>
        <NSpace justify="end">
          <NButton @click="showCompleteModal = false">取消</NButton>
          <NButton type="primary" @click="confirmComplete">确认完成</NButton>
        </NSpace>
      </template>
    </NModal>

    <!-- 补完成弹窗（允许超过原计划数量） -->
    <NModal
      v-model:show="showBackfillModal"
      preset="card"
      title="补完成"
      style="width: 360px"
    >
      <NForm label-placement="top">
        <NFormItem
          :label="`补录实际完成量 (原计划 ${task.plan_qty}${task.unit})`"
        >
          <NInputNumber
            v-model:value="backfillQty"
            :min="0"
            :step="1"
            style="width: 100%"
          />
        </NFormItem>
        <div class="text-xs text-gray-500">
          补完成只更新历史完成记录，不会影响未来任务的计划数量
        </div>
      </NForm>
      <template #footer>
        <NSpace justify="end">
          <NButton @click="showBackfillModal = false">取消</NButton>
          <NButton type="warning" @click="doBackfill">确认补完成</NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>
