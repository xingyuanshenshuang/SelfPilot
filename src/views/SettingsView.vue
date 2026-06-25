<script setup lang="ts">
import { onMounted, ref } from "vue";
import {
  NCard,
  NButton,
  NSpace,
  NRadioGroup,
  NRadioButton,
  NModal,
  NSelect,
  NDescriptions,
  NDescriptionsItem,
  useMessage,
  useDialog,
} from "naive-ui";
import { Icon } from "@iconify/vue";
import { useSettingStore } from "@/stores/settingStore";
import * as backupApi from "@/api/backup";
import type { ImportConflictMode, ImportResult } from "@/types";

const settingStore = useSettingStore();
const message = useMessage();
const dialog = useDialog();

const exporting = ref(false);
const importing = ref(false);
const showImportModal = ref(false);
const importPreview = ref("");
const conflictMode = ref<ImportConflictMode>("skip");
const fileInputRef = ref<HTMLInputElement | null>(null);

const conflictOptions = [
  { label: "跳过冲突项（保留本地）", value: "skip" },
  { label: "覆盖冲突项（使用导入数据）", value: "overwrite" },
  { label: "重命名导入项（生成新 ID）", value: "rename" },
];

onMounted(async () => {
  if (!settingStore.loaded) {
    await settingStore.loadTheme();
  }
});

async function handleThemeChange(value: "light" | "dark") {
  await settingStore.setTheme(value);
  message.success(value === "dark" ? "已切换到深色主题" : "已切换到浅色主题");
}

