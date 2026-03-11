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

interface SimpleModelConfig {
  id: string
  name: string
  apiUrl: string
  apiKey: string
  model: string
  isActive: boolean
  isDefault: boolean
}

const emit = defineEmits<{
  showSettings: []
}>()

const messages = ref<Message[]>([])
const inputText = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
const isLoading = ref(false)
const currentModel = ref<SimpleModelConfig | null>(null)
let messageId = 0

// 预设模型配置
const presetModels: SimpleModelConfig[] = [
  {
    id: 'aliyun-qwen',
    name: '阿里云 Qwen',
    apiUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    apiKey: '',
    model: 'qwen3.5-plus',
    isActive: true,
    isDefault: true
  },
  {
    id: 'openai',
    name: 'OpenAI GPT',
    apiUrl: 'https://api.openai.com/v1',
    apiKey: '',
    model: 'gpt-4o-mini',
    isActive: false,
    isDefault: false
  }
]

function loadConfig(): SimpleModelConfig {
  const defaultModel = presetModels[0]!
  try {
    const saved = localStorage.getItem('simple_llm_config')
    if (saved) {
      const config = JSON.parse(saved)
      return { ...defaultModel, ...config }
    }
  } catch (e) {
    console.error('Failed to load config:', e)
  }
  return defaultModel
}

