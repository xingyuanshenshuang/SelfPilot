// 预设鼓励语（PRD §4.2 模块五：不少于10条）
export const DEFAULT_ENCOURAGEMENTS: string[] = [
  "今天又进步了！",
  "坚持就是胜利，继续加油！",
  "每一步都算数，你做得很棒！",
  "学习是给自己最好的礼物。",
  "今天的努力，是明天的底气。",
  "小步快跑，日积月累就是大跨越！",
  "你比昨天的自己更强了。",
  "完成一个任务就是一次胜利！",
  "自律给你自由，继续前行。",
  "种一棵树最好的时间是十年前，其次是现在。",
];

/** 随机抽取一句鼓励语 */
export function randomEncouragement(): string {
  const idx = Math.floor(Math.random() * DEFAULT_ENCOURAGEMENTS.length);
  return DEFAULT_ENCOURAGEMENTS[idx];
}
