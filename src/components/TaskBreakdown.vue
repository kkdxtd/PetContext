<script setup lang="ts">
import { ref, computed } from 'vue'

interface Task {
  id: number
  text: string
  completed: boolean
  createdAt: Date
}

const tasks = ref<Task[]>([])
const newTaskText = ref('')
let taskId = 0

const activeTasks = computed(() => tasks.value.filter(t => !t.completed))
const completedTasks = computed(() => tasks.value.filter(t => t.completed))

function addTask() {
  const text = newTaskText.value.trim()
  if (!text) return
  
  tasks.value.push({
    id: ++taskId,
    text,
    completed: false,
    createdAt: new Date()
  })
  newTaskText.value = ''
}

function toggleTask(task: Task) {
  task.completed = !task.completed
}

function deleteTask(task: Task) {
  const index = tasks.value.findIndex(t => t.id === task.id)
  if (index > -1) {
    tasks.value.splice(index, 1)
  }
}

function clearCompleted() {
  tasks.value = tasks.value.filter(t => !t.completed)
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    addTask()
  }
}
</script>

<template>
  <div class="task-panel">
    <div class="task-header">
      <span>任务列表</span>
      <span class="task-count" v-if="activeTasks.length > 0">{{ activeTasks.length }}</span>
    </div>
    
    <div class="task-stats" v-if="tasks.length > 0">
      <span>完成: {{ completedTasks.length }}/{{ tasks.length }}</span>
      <button 
        v-if="completedTasks.length > 0" 
        @click="clearCompleted"
        class="clear-btn"
      >
        清除已完成
      </button>
    </div>
    
    <div class="task-list">
      <div v-if="tasks.length === 0" class="empty-state">
        <p>暂无任务</p>
        <p class="hint">添加你的第一个任务吧</p>
      </div>
      
      <div
        v-for="task in tasks"
        :key="task.id"
        class="task-item"
        :class="{ completed: task.completed }"
      >
        <label class="checkbox-wrapper">
          <input
            type="checkbox"
            :checked="task.completed"
            @change="toggleTask(task)"
          />
          <span class="checkmark"></span>
        </label>
        <span class="task-text">{{ task.text }}</span>
        <button class="delete-btn" @click="deleteTask(task)">×</button>
      </div>
    </div>
    
    <div class="input-area">
      <input
        v-model="newTaskText"
        type="text"
        placeholder="添加新任务..."
        @keydown="handleKeydown"
      />
      <button @click="addTask" :disabled="!newTaskText.trim()">
        添加
      </button>
    </div>
  </div>
</template>

<style scoped>
.task-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 12px;
  overflow: hidden;
}

.task-header {
  padding: 12px 16px;
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  color: white;
  font-weight: bold;
  font-size: 14px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.task-count {
  background: rgba(255,255,255,0.3);
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 12px;
}

.task-stats {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  font-size: 12px;
  color: #666;
  border-bottom: 1px solid #eee;
}

.clear-btn {
  background: none;
  border: none;
  color: #f5576c;
  cursor: pointer;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
}

.clear-btn:hover {
  background: rgba(245, 87, 108, 0.1);
}

.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #999;
  font-size: 13px;
}

.empty-state .hint {
  font-size: 11px;
  margin-top: 8px;
  color: #bbb;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  border-radius: 8px;
  margin-bottom: 8px;
  background: #f8f9fa;
  transition: all 0.2s;
}

.task-item:hover {
  background: #f0f0f0;
}

.task-item.completed .task-text {
  text-decoration: line-through;
  color: #999;
}

.checkbox-wrapper {
  position: relative;
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.checkbox-wrapper input {
  opacity: 0;
  width: 0;
  height: 0;
}

.checkmark {
  position: absolute;
  top: 0;
  left: 0;
  width: 18px;
  height: 18px;
  background: white;
  border: 2px solid #ddd;
  border-radius: 4px;
  transition: all 0.2s;
}

.checkbox-wrapper input:checked ~ .checkmark {
  background: #f5576c;
  border-color: #f5576c;
}

.checkmark:after {
  content: "";
  position: absolute;
  display: none;
  left: 5px;
  top: 1px;
  width: 4px;
  height: 9px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.checkbox-wrapper input:checked ~ .checkmark:after {
  display: block;
}

.task-text {
  flex: 1;
  font-size: 13px;
  color: #333;
}

.delete-btn {
  background: none;
  border: none;
  color: #999;
  font-size: 18px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  opacity: 0;
  transition: all 0.2s;
}

.task-item:hover .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  color: #f5576c;
  background: rgba(245, 87, 108, 0.1);
}

.input-area {
  display: flex;
  gap: 8px;
  padding: 12px;
  border-top: 1px solid #eee;
}

.input-area input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 20px;
  outline: none;
  font-size: 13px;
}

.input-area input:focus {
  border-color: #f5576c;
}

.input-area button {
  padding: 8px 16px;
  background: #f5576c;
  color: white;
  border: none;
  border-radius: 20px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s;
}

.input-area button:hover:not(:disabled) {
  background: #e0465a;
}

.input-area button:disabled {
  background: #ccc;
  cursor: not-allowed;
}
</style>
