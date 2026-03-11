<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface LLMConfig {
  apiUrl: string
  apiKey: string
  model: string
}

const emit = defineEmits<{
  close: []
}>()

const config = ref<LLMConfig>({
  apiUrl: '',
  apiKey: '',
  model: ''
})

const showKey = ref(false)

function loadConfig() {
  try {
    const saved = localStorage.getItem('llm_config')
    if (saved) {
      config.value = JSON.parse(saved)
    } else {
      config.value = {
        apiUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
        apiKey: '',
        model: 'qwen3.5-plus'
      }
    }
  } catch (e) {
    console.error('Failed to load config:', e)
  }
}

function saveConfig() {
  localStorage.setItem('llm_config', JSON.stringify(config.value))
  emit('close')
}

function resetToDefault() {
  config.value = {
    apiUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    apiKey: '',
    model: 'qwen3.5-plus'
  }
}

onMounted(() => {
  loadConfig()
})
</script>

<template>
  <div class="settings-overlay" @click.self="$emit('close')">
    <div class="settings-modal">
      <div class="modal-header">
        <span class="modal-title">⚙️ LLM API 设置</span>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>
      
      <div class="modal-content">
        <div class="form-group">
          <label>API Base URL</label>
          <input 
            v-model="config.apiUrl" 
            type="text" 
            placeholder="https://dashscope.aliyuncs.com/compatible-mode/v1"
          />
          <span class="hint">LLM 服务的 API 地址</span>
        </div>
        
        <div class="form-group">
          <label>API Key</label>
          <div class="input-with-toggle">
            <input 
              v-model="config.apiKey" 
              :type="showKey ? 'text' : 'password'" 
              placeholder="输入你的 API Key"
            />
            <button class="toggle-btn" @click="showKey = !showKey">
              {{ showKey ? '🙈' : '👁️' }}
            </button>
          </div>
          <span class="hint">用于身份验证的密钥</span>
        </div>
        
        <div class="form-group">
          <label>Model</label>
          <input 
            v-model="config.model" 
            type="text" 
            placeholder="qwen3.5-plus"
          />
          <span class="hint">使用的模型名称</span>
        </div>
        
        <div class="preset-section">
          <span class="preset-label">常用预设：</span>
          <div class="preset-buttons">
            <button @click="config = { apiUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1', apiKey: config.apiKey, model: 'qwen3.5-plus' }">
              阿里云 Qwen
            </button>
            <button @click="config = { apiUrl: 'https://api.openai.com/v1', apiKey: config.apiKey, model: 'gpt-4o-mini' }">
              OpenAI
            </button>
            <button @click="config = { apiUrl: 'https://ark.cn-beijing.volces.com/api/v3', apiKey: config.apiKey, model: 'doubao-seed-1-6-flash-250828' }">
              火山引擎
            </button>
          </div>
        </div>
      </div>
      
      <div class="modal-footer">
        <button class="reset-btn" @click="resetToDefault">恢复默认</button>
        <div class="footer-right">
          <button class="cancel-btn" @click="$emit('close')">取消</button>
          <button class="save-btn" @click="saveConfig">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.settings-modal {
  width: 420px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.modal-title {
  font-weight: 600;
  font-size: 14px;
}

.close-btn {
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: white;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.modal-content {
  padding: 20px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: #333;
  margin-bottom: 6px;
}

.form-group input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 13px;
  outline: none;
}

.form-group input:focus {
  border-color: #667eea;
}

.hint {
  display: block;
  font-size: 11px;
  color: #888;
  margin-top: 4px;
}

.input-with-toggle {
  display: flex;
  gap: 8px;
}

.input-with-toggle input {
  flex: 1;
}

.toggle-btn {
  padding: 0 12px;
  background: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
}

.toggle-btn:hover {
  background: #eee;
}

.preset-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid #eee;
}

.preset-label {
  font-size: 12px;
  color: #666;
  margin-bottom: 8px;
  display: block;
}

.preset-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.preset-buttons button {
  padding: 6px 12px;
  background: #f0f0f0;
  border: none;
  border-radius: 6px;
  font-size: 11px;
  color: #666;
  cursor: pointer;
  transition: all 0.2s;
}

.preset-buttons button:hover {
  background: #e0e0e0;
  color: #333;
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  padding: 14px 20px;
  background: #f9f9f9;
  border-top: 1px solid #eee;
}

.footer-right {
  display: flex;
  gap: 8px;
}

.reset-btn {
  padding: 8px 14px;
  background: transparent;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 12px;
  color: #666;
  cursor: pointer;
}

.reset-btn:hover {
  background: #f0f0f0;
}

.cancel-btn {
  padding: 8px 14px;
  background: transparent;
  border: none;
  border-radius: 8px;
  font-size: 12px;
  color: #666;
  cursor: pointer;
}

.cancel-btn:hover {
  background: #eee;
}

.save-btn {
  padding: 8px 20px;
  background: #667eea;
  border: none;
  border-radius: 8px;
  font-size: 12px;
  color: white;
  cursor: pointer;
}

.save-btn:hover {
  background: #5568d3;
}
</style>
