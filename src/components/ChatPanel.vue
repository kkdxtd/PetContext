<script setup lang="ts">
import { ref, nextTick } from 'vue'

interface Message {
  id: number
  text: string
  isUser: boolean
  timestamp: Date
}

const messages = ref<Message[]>([])
const inputText = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
let messageId = 0

function addMessage(text: string, isUser: boolean) {
  messages.value.push({
    id: ++messageId,
    text,
    isUser,
    timestamp: new Date()
  })
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

function sendMessage() {
  const text = inputText.value.trim()
  if (!text) return
  
  addMessage(text, true)
  inputText.value = ''
  
  setTimeout(() => {
    const responses = [
      '喵~ (・ω・)',
      '主人好厉害！',
      '需要我帮忙吗？',
      '继续加油吧~',
      '我在这里陪着你哦',
      '今天的任务完成了吗？'
    ]
    const randomResponse = responses[Math.floor(Math.random() * responses.length)] || '喵~'
    addMessage(randomResponse, false)
  }, 500 + Math.random() * 1000)
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    sendMessage()
  }
}
</script>

<template>
  <div class="chat-panel">
    <div class="chat-header">
      <span>聊天</span>
    </div>
    
    <div class="messages" ref="messagesContainer">
      <div v-if="messages.length === 0" class="empty-state">
        <p>点击宠物打开面板</p>
        <p class="hint">双击猫咪打开聊天面板</p>
      </div>
      
      <div
        v-for="msg in messages"
        :key="msg.id"
        class="message"
        :class="{ 'user': msg.isUser, 'pet': !msg.isUser }"
      >
        <div class="message-content">{{ msg.text }}</div>
        <div class="message-time">
          {{ msg.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}
        </div>
      </div>
    </div>
    
    <div class="input-area">
      <input
        v-model="inputText"
        type="text"
        placeholder="发送消息..."
        @keydown="handleKeydown"
      />
      <button @click="sendMessage" :disabled="!inputText.trim()">
        发送
      </button>
    </div>
  </div>
</template>

<style scoped>
.chat-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 12px;
  overflow: hidden;
}

.chat-header {
  padding: 12px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-weight: bold;
  font-size: 14px;
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
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

.message {
  max-width: 80%;
  padding: 8px 12px;
  border-radius: 12px;
  font-size: 13px;
  line-height: 1.4;
}

.message.user {
  align-self: flex-end;
  background: #667eea;
  color: white;
  border-bottom-right-radius: 4px;
}

.message.pet {
  align-self: flex-start;
  background: #f0f0f0;
  color: #333;
  border-bottom-left-radius: 4px;
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
  border-color: #667eea;
}

.input-area button {
  padding: 8px 16px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 20px;
  cursor: pointer;
  font-size: 13px;
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
