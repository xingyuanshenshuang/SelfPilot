# 技术选型与组件库落地明细

> 项目：离线个人任务规划桌面应用
> 选型日期：2026-06-22
> 目标平台：Windows / macOS

---

## 1. 核心框架与语言

| 项目 | 选型结论 | 推荐版本 | 核心理由 |
|------|----------|----------|----------|
| 桌面框架 | **Tauri** | 2.x（最新稳定版） | 系统 WebView 渲染，冷启动 0.3–1s，包体 4–15 MB，满足 PRD 启动 < 3s 的硬性指标；同时预留 iOS/Android 扩展路径 |
| 后端语言 | **Rust** | 1.80+ | 内存安全 + 零成本抽象，SQLite 批量写入性能比 Node.js 方案快约 1.3–1.5 倍；复杂进度汇总可在 Rust 侧多线程计算，前端仅接收结果，保证 < 0.5s 响应 |
| 前端框架 | **Vue 3** | 3.4+ | Composition API 逻辑复用度高，模板语法学习曲线平缓，降低团队跨语言认知负担；响应式系统（Proxy）配合 shallowRef 可在 500–1000 节点树形结构中保持 60fps |
| 前端语言 | **TypeScript** | 5.5+ | 与 Rust 强类型风格一致，减少 IPC 接口的类型错位；Pinia 与 Vue 3 深度集成，类型推导完善 |
| 构建工具 | **Vite** | 5.x+ | Tauri 官方默认集成，HMR 极速，开发体验与生产构建一致性高 |

---

## 2. UI 组件库（基础控件）

| 项目 | 选型结论 | 推荐版本 | 核心理由 |
|------|----------|----------|----------|
| 基础组件库 | **Naive UI** | 2.x+ | Vue 3 专用，组件轻量、主题定制能力强，支持深色模式（PRD 要求浅色/深色主题）；Dialog、Drawer、Progress 等控件与任务规划场景高度契合 |
| 图标库 | **@iconify/vue** | 4.x+ | 按需加载，图标总量 20 万+，任务状态徽标（✅ / 🟡 / 🔲 / ⚠️）可快速映射，不增加包体积 |
| CSS 框架 | **UnoCSS** | 0.60+ | 原子化按需生成，避免传统 CSS 框架的冗余；与 Vite 集成零配置，适合左右布局、网格日历等高密度 UI |

> 关联 PRD：首页左右结构、目标进度条、今日待办列表、弹窗输入实际完成量等交互均依赖基础控件的高频调用，Naive UI 的 Vue 3 原生实现可避免 wrapper 层性能损耗。

---

## 3. 树形结构组件（目标树模块）

| 项目 | 选型结论 | 推荐版本 | 核心理由 |
|------|----------|----------|----------|
| 树形组件 | **vue-vtree** 或 **he-tree-vue** | 3.x+ | 支持虚拟滚动（Virtual Scroll），实测 1000 节点无卡顿；内置拖拽（drag-and-drop）与右键菜单插槽，满足 PRD 中「在任意节点右键或点击【+】添加子任务」的交互需求 |
| 拖拽增强 | **vuedraggable**（备用） | 4.x+ | 若 vtree 内置拖拽不满足精细化需求，可配合 vuedraggable 实现节点跨层级拖拽，动画流畅 |

> 关联 PRD：目标树为应用核心模块，节点含目标/阶段/任务三级，需支持展开/折叠、右键菜单、拖拽排序。虚拟滚动是满足 < 0.5s 操作响应的关键。

---

## 4. 日历视图组件（月/周/日网格）

| 项目 | 选型结论 | 推荐版本 | 核心理由 |
|------|----------|----------|----------|
| 日历组件 | **FullCalendar**（Vue 3 封装） | 6.x+ | 月/周/日三视图原生支持，单元格内容完全自定义渲染，可在日期格内嵌入任务状态徽标（✅ / 🟡 / 🔲 / ⚠️）；点击日期展开详情列表的交互可通过 dateClick 事件直接实现 |
| 日期处理 | **date-fns** | 3.x+ | 函数式、按需引入，比 moment.js 包体积小 90%；自动拆解计划、逾期判断、周期任务（每日/每周几/每月几号）均依赖日期计算 |

