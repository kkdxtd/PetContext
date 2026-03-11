<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import ChatArea from './ChatArea.vue'
import TaskList from './TaskList.vue'
import SettingsModal from './SettingsModal.vue'

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
  visible: boolean
  petX: number
  petY: number
  petWidth: number
  petHeight: number
}>()

const emit = defineEmits<{
  close: []
  updateTasks: [tasks: TaskBreakdown]
}>()

const DIALOG_WIDTH = 700
const DIALOG_HEIGHT = 500

const screenSize = ref({ width: 1920, height: 1080 })
const showSettings = ref(false)
const currentTasks = ref<TaskBreakdown | null>(null)

const position = computed(() => {
  const { petX, petY, petWidth, petHeight } = props
  const { width: screenWidth, height: screenHeight } = screenSize.value
  
  const rightSpace = screenWidth - (petX + petWidth)
  const leftSpace = petX
  const topSpace = petY
  const bottomSpace = screenHeight - (petY + petHeight)
  
  const showAbove = topSpace > bottomSpace && topSpace >= DIALOG_HEIGHT
  const showBelow = !showAbove && bottomSpace >= DIALOG_HEIGHT
  const showLeft = leftSpace > rightSpace && leftSpace >= DIALOG_WIDTH
  
  let x: number, y: number
  
  if (showLeft) {
    x = petX - DIALOG_WIDTH - 10
  } else {
    x = petX + petWidth + 10
  }
  
  if (showAbove) {
    y = petY - DIALOG_HEIGHT
  } else if (showBelow) {
    y = petY + petHeight
  } else {
    y = petY
  }
  
  x = Math.max(10, Math.min(x, screenWidth - DIALOG_WIDTH - 10))
  y = Math.max(10, Math.min(y, screenHeight - DIALOG_HEIGHT - 10))
  
  return { x, y }
})

async function updateScreenSize() {
  try {
    screenSize.value = {
      width: window.screen.width,
      height: window.screen.height
    }
  } catch (e) {
    console.error('Failed to get screen size:', e)
    screenSize.value = { width: 1920, height: 1080 }
  }
}

function handleClose() {
  emit('close')
}

function handleTasksUpdate(tasks: TaskBreakdown) {
  currentTasks.value = tasks
  emit('updateTasks', tasks)
}

function toggleSettings() {
  showSettings.value = !showSettings.value
}

watch(() => props.visible, async (newVal) => {
  if (newVal) {
    await updateScreenSize()
  }
})

onMounted(async () => {
  await updateScreenSize()
})
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-overlay" @click.self="handleClose">
      <div 
        class="dialog-panel"
        :style="{ 
          left: position.x + 'px', 
          top: position.y + 'px' 
        }"
      >
        <div class="dialog-header">
          <span class="dialog-title">🐱 任务规划助手</span>
          <div class="header-actions">
            <button class="icon-btn" @click="toggleSettings" title="设置">
              ⚙️
            </button>
            <button class="icon-btn close-btn" @click="handleClose" title="关闭">
              ✕
            </button>
          </div>
        </div>
        
        <div class="dialog-content">
          <div class="chat-section">
            <ChatArea 
              @tasks-updated="handleTasksUpdate"
            />
          </div>
          <div class="task-section">
            <TaskList :tasks="currentTasks" />
          </div>
        </div>
      </div>
      
      <SettingsModal 
        v-if="showSettings" 
        @close="showSettings = false" 
      />
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 1000;
}

.dialog-panel {
  position: fixed;
  width: 700px;
  height: 500px;
  background: rgba(255, 255, 255, 0.98);
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.dialog-title {
  font-weight: bold;
  font-size: 15px;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.icon-btn {
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: white;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.close-btn:hover {
  background: rgba(255, 100, 100, 0.8);
}

.dialog-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.chat-section {
  flex: 1;
  border-right: 1px solid #eee;
  overflow: hidden;
}

.task-section {
  width: 280px;
  overflow: hidden;
}
</style>
