<template>
  <div class="home-staging-panel">
    <TwoRowLayout>
      <template #top>
        <div class="panel-controls">
          <!-- 左侧：标题 -->
          <div class="controls-left">
            <div class="panel-title-wrapper">
              <span class="panel-title-text">{{ $t('task.label.scheduled') }}</span>
            </div>
          </div>

          <!-- 中间：占位 -->
          <div class="spacer"></div>

          <!-- 右侧控制组（预留） -->
          <div class="controls-right"></div>
        </div>
      </template>

      <template #bottom>
        <div class="staging-groups-wrapper">
          <StagingTaskGroups />
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import StagingTaskGroups from '@/components/assembles/tasks/StagingTaskGroups.vue'
</script>

<style scoped>
.home-staging-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ==================== 控制栏 ==================== */
.panel-controls {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.2rem;
  padding: 1.2rem 0.8rem 1.2rem 1.6rem;
  background-color: transparent;
}

.controls-left {
  position: relative;
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.controls-right {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

/* ==================== 标题样式 ==================== */
.panel-title-wrapper {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.panel-title-text {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
  white-space: nowrap;
}

/* 占位 */
.spacer {
  flex: 1;
}

/* ==================== 分组容器包装 ==================== */
.staging-groups-wrapper {
  height: 100%;
  padding: 1rem;
  overflow: hidden;
}
</style>