> 关联 PRD：日历视图需展示自动拆解任务 + 周期性手动任务，每个单元格内显示任务名称及完成状态。FullCalendar 的 view 切换与自定义 render 能力可精准还原 PRD 首页结构图中的日历交互。

---

## 5. 图表库（柱状图、折线图、日历热力图）

| 项目 | 选型结论 | 推荐版本 | 核心理由 |
|------|----------|----------|----------|
| 综合图表 | **ECharts**（vue-echarts 封装） | 5.5+ | 柱状图、折线图、热力图均原生支持，中文文档完善；canvas 渲染在 1000 数据点下仍保持 60fps，满足 < 0.5s 渲染响应 |
| 热力图专用 | **ECharts Heatmap** | 内置 | 日历热力图（GitHub-contribution 风格）通过 calendar + heatmap 坐标系直接实现，颜色深浅映射每日完成比例（完成任务量 / 当日应有任务总量），与 PRD 数据统计模块需求完全吻合 |
| 环形进度 | **ECharts Pie/Gauge** | 内置 | 目标总览页的环形进度条可用 Pie 的 ring 类型实现，支持动态更新百分比 |

> 关联 PRD：数据统计模块需切换「各目标完成百分比（柱状图）」与「近 7 天/30 天趋势（折线图）」，以及日历热力图。ECharts 单库覆盖全部需求，避免多图表库混用导致的包体积膨胀。

---

## 6. 状态管理方案

| 项目 | 选型结论 | 推荐版本 | 核心理由 |
|------|----------|----------|----------|
| 全局状态 | **Pinia** | 2.x+ | Vue 3 官方推荐，类型支持完善，模块化管理适合多视图（目标树、日历、统计）共享任务数据 |
| 本地存储同步 | **pinia-plugin-persistedstate** | 3.x+ | 将 Pinia state 自动持久化到 localStorage，应用重启后快速恢复界面状态，减少 SQLite 初始查询压力 |

### 子任务完成量实时汇总到父级进度的核心逻辑

PRD 要求：子任务完成量变化后，实时滚动更新父级（阶段/目标）的完成百分比。推荐采用 **"Rust 侧计算 + Pinia 单向同步"** 架构：

```
[用户操作] → Vue 组件调用 invoke('update_task_progress')
                ↓
         [Tauri Command / Rust]
                ↓
    sqlx 更新任务完成量 → 递归 CTE 查询整棵子树
                ↓
    Rust 计算各父节点汇总进度（多线程并行）
                ↓
    返回 { updated_nodes: Vec<NodeProgress> }
                ↓
         [Pinia Store]
                ↓
    patch 更新对应节点进度 → 触发 Vue 响应式重渲染
```

**为什么放在 Rust 侧计算：**

- 进度汇总涉及递归查询（CTE）与浮点除法，Rust 侧可利用 sqlx 的异步连接池与 tokio 多线程，批量计算 1000 个节点的进度可在 10ms 级完成；
- 前端仅接收扁平化的 `updated_nodes` 数组，通过 Pinia 的 `patch` 方法局部更新 state，避免整棵树重新序列化，保证 UI 响应 < 0.5s；
- 若未来云端同步，Rust 侧的汇总逻辑可直接复用，只需替换数据源为 HTTP API。

---

## 7. 本地数据持久化