/** 导出数据为 JSON 文件下载 */
async function handleExport() {
  exporting.value = true;
  try {
    const jsonStr = await backupApi.exportData();
    const blob = new Blob([jsonStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    const ts = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
    a.download = `selfpilot-backup-${ts}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    message.success("已导出备份文件");
  } catch (e) {
    message.error(`导出失败: ${String(e)}`);
  } finally {
    exporting.value = false;
  }
}

/** 选择导入文件 */
function triggerFileSelect() {
  fileInputRef.value?.click();
}

function handleFileChange(e: Event) {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = () => {
    importPreview.value = String(reader.result || "");
    showImportModal.value = true;
    // 重置 input，便于重复选择同一文件
    target.value = "";
  };
  reader.onerror = () => {
    message.error("文件读取失败");
  };
  reader.readAsText(file);
}

/** 确认导入 */
async function confirmImport() {
  if (!importPreview.value) {
    message.warning("没有可导入的数据");
    return;
  }
  importing.value = true;
  try {
    const result: ImportResult = await backupApi.importData({
      data: importPreview.value,
      conflict_mode: conflictMode.value,
    });
    showImportModal.value = false;
    importPreview.value = "";

    const total =
      result.goals_imported +
      result.stages_imported +
      result.tasks_imported +
      result.encouragements_imported +
      result.settings_imported;
    const skipped =
      result.goals_skipped + result.stages_skipped + result.tasks_skipped;

    message.success(
      `导入完成：共 ${total} 项${skipped > 0 ? `，跳过 ${skipped} 项` : ""}`,
    );

    dialog.info({
      title: "导入结果",
      content: () => `
        目标：导入 ${result.goals_imported}，跳过 ${result.goals_skipped}
        阶段：导入 ${result.stages_imported}，跳过 ${result.stages_skipped}
        任务：导入 ${result.tasks_imported}，跳过 ${result.tasks_skipped}
        鼓励语：导入 ${result.encouragements_imported}
        设置：导入 ${result.settings_imported}
      `,
      positiveText: "知道了",
    });
  } catch (e) {
    message.error(`导入失败: ${String(e)}`);
  } finally {
    importing.value = false;
  }
}

function cancelImport() {
  showImportModal.value = false;
  importPreview.value = "";
}
</script>

<template>
  <div class="space-y-4">
    <!-- 主题设置 -->
    <NCard :bordered="false">
      <template #header>
        <div class="flex items-center gap-2">
          <Icon icon="mdi:palette-outline" width="20" class="text-brand-500" />
          <span>主题设置</span>
        </div>
      </template>
      <NSpace vertical :size="12">
        <div class="text-sm text-gray-600">选择应用的主题外观：</div>
        <NRadioGroup
          :value="settingStore.theme"
          @update:value="handleThemeChange"
        >
          <NRadioButton value="light">
            <template #icon><Icon icon="mdi:weather-sunny" /></template>
            浅色
          </NRadioButton>
          <NRadioButton value="dark">
            <template #icon><Icon icon="mdi:weather-night" /></template>
            深色
          </NRadioButton>
        </NRadioGroup>
      </NSpace>
    </NCard>

    <!-- 数据管理 -->
    <NCard :bordered="false">
      <template #header>
        <div class="flex items-center gap-2">
          <Icon
            icon="mdi:database-cog-outline"
            width="20"
            class="text-brand-500"
          />
          <span>数据管理</span>
        </div>
      </template>
      <NSpace vertical :size="16">
        <!-- 导出 -->
        <div>
          <div class="text-sm font-medium mb-2 flex items-center gap-2">
            <Icon icon="mdi:export" width="16" class="text-green-500" />
            导出数据
          </div>
          <div class="text-xs text-gray-500 mb-2">
            将所有目标、阶段、任务、鼓励语和设置导出为 JSON 备份文件。
          </div>
          <NButton
            type="primary"
            ghost
            :loading="exporting"
            @click="handleExport"
          >
            <template #icon><Icon icon="mdi:download" /></template>
            导出备份
          </NButton>
        </div>

        <div class="border-t border-gray-100" />

        <!-- 导入 -->
        <div>
          <div class="text-sm font-medium mb-2 flex items-center gap-2">
            <Icon icon="mdi:import" width="16" class="text-blue-500" />
            导入数据
          </div>
          <div class="text-xs text-gray-500 mb-2">
            从 JSON 备份文件恢复数据。ID 冲突时可选择跳过、覆盖或重命名。
          </div>
          <NButton
            type="primary"
            ghost
            :loading="importing"
            @click="triggerFileSelect"
          >
            <template #icon><Icon icon="mdi:upload" /></template>
            选择文件导入
          </NButton>
          <input
            ref="fileInputRef"
            type="file"
            accept=".json,application/json"
            style="display: none"
            @change="handleFileChange"
          />
        </div>
      </NSpace>
    </NCard>

    <!-- 关于 -->
    <NCard :bordered="false">
      <template #header>
        <div class="flex items-center gap-2">
          <Icon
            icon="mdi:information-outline"
            width="20"
            class="text-brand-500"
          />
          <span>关于</span>
        </div>
      </template>
      <NDescriptions :column="1" label-placement="left" bordered size="small">
        <NDescriptionsItem label="应用名称"
          >SelfPilot 自学计划任务规划</NDescriptionsItem
        >
        <NDescriptionsItem label="版本">0.1.0</NDescriptionsItem>
        <NDescriptionsItem label="技术栈"
          >Tauri 2 + Vue 3 + SQLite</NDescriptionsItem
        >
        <NDescriptionsItem label="数据存储"
          >本地 SQLite（selfpilot.db）</NDescriptionsItem
        >
      </NDescriptions>
    </NCard>

    <!-- 导入确认弹窗 -->
    <NModal
      v-model:show="showImportModal"
      preset="card"
      title="确认导入数据"
      style="width: 520px"
    >
      <NSpace vertical :size="12">
        <div class="text-sm">已读取备份文件，请选择 ID 冲突时的处理方式：</div>
        <NSelect v-model:value="conflictMode" :options="conflictOptions" />
        <div class="text-xs text-gray-500">
          <div>
            <Icon icon="mdi:skip-next" width="12" class="inline" />
            跳过：保留本地数据，不导入冲突项
          </div>
          <div>
            <Icon icon="mdi:overwrite" width="12" class="inline" />
            覆盖：用导入数据替换本地冲突项
          </div>
          <div>
            <Icon icon="mdi:rename-box" width="12" class="inline" />
            重命名：为导入项生成新 ID，保留双方数据
          </div>
        </div>
      </NSpace>
      <template #footer>
        <NSpace justify="end">
          <NButton @click="cancelImport">取消</NButton>
          <NButton type="primary" :loading="importing" @click="confirmImport">
            确认导入
          </NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>
