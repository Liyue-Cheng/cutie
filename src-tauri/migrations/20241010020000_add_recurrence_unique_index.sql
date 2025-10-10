-- 添加唯一索引防止并发创建重复的循环实例
--
-- 确保同一个循环规则在同一天只能有一个实例

CREATE UNIQUE INDEX IF NOT EXISTS idx_recurrence_instance_unique
ON task_recurrence_links(recurrence_id, instance_date);