| 项目 | 选型结论 | 推荐版本 | 核心理由 |
|------|----------|----------|----------|
| SQLite 驱动 | **tauri-plugin-sql**（官方插件） | 2.x+ | 官方维护，底层直接绑定 Rust sqlite3，跨平台一致；支持参数化查询与事务，防止 SQL 注入 |
| Rust ORM/查询层 | **sqlx** | 0.8+ | 编译时 SQL 检查（query checked at compile time），避免运行时 schema 不匹配；对递归 CTE（父子级查询）有完全控制权，适合进度汇总优化；异步原生，与 Tauri Command 无缝衔接 |
| 数据建模 | **sqlx migrate** + 手写 DDL | 内置 | 轻量迁移管理，版本控制友好；PRD 数据模型（UUID + parent_id + path）结构清晰，手写 DDL 比 ORM 生成更可控 |
| JSON 导入导出 | **serde**（Rust）+ **serde_json** | 1.0+ | Rust 生态标准序列化库，结构体与 JSON 一键互转；导出时全量读取 SQLite → Vec<Goal> → JSON 文件，导入时反向解析并处理 ID 冲突（覆盖/跳过/重命名） |

### JSON 全量导入导出实现要点

- **导出**：Rust Command 中通过 sqlx 查询全量数据（Goals + Stages + Tasks）组装为嵌套结构体，serde_json 序列化后由 Tauri 的 fs API 写入用户指定路径；
- **导入**：读取 JSON → serde_json 反序列化为 Vec<Goal> → 遍历处理 UUID 冲突（根据用户选择覆盖/跳过/生成新 ID）→ sqlx 事务批量插入；事务保证导入原子性，失败可回滚。

---

## 8. 项目目录结构建议

按功能模块分层，前后端代码隔离，便于后续云端接口替换。

```
self-study-planner/
├── src/                          # 前端源码（Vue 3 + TypeScript）
│   ├── main.ts                   # 应用入口
│   ├── App.vue                   # 根组件（左右布局）
│   ├── api/                      # Tauri Command 调用封装
│   │   ├── goal.ts               # 目标相关 IPC
│   │   ├── task.ts               # 任务相关 IPC
│   │   ├── calendar.ts           # 日历查询 IPC
│   │   ├── stats.ts              # 统计图表 IPC
│   │   └── sync.ts               # 【预留】云端同步接口
│   ├── stores/                   # Pinia 状态管理
│   │   ├── goalStore.ts          # 目标树状态 + 进度汇总监听
│   │   ├── taskStore.ts          # 任务状态（今日待办、逾期）
│   │   ├── calendarStore.ts      # 日历视图状态
│   │   ├── statsStore.ts         # 统计图表数据缓存
│   │   └── settingsStore.ts      # 用户设置 + 主题
│   ├── views/                    # 页面级组件（对应左侧导航）
│   │   ├── DashboardView.vue     # 目标总览
│   │   ├── GoalTreeView.vue      # 目标树
│   │   ├── CalendarView.vue      # 日历视图
│   │   ├── StatsView.vue         # 数据统计
│   │   ├── EncouragementView.vue # 鼓励语库
│   │   └── SettingsView.vue      # 设置
│   ├── components/               # 公共/复用组件
│   │   ├── GoalTree/             # 树形组件封装
│   │   ├── CalendarGrid/         # 日历网格封装
│   │   ├── Charts/               # ECharts 封装（柱状图、折线图、热力图）
│   │   ├── TaskItem.vue          # 任务行项（待办/逾期）
│   │   └── ProgressRing.vue      # 环形进度条
│   ├── composables/              # Vue Composition 复用逻辑
│   │   ├── useProgressCalc.ts    # 进度计算监听
│   │   ├── useDragDrop.ts        # 拖拽逻辑
│   │   └── useTheme.ts           # 深色/浅色主题切换
│   ├── types/                    # 全局 TypeScript 类型
│   │   └── index.ts              # Goal, Stage, Task, Progress 等接口
│   └── utils/                    # 工具函数
│       ├── date.ts               # date-fns 封装
│       └── uuid.ts               # UUID 生成
│
├── src-tauri/                    # Tauri / Rust 后端
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 配置
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 模块聚合
│   │   ├── commands/             # Tauri Commands（IPC 接口层）
│   │   │   ├── goal.rs           # 目标 CRUD + 自动拆解
│   │   │   ├── task.rs           # 任务 CRUD + 完成/跳过/补完成
│   │   │   ├── calendar.rs       # 日历范围查询
│   │   │   ├── stats.rs          # 统计数据聚合
│   │   │   ├── export_import.rs  # JSON 导出/导入
│   │   │   └── sync.rs           # 【预留】云端同步 Command
│   │   ├── db/                   # 数据库层
│   │   │   ├── mod.rs            # 连接池初始化
│   │   │   ├── migrate.rs        # sqlx migrate 管理
│   │   │   ├── models.rs         # Rust 结构体（Goal, Stage, Task）
│   │   │   └── queries.rs        # sqlx 查询封装（含 CTE 递归）
│   │   ├── services/             # 业务逻辑层
│   │   │   ├── progress_service.rs   # 进度汇总计算（核心）
│   │   │   ├── split_service.rs      # 自动拆解算法
│   │   │   └── sync_service.rs       # 【预留】同步协议（diff + timestamp）
│   │   └── error.rs              # 全局错误类型
│   └── migrations/               # sqlx 迁移文件
│       ├── 001_init.sql
│       └── 002_add_encouragement.sql
│
├── public/                       # 静态资源
├── package.json
├── vite.config.ts
├── tsconfig.json
└── uno.config.ts                 # UnoCSS 配置
```

