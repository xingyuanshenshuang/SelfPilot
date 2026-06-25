-- SelfPilot Sprint 5 迁移：鼓励语等级 + 预设鼓励语分级
-- 对应 PRD §4.2 模块七 & 分阶段计划 Sprint 5：鼓励语个性化规则

-- 添加 level 字段（normal | advanced | highlight | celebration）
ALTER TABLE encouragements ADD COLUMN level TEXT NOT NULL DEFAULT 'normal'
    CHECK(level IN ('normal','advanced','highlight','celebration'));

-- 将现有预设鼓励语分级：
-- preset-01 ~ preset-04 → normal（普通，1天）
-- preset-05 ~ preset-07 → advanced（进阶，3天）
-- preset-08 ~ preset-09 → highlight（高亮，7天）
-- preset-10 → celebration（庆祝，全部完成）
UPDATE encouragements SET level = 'normal' WHERE id IN ('preset-01','preset-02','preset-03','preset-04');
UPDATE encouragements SET level = 'advanced' WHERE id IN ('preset-05','preset-06','preset-07');
UPDATE encouragements SET level = 'highlight' WHERE id IN ('preset-08','preset-09');
UPDATE encouragements SET level = 'celebration' WHERE id = 'preset-10';

-- 新增进阶鼓励语（3天连续）
INSERT OR IGNORE INTO encouragements (id, text, category, level, created_at) VALUES
    ('preset-adv-01', '连续三天打卡，习惯正在养成！', 'preset', 'advanced', '2026-06-15T00:00:00'),
    ('preset-adv-02', '三连击！你已超越多数人的坚持。', 'preset', 'advanced', '2026-06-15T00:00:00'),
    ('preset-adv-03', '坚持三天的你，比昨天更强大。', 'preset', 'advanced', '2026-06-15T00:00:00');

-- 新增高亮鼓励语（7天连续）
INSERT OR IGNORE INTO encouragements (id, text, category, level, created_at) VALUES
    ('preset-hl-01', '一周打卡达成！你已是自律的代名词。', 'preset', 'highlight', '2026-06-15T00:00:00'),
    ('preset-hl-02', '七日坚持，星光不负赶路人。', 'preset', 'highlight', '2026-06-15T00:00:00'),
    ('preset-hl-03', '一周不间断，未来可期！', 'preset', 'highlight', '2026-06-15T00:00:00');

-- 新增庆祝鼓励语（全部目标完成）
INSERT OR IGNORE INTO encouragements (id, text, category, level, created_at) VALUES
    ('preset-cele-01', '全部目标已完成！你是自己的英雄！', 'preset', 'celebration', '2026-06-15T00:00:00'),
    ('preset-cele-02', '征程圆满！这一刻属于坚持不懈的你。', 'preset', 'celebration', '2026-06-15T00:00:00'),
    ('preset-cele-03', '所有目标达成！为这段旅程画上完美句号。', 'preset', 'celebration', '2026-06-15T00:00:00');
