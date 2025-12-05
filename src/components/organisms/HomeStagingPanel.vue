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
        <StagingList :hide-header="true" />
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import StagingList from '@/components/assembles/tasks/list/StagingList.vue'
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
</style>
