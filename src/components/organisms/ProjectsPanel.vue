<template>
  <div class="projects-panel">
    <!-- 项目列表 -->
    <div class="project-list-section">
      <ProjectListPanel
        :selected-id="selectedProjectId"
        @select-project="handleSelectProject"
        @create-project="handleCreateProject"
        @edit-project="handleEditProject"
        @add-section="handleAddSectionFromList"
      />
    </div>

    <!-- 项目详情 -->
    <div class="project-detail-section">
      <ProjectDetailPanel
        :project-id="selectedProjectId"
        @edit-project="handleEditProject"
        @create-section="handleCreateSection"
        @edit-section="handleEditSection"
      />
    </div>

    <!-- 新建项目对话框 -->
    <ProjectCreateModal
      :show="showCreateModal"
      @close="showCreateModal = false"
      @success="handleProjectCreated"
    />

    <!-- 编辑项目对话框 -->
    <ProjectEditModal
      :show="showEditModal"
      :project-id="editingProjectId"
      @close="showEditModal = false"
      @success="handleProjectUpdated"
    />

    <!-- 添加章节对话框 -->
    <ProjectSectionCreateModal
      :show="showCreateSectionModal"
      :project-id="selectedProjectId || null"
      @close="showCreateSectionModal = false"
      @success="handleSectionCreated"
    />

    <!-- 编辑章节对话框 -->
    <ProjectSectionEditModal
      :show="showEditSectionModal"
      :section-id="editingSectionId"
      @close="showEditSectionModal = false"
      @success="handleSectionUpdated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useProjectStore } from '@/stores/project'
import { useTaskStore } from '@/stores/task'
import { pipeline } from '@/cpu'
import ProjectListPanel from '@/components/organisms/ProjectListPanel.vue'
import ProjectDetailPanel from '@/components/organisms/ProjectDetailPanel.vue'
import ProjectCreateModal from '@/components/organisms/ProjectCreateModal.vue'
import ProjectEditModal from '@/components/organisms/ProjectEditModal.vue'
import ProjectSectionCreateModal from '@/components/organisms/ProjectSectionCreateModal.vue'
import ProjectSectionEditModal from '@/components/organisms/ProjectSectionEditModal.vue'

const projectStore = useProjectStore()
const taskStore = useTaskStore()

// 当前选中的项目 ID
const selectedProjectId = ref<string | null | undefined>(undefined)

// 对话框状态
const showCreateModal = ref(false)
const showEditModal = ref(false)
const showCreateSectionModal = ref(false)
const showEditSectionModal = ref(false)
const editingProjectId = ref<string | null>(null)
const editingSectionId = ref<string | null>(null)

// 选择项目
const handleSelectProject = (id: string | null) => {
  selectedProjectId.value = id

  // 如果选择了具体项目，加载该项目的 sections
  if (id !== null) {
    pipeline
      .dispatch('project_section.fetch_all', {
        project_id: id,
      })
      .catch((error) => {
        console.error('Failed to load project sections:', error)
      })
  }
}

// 创建项目
const handleCreateProject = () => {
  showCreateModal.value = true
}

// 项目创建成功
const handleProjectCreated = async (projectId: string) => {
  console.log('✅ 项目创建成功:', projectId)
  // 选中新创建的项目
  selectedProjectId.value = projectId
  // 加载该项目的 sections
  try {
    await pipeline.dispatch('project_section.fetch_all', {
      project_id: projectId,
    })
  } catch (error) {
    console.error('Failed to load project sections:', error)
  }
}

// 编辑项目
const handleEditProject = (id: string) => {
  editingProjectId.value = id
  showEditModal.value = true
}

// 项目更新成功
const handleProjectUpdated = () => {
  console.log('✅ 项目更新成功')
}

// 创建章节
const handleCreateSection = () => {
  showCreateSectionModal.value = true
}

// 从列表右键菜单添加章节（需要先选中项目）
const handleAddSectionFromList = (projectId: string) => {
  // 先选中该项目
  selectedProjectId.value = projectId
  // 然后打开创建章节对话框
  showCreateSectionModal.value = true
}

// 章节创建成功
const handleSectionCreated = () => {
  console.log('✅ 章节创建成功')
}

// 编辑章节
const handleEditSection = (sectionId: string) => {
  editingSectionId.value = sectionId
  showEditSectionModal.value = true
}

// 章节更新成功
const handleSectionUpdated = () => {
  console.log('✅ 章节更新成功')
}

// 初始化时加载项目数据
onMounted(async () => {
  console.log('🚀 ProjectsPanel mounted')
  try {
    console.log('📥 Loading tasks...')
    await taskStore.fetchAllIncompleteTasks_DMA()
    console.log('✅ Tasks loaded:', taskStore.allTasks.length)

    console.log('📥 Loading projects...')
    await pipeline.dispatch('project.fetch_all', {})
    console.log('✅ Projects loaded:', projectStore.activeProjects.length)

    const firstProject = projectStore.activeProjects[0]
    if (firstProject) {
      console.log('📌 Selecting first project:', firstProject.name)
      selectedProjectId.value = firstProject.id
      await pipeline.dispatch('project_section.fetch_all', {
        project_id: firstProject.id,
      })
    } else {
      selectedProjectId.value = null
      console.log('ℹ️ No projects found, default to "无项目" view')
    }
  } catch (error) {
    console.error('❌ Failed to load projects or tasks:', error)
    selectedProjectId.value = null
  }
})
</script>

<style scoped>
.projects-panel {
  display: flex;
  width: 100%;
  height: 100%;
  background: var(--color-background-content, #f0f);
  gap: 1px;
}

.project-list-section {
  width: 32rem;
  flex-shrink: 0;
  height: 100%;
  background: var(--color-background-content, #f0f);
  border-right: 1px solid var(--color-border-adaptive-light-normal-dark-soft, #f0f);
}

.project-detail-section {
  flex: 1;
  height: 100%;
  overflow: hidden;
}
</style>
