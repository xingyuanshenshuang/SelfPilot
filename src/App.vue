<script setup lang="ts">
import { ref, computed, defineAsyncComponent } from "vue";
import {
  NLayout,
  NLayoutSider,
  NLayoutContent,
  NMenu,
  NMessageProvider,
  NDialogProvider,
  NConfigProvider,
  NScrollbar,
  darkTheme,
} from "naive-ui";
import type { MenuOption } from "naive-ui";
import { Icon } from "@iconify/vue";
import { h } from "vue";

const activeView = ref<string>("dashboard");
const isDark = ref(false);

const DashboardView = defineAsyncComponent(
  () => import("./views/DashboardView.vue"),
);
const GoalTreeView = defineAsyncComponent(
  () => import("./views/GoalTreeView.vue"),
);

const menuOptions: MenuOption[] = [
  {
    label: "目标总览",
    key: "dashboard",
    icon: () => h(Icon, { icon: "mdi:view-dashboard-outline" }),
  },
  {
    label: "目标树",
    key: "goalTree",
    icon: () => h(Icon, { icon: "mdi:file-tree-outline" }),
  },
  {
    label: "日历视图",
    key: "calendar",
    icon: () => h(Icon, { icon: "mdi:calendar-month-outline" }),
    disabled: true,
  },
  {
    label: "数据统计",
    key: "stats",
    icon: () => h(Icon, { icon: "mdi:chart-line" }),
    disabled: true,
  },
  {
    label: "鼓励语库",
    key: "encouragement",
    icon: () => h(Icon, { icon: "mdi:emoticon-happy-outline" }),
    disabled: true,
  },
  {
    label: "设置",
    key: "settings",
    icon: () => h(Icon, { icon: "mdi:cog-outline" }),
    disabled: true,
  },
];

const currentView = computed(() => {
  switch (activeView.value) {
    case "dashboard":
      return DashboardView;
    case "goalTree":
      return GoalTreeView;
    default:
      return null;
  }
});
</script>

<template>
  <n-config-provider :theme="isDark ? darkTheme : null">
    <n-message-provider>
      <n-dialog-provider>
        <n-layout has-sider class="h-screen">
          <n-layout-sider
            bordered
            :width="208"
            :collapsed-width="64"
            show-trigger="bar"
            collapse-mode="width"
          >
            <div
              class="logo flex items-center gap-2 px-4 py-4 text-lg font-bold text-brand-600"
            >
              <Icon icon="mdi:rocket-launch" width="24" />
              <span v-if="true">SelfPilot</span>
            </div>
            <n-menu
              v-model:value="activeView"
              :options="menuOptions"
              :indent="18"
              :collapsed-width="64"
              :collapsed-icon-size="20"
            />
          </n-layout-sider>
          <n-layout>
            <n-layout-content class="h-full">
              <n-scrollbar>
                <div class="p-4">
                  <component :is="currentView" v-if="currentView" />
                  <div
                    v-else
                    class="flex h-full items-center justify-center text-gray-400"
                  >
                    <div class="text-center">
                      <Icon icon="mdi:hammer-wrench" width="48" />
                      <p class="mt-2">该模块开发中</p>
                    </div>
                  </div>
                </div>
              </n-scrollbar>
            </n-layout-content>
          </n-layout>
        </n-layout>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
html,
body,
#app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family:
    -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue",
    Arial, "PingFang SC", "Microsoft YaHei", sans-serif;
}
</style>
