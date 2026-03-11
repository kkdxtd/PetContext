<script setup lang="ts">
import { computed } from 'vue'

interface SubTask {
  id: number
  name: string
  xp: number
  time: string
  done: boolean
}

interface TaskBreakdown {
  task: string
  sub_tasks: SubTask[]
  health_tip: string
  total_xp: number
}

const props = defineProps<{
  tasks: TaskBreakdown | null
}>()

const completedCount = computed(() => {
  if (!props.tasks) return 0
  return props.tasks.sub_tasks.filter(t => t.done).length
})

const totalXp = computed(() => {
  if (!props.tasks) return 0
  return props.tasks.sub_tasks
    .filter(t => t.done)
    .reduce((sum, t) => sum + t.xp, 0)
})

function toggleTask(task: SubTask) {
  task.done = !task.done
}

function getTaskIcon(task: SubTask): string {
  if (task.done) return '✅'
  if (task.name.includes('准备')) return '📋'
  if (task.name.includes('核心') || task.name.includes('执行')) return '⚡'
  if (task.name.includes('检查') || task.name.includes('收尾')) return '✅'
  if (task.name.includes('休息') || task.name.includes('放松')) return '☕'
  return '📝'
}
</script>

<template>
  <div class="task-list">
    <div class="task-header">
      <span class="title">📋 任务清单</span>
      <span class="badge" v-if="tasks">{{ completedCount }}/{{ tasks.sub_tasks.length }}</span>
    </div>
    
    <div v-if="!tasks" class="empty-state">
      <div class="empty-icon">📝</div>
      <p>暂无任务</p>
      <p class="hint">在左侧发送任务，我会帮你拆解</p>
    </div>
    
    <div v-else class="task-content">
      <div class="task-main">
        <div class="main-task">{{ tasks.task }}</div>
        <div class="health-tip">{{ tasks.health_tip }}</div>
      </div>
      
      <div class="progress-section">
        <div class="progress-bar">
          <div 
            class="progress-fill" 
            :style="{ width: (completedCount / tasks.sub_tasks.length * 100) + '%' }"
          ></div>
        </div>
        <div class="xp-info">
          💪 {{ totalXp }} / {{ tasks.total_xp }} XP
        </div>
      </div>
      
      <div class="sub-tasks">
        <div
          v-for="task in tasks.sub_tasks"
          :key="task.id"
          class="task-item"
          :class="{ completed: task.done }"
          @click="toggleTask(task)"
        >
          <div class="checkbox">
            <span v-if="task.done">✓</span>
          </div>
          <div class="task-info">
            <span class="task-name">{{ getTaskIcon(task) }} {{ task.name }}</span>
            <span class="task-meta">{{ task.time }} · {{ task.xp }} XP</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.task-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f8f9fa;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 14px;
  border-bottom: 1px solid #eee;
}

.title {
  font-weight: 600;
  font-size: 13px;
  color: #333;
}

.badge {
  background: #667eea;
  color: white;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #999;
  padding: 20px;
}

.empty-icon {
  font-size: 40px;
  margin-bottom: 10px;
}

.empty-state p {
  margin: 4px 0;
  font-size: 13px;
}

.hint {
  font-size: 11px !important;
  color: #bbb;
}

.task-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.task-main {
  background: white;
  padding: 12px;
  border-radius: 10px;
  margin-bottom: 12px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
}

.main-task {
  font-weight: 600;
  font-size: 14px;
  color: #333;
  margin-bottom: 8px;
}

.health-tip {
  font-size: 11px;
  color: #888;
  line-height: 1.4;
}

.progress-section {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}

.progress-bar {
  flex: 1;
  height: 8px;
  background: #e9ecef;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #667eea, #764ba2);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.xp-info {
  font-size: 12px;
  font-weight: 600;
  color: #667eea;
  white-space: nowrap;
}

.sub-tasks {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: white;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 1px 3px rgba(0,0,0,0.06);
}

.task-item:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 6px rgba(0,0,0,0.1);
}

.task-item.completed {
  opacity: 0.6;
}

.task-item.completed .task-name {
  text-decoration: line-through;
  color: #999;
}

.checkbox {
  width: 20px;
  height: 20px;
  border: 2px solid #ddd;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: white;
  transition: all 0.2s;
  flex-shrink: 0;
}

.task-item.completed .checkbox {
  background: #667eea;
  border-color: #667eea;
}

.task-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.task-name {
  font-size: 12px;
  color: #333;
  line-height: 1.3;
}

.task-meta {
  font-size: 10px;
  color: #888;
}
</style>
