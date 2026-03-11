<script setup lang="ts">
import { ref, nextTick, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Message {
  id: number
  text: string
  isUser: boolean
  timestamp: Date
  isThinking?: boolean
}

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

interface LLMConfig {
  apiUrl: string
  apiKey: string
  model: string
}

const emit = defineEmits<{
  tasksUpdated: [tasks: TaskBreakdown]
}>()

const messages = ref<Message[]>([])
const inputText = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
const isLoading = ref(false)
let messageId = 0

const defaultConfig: LLMConfig = {
  apiUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
  apiKey: '',
  model: 'qwen3.5-plus'
}

function loadConfig(): LLMConfig {
  try {
    const saved = localStorage.getItem('llm_config')
    if (saved) {
      return { ...defaultConfig, ...JSON.parse(saved) }
    }
  } catch (e) {
    console.error('Failed to load config:', e)
  }
  return defaultConfig
}

function addMessage(text: string, isUser: boolean, isThinking = false) {
  messages.value.push({
    id: ++messageId,
    text,
    isUser,
    timestamp: new Date(),
    isThinking
  })
  scrollToBottom()
}

function scrollToBottom() {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

async function sendMessage() {
  const text = inputText.value.trim()
  if (!text || isLoading.value) return
  
  const config = loadConfig()
  if (!config.apiKey) {
    addMessage('请先在设置中配置 LLM API Key！', false)
    return
  }
  
  addMessage(text, true)
  inputText.value = ''
  
  const thinkingMsg: Message = {
    id: ++messageId,
    text: '正在思考中...',
    isUser: false,
    timestamp: new Date(),
    isThinking: true
  }
  messages.value.push(thinkingMsg)
  isLoading.value = true
  
  try {
    const result = await invoke<any>('breakdown_task', {
      apiUrl: config.apiUrl,
      apiKey: config.apiKey,
      model: config.model,
      task: text
    })
    
    messages.value = messages.value.filter(m => m.id !== thinkingMsg.id)
    
    if (result.type === 'task_breakdown') {
      const taskBreakdown: TaskBreakdown = {
        task: result.task,
        sub_tasks: result.sub_tasks,
        health_tip: result.health_tip,
        total_xp: result.total_xp
      }
      
      addMessage(`好的！我已经把「${text}」拆解成了以下任务：`, false)
      
      let taskList = '\n'
      result.sub_tasks.forEach((task: any) => {
        taskList += `${task.id}. ${task.name}\n   ⏱️ ${task.time} | ✨ ${task.xp} XP\n`
      })
      taskList += `\n💪 总计可获得 ${result.total_xp} XP！`
      taskList += `\n\n${result.health_tip}`
      
      addMessage(taskList, false)
      
      emit('tasksUpdated', taskBreakdown)
    } else if (result.type === 'error') {
      addMessage(`出错了：${result.message}`, false)
    } else {
      addMessage(JSON.stringify(result, null, 2), false)
    }
  } catch (error: any) {
    messages.value = messages.value.filter(m => m.id !== thinkingMsg.id)
    addMessage(`调用失败：${error}`, false)
  } finally {
    isLoading.value = false
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    sendMessage()
  }
}

onMounted(() => {
  addMessage('你好！我是你的桌面猫猫助手 🐱\n\n请告诉我你想要完成的任务，我会帮你拆解成小步骤！', false)
})
</script>

<template>
  <div class="chat-area">
    <div class="messages" ref="messagesContainer">
      <div
        v-for="msg in messages"
        :key="msg.id"
        class="message"
        :class="{ 'user': msg.isUser, 'pet': !msg.isUser, 'thinking': msg.isThinking }"
      >
        <div class="message-content">{{ msg.text }}</div>
        <div class="message-time" v-if="!msg.isThinking">
          {{ msg.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}
        </div>
      </div>
    </div>
    
    <div class="input-area">
      <input
        v-model="inputText"
        type="text"
        placeholder="输入任务让猫猫帮你拆解..."
        @keydown="handleKeydown"
        :disabled="isLoading"
      />
      <button @click="sendMessage" :disabled="!inputText.trim() || isLoading">
        {{ isLoading ? '...' : '发送' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.chat-area {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #fafafa;
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.message {
  max-width: 85%;
  padding: 10px 14px;
  border-radius: 14px;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.message.user {
  align-self: flex-end;
  background: #667eea;
  color: white;
  border-bottom-right-radius: 4px;
}

.message.pet {
  align-self: flex-start;
  background: white;
  color: #333;
  border-bottom-left-radius: 4px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.message.thinking {
  align-self: flex-start;
  background: #f0f0f0;
  color: #888;
  font-style: italic;
}

.message-time {
  font-size: 10px;
  opacity: 0.7;
  margin-top: 4px;
  text-align: right;
}

.input-area {
  display: flex;
  gap: 8px;
  padding: 12px;
  border-top: 1px solid #eee;
  background: white;
}

.input-area input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid #ddd;
  border-radius: 20px;
  outline: none;
  font-size: 13px;
}

.input-area input:focus {
  border-color: #667eea;
}

.input-area input:disabled {
  background: #f5f5f5;
}

.input-area button {
  padding: 10px 20px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 20px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: background 0.2s;
}

.input-area button:hover:not(:disabled) {
  background: #5568d3;
}

.input-area button:disabled {
  background: #ccc;
  cursor: not-allowed;
}
</style>