**分层设计意图：**

- `src/api/` 与 `src-tauri/src/commands/` 一一对应，前端开发者只需关注 TypeScript 接口，Rust 开发者负责底层实现；
- `src-tauri/src/services/` 独立业务逻辑层，进度汇总、自动拆解等核心算法集中管理，未来云端同步只需在 services 层增加 HTTP client 实现，commands 层无需改动；
- `sync.rs` 与 `sync_service.rs` 为预留模块，初期仅返回 mock 数据，降低后续云端功能接入成本。

---

## 9. 技术栈全家桶一览

| 层级 | 选型 | 版本 | 一句话选择理由 |
|------|------|------|----------------|
| 桌面框架 | Tauri | 2.x | 启动 < 1s、包体 < 15 MB，唯一同时满足轻量与极速启动 |
| 后端语言 | Rust | 1.80+ | 内存安全，SQLite 批量性能比 Node.js 快 1.3–1.5 倍 |
| 前端框架 | Vue 3 | 3.4+ | 模板语法平缓，响应式系统支撑 1000 节点 60fps |
| 前端语言 | TypeScript | 5.5+ | 与 Rust 强类型风格一致，减少 IPC 类型错位 |
| 构建工具 | Vite | 5.x+ | Tauri 官方默认，HMR 极速 |
| 基础 UI 库 | Naive UI | 2.x+ | Vue 3 原生、主题定制强，深色模式一键切换 |
| 图标库 | @iconify/vue | 4.x+ | 20 万+ 图标按需加载，任务状态徽标映射零成本 |
| CSS 引擎 | UnoCSS | 0.60+ | 原子化按需生成，避免传统框架冗余 |
| 树形组件 | vue-vtree | 3.x+ | 虚拟滚动 + 内置拖拽/右键，1000 节点无卡顿 |
| 日历组件 | FullCalendar | 6.x+ | 月/周/日三视图原生支持，单元格自定义渲染任务徽标 |
| 日期工具 | date-fns | 3.x+ | 按需引入，包体比 moment.js 小 90% |
| 图表库 | ECharts | 5.5+ | 柱状图、折线图、热力图单库覆盖，中文文档完善 |
| 状态管理 | Pinia | 2.x+ | Vue 3 官方推荐，模块化管理多视图共享数据 |
| 状态持久化 | pinia-plugin-persistedstate | 3.x+ | 自动同步 localStorage，减少 SQLite 初始查询 |
| SQLite 驱动 | tauri-plugin-sql | 2.x+ | 官方维护，跨平台一致，支持事务 |
| Rust 查询层 | sqlx | 0.8+ | 编译时 SQL 检查，CTE 递归完全可控，异步原生 |
| JSON 序列化 | serde + serde_json | 1.0+ | Rust 标准，结构体与 JSON 一键互转，导入导出原子事务 |
