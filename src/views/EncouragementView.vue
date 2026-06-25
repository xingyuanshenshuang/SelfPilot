<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import {
  NCard,
  NButton,
  NSpace,
  NInput,
  NTag,
  NPopconfirm,
  NEmpty,
  NStatistic,
  NSelect,
  useMessage,
} from "naive-ui";
import { Icon } from "@iconify/vue";
import { useEncouragementStore } from "@/stores/encouragementStore";
import type { EncouragementLevel } from "@/types";

type TagType = "default" | "success" | "error" | "warning" | "info" | "primary";

const store = useEncouragementStore();
const message = useMessage();

const newText = ref("");
const newLevel = ref<EncouragementLevel>("normal");

const levelOptions = [
  { label: "普通（1天连续）", value: "normal" },
  { label: "进阶（3天连续）", value: "advanced" },
  { label: "高亮（7天连续）", value: "highlight" },
  { label: "庆祝（全部完成）", value: "celebration" },
];

/** 等级元信息 */
const LEVEL_META: Record<
  EncouragementLevel,
  {
    label: string;
    color: TagType;
    icon: string;
    desc: string;
    iconColor: string;
  }
> = {
  normal: {
    label: "普通",
    color: "info",
    icon: "mdi:emoticon-happy-outline",
    desc: "连续 1 天完成时抽取",
    iconColor: "text-blue-500",
  },
  advanced: {
    label: "进阶",
    color: "success",
    icon: "mdi:emoticon-cool-outline",
    desc: "连续 3 天完成时抽取",
    iconColor: "text-green-500",
  },
  highlight: {
    label: "高亮",
    color: "warning",
    icon: "mdi:emoticon-star-outline",
    desc: "连续 7 天完成时抽取",
    iconColor: "text-orange-500",
  },
  celebration: {
    label: "庆祝",
    color: "error",
    icon: "mdi:trophy-outline",
    desc: "全部目标完成时抽取",
    iconColor: "text-red-500",
  },
};

onMounted(async () => {
  await Promise.all([store.fetchAll(), store.fetchStreak()]);
});

async function handleAdd() {
  const text = newText.value.trim();
  if (!text) {
    message.warning("请输入鼓励语内容");
    return;
  }
  try {
    await store.add(text, newLevel.value);
    message.success("已添加");
    newText.value = "";
  } catch (e) {
    message.error(String(e));
  }
}

async function handleDelete(id: string) {
  try {
    await store.remove(id);
    message.success("已删除");
  } catch (e) {
    message.error(String(e));
  }
}

async function handleRefreshStreak() {
  await store.fetchStreak();
  message.success("已刷新连续天数");
}

/** 按等级分组的自定义鼓励语 */
const customByLevel = computed(() => {
  const groups: Record<EncouragementLevel, typeof store.customList> = {
    normal: [],
    advanced: [],
    highlight: [],
    celebration: [],
  };
  for (const e of store.customList) {
    if (groups[e.level]) {
      groups[e.level].push(e);
    }
  }
  return groups;
});

/** 按等级分组的预设鼓励语 */
const presetByLevel = computed(() => store.byLevel);
</script>

<template>
  <div class="space-y-4">
    <!-- 连续天数统计 -->
    <NCard :bordered="false">
      <template #header>
        <div class="flex items-center gap-2">
          <Icon icon="mdi:fire" width="22" class="text-orange-500" />
          <span>连续完成打卡</span>
        </div>
      </template>
      <template #header-extra>
        <NButton size="small" quaternary @click="handleRefreshStreak">
          <template #icon><Icon icon="mdi:refresh" /></template>
          刷新
        </NButton>
      </template>
      <div class="grid grid-cols-3 gap-4">
        <div class="text-center">
          <NStatistic label="当前连续" :value="store.streak.current_streak">
            <template #suffix>天</template>
          </NStatistic>
        </div>
        <div class="text-center">
          <NStatistic label="最长连续" :value="store.streak.longest_streak">
            <template #suffix>天</template>
          </NStatistic>
        </div>
        <div class="text-center">
          <NStatistic
            label="今日状态"
            :value="store.streak.completed_today ? '已完成' : '未完成'"
          />
        </div>
      </div>
      <div class="mt-3 text-xs text-gray-500">
        规则：每天至少完成一个任务计为打卡；当天无任务不中断也不计入；当天有任务但未完成则中断。
      </div>
    </NCard>

    <!-- 添加自定义鼓励语 -->
    <NCard :bordered="false">
      <template #header>
        <div class="flex items-center gap-2">
          <Icon
            icon="mdi:plus-circle-outline"
            width="20"
            class="text-brand-500"
          />
          <span>添加自定义鼓励语</span>
        </div>
      </template>
      <NSpace align="center" :wrap="false">
        <NInput
          v-model:value="newText"
          placeholder="输入鼓励语，回车快速添加"
          style="width: 360px"
          maxlength="100"
          show-count
          @keyup.enter="handleAdd"
        />
        <NSelect
          v-model:value="newLevel"
          :options="levelOptions"
          style="width: 180px"
        />
        <NButton type="primary" @click="handleAdd">
          <template #icon><Icon icon="mdi:plus" /></template>
          添加
        </NButton>
      </NSpace>
    </NCard>

    <!-- 按等级展示鼓励语 -->
    <NCard
      v-for="level in [
        'normal',
        'advanced',
        'highlight',
        'celebration',
      ] as EncouragementLevel[]"
      :key="level"
      :bordered="false"
    >
      <template #header>
        <div class="flex items-center gap-2">
          <Icon
            :icon="LEVEL_META[level].icon"
            width="20"
            :class="LEVEL_META[level].iconColor"
          />
          <span>{{ LEVEL_META[level].label }}鼓励语</span>
          <NTag size="tiny" :type="LEVEL_META[level].color" round>
            {{ LEVEL_META[level].desc }}
          </NTag>
        </div>
      </template>
      <div
        v-if="
          presetByLevel[level].length > 0 || customByLevel[level].length > 0
        "
        class="space-y-2"
      >
        <div
          v-for="item in [...presetByLevel[level], ...customByLevel[level]]"
          :key="item.id"
          class="p-3 rounded border text-sm flex items-start gap-2"
          :class="{
            'border-blue-100 bg-blue-50/50': item.category === 'preset',
            'border-green-100 bg-green-50/50': item.category === 'custom',
          }"
        >
          <Icon
            icon="mdi:format-quote-open"
            width="16"
            class="text-gray-400 mt-0.5"
          />
          <span class="flex-1">{{ item.text }}</span>
          <NTag
            size="tiny"
            :bordered="false"
            :type="item.category === 'preset' ? 'info' : 'success'"
          >
            {{ item.category === "preset" ? "预设" : "自定义" }}
          </NTag>
          <NPopconfirm
            v-if="item.category === 'custom'"
            @positive-click="handleDelete(item.id)"
          >
            <template #trigger>
              <NButton size="tiny" quaternary type="error">
                <Icon icon="mdi:delete" width="14" />
              </NButton>
            </template>
            确定删除这条鼓励语？
          </NPopconfirm>
        </div>
      </div>
      <NEmpty v-else :description="`暂无${LEVEL_META[level].label}鼓励语`" />
    </NCard>
  </div>
</template>