function saveConfig(config: SimpleModelConfig) {
  localStorage.setItem('simple_llm_config', JSON.stringify(config))
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
    addMessage('请先配置 API Key！点击右上角设置按钮进行配置。', false)
    return
  }
  
  addMessage(text, true)
  inputText.value = ''
  
  const thinkingMsg: Message = {
    id: ++messageId,
    text: '🐱 正在思考中...',
    isUser: false,
    timestamp: new Date(),
    isThinking: true
  }
  messages.value.push(thinkingMsg)
  isLoading.value = true
  
  try {
    // 使用通用对话接口替代任务拆解
    const result = await invoke<any>('chat_with_model', {
      apiUrl: config.apiUrl,
      apiKey: config.apiKey,
      model: config.model,
      message: text
    })
    
    messages.value = messages.value.filter(m => m.id !== thinkingMsg.id)
    
    if (result.success) {
      addMessage(result.response, false)
    } else {
      addMessage(`❌ ${result.error || '对话失败，请重试'}`, false)
    }
  } catch (error: any) {
    messages.value = messages.value.filter(m => m.id !== thinkingMsg.id)
    addMessage(`❌ 调用失败：${error.message || error}`, false)
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

function switchModel() {
  const config = loadConfig()
  const availableModels = presetModels.filter(m => m.id !== config.id)
  
  if (availableModels.length > 0) {
    const newModel = availableModels[0]
    if (newModel) {
      newModel.isActive = true
      newModel.apiKey = config.apiKey // 保留API密钥
      config.isActive = false
      saveConfig({...newModel, apiKey: config.apiKey})
      currentModel.value = {...newModel, apiKey: config.apiKey}
      addMessage(`🔄 已切换到 ${newModel.name}`, false)
    }
  }
}

function testConnection() {
  const config = loadConfig()
  if (!config.apiKey) {
    addMessage('⚠️ 请先配置 API Key', false)
    return
  }
  
  addMessage(`🧪 正在测试 ${config.name} 连接...`, false)
  
  invoke<boolean>('test_model_connection', {
    apiUrl: config.apiUrl,
    apiKey: config.apiKey,
    model: config.model
  }).then(success => {
    if (success) {
      addMessage('✅ 连接测试成功！', false)
    } else {
      addMessage('❌ 连接测试失败，请检查配置', false)
    }
  }).catch(error => {
    addMessage(`❌ 测试失败：${error}`, false)
  })
}

onMounted(() => {
  currentModel.value = loadConfig()
  addMessage('🐱 你好！我是你的桌面猫猫助手\n\n有什么我可以帮助你的吗？', false)
})
</script>

<template>
  <div class="simple-chat">
    <!-- 顶部状态栏 -->
    <div class="chat-header">
      <div class="model-info">
        <span class="model-name">{{ currentModel?.name || '未配置' }}</span>
        <span class="model-status" :class="{ 'connected': currentModel?.apiKey }">
          {{ currentModel?.apiKey ? '🟢 已配置' : '🔴 未配置' }}
        </span>
      </div>
      <div class="header-actions">
        <button @click="switchModel" class="action-btn" title="切换模型">
          🔁
        </button>
        <button @click="testConnection" class="action-btn" title="测试连接">
          🧪
        </button>
        <button @click="$emit('showSettings')" class="action-btn" title="设置">
          ⚙️
        </button>
      </div>
    </div>
    
    <!-- 消息区域 -->
    <div class="messages-container" ref="messagesContainer">
      <div
        v-for="msg in messages"
        :key="msg.id"
        class="message"
        :class="{ 
          'user-message': msg.isUser, 
          'bot-message': !msg.isUser,
          'thinking': msg.isThinking 
        }"
      >
        <div class="message-content">
          {{ msg.text }}
        </div>
        <div class="message-time" v-if="!msg.isThinking">
          {{ msg.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}
        </div>
      </div>
    </div>
    
    <!-- 输入区域 -->
    <div class="input-area">
      <input
        v-model="inputText"
        type="text"
        placeholder="和猫猫聊聊天吧..."
        @keydown="handleKeydown"
        :disabled="isLoading || !currentModel?.apiKey"
        class="message-input"
      />
      <button 
        @click="sendMessage" 
        :disabled="!inputText.trim() || isLoading || !currentModel?.apiKey"
        class="send-button"
      >
        {{ isLoading ? '⏳' : '发送' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.simple-chat {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  overflow: hidden;
}

.chat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

.model-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.model-name {
  font-size: 14px;
  font-weight: 600;
  color: white;
}

.model-status {
  font-size: 11px;
  color: #ff6b6b;
}

.model-status.connected {
  color: #51cf66;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: white;
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: translateY(-1px);
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message {
  max-width: 85%;
  padding: 12px 16px;
  border-radius: 18px;
  font-size: 14px;
  line-height: 1.5;
  position: relative;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.user-message {
  align-self: flex-end;
  background: rgba(255, 255, 255, 0.95);
  color: #333;
  border-bottom-right-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.bot-message {
  align-self: flex-start;
  background: rgba(255, 255, 255, 0.25);
  color: white;
  border-bottom-left-radius: 6px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.thinking {
  align-self: flex-start;
  background: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.7);
  font-style: italic;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.7; }
  50% { opacity: 1; }
}

.message-content {
  white-space: pre-wrap;
  word-break: break-word;
}

.message-time {
  font-size: 10px;
  opacity: 0.7;
  margin-top: 6px;
  text-align: right;
}

.input-area {
  display: flex;
  gap: 10px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(10px);
  border-top: 1px solid rgba(255, 255, 255, 0.2);
}

.message-input {
  flex: 1;
  padding: 12px 16px;
  border: none;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.9);
  font-size: 14px;
  outline: none;
  transition: all 0.2s;
}

.message-input:focus {
  background: white;
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.5);
}

.message-input:disabled {
  background: rgba(255, 255, 255, 0.5);
  cursor: not-allowed;
}

.send-button {
  padding: 12px 24px;
  background: rgba(255, 255, 255, 0.9);
  color: #667eea;
  border: none;
  border-radius: 24px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.send-button:hover:not(:disabled) {
  background: white;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.send-button:disabled {
  background: rgba(255, 255, 255, 0.5);
  color: #ccc;
  cursor: not-allowed;
  transform: none;
}

/* 滚动条样式 */
.messages-container::-webkit-scrollbar {
  width: 6px;
}

.messages-container::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}

.messages-container::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.3);
  border-radius: 3px;
}

.messages-container::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.5);
}
</style>