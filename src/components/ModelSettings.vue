<script setup lang="ts">
import { ref, onMounted } from 'vue'

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
  close: []
}>()

const config = ref<SimpleModelConfig>({
  id: '',
  name: '',
  apiUrl: '',
  apiKey: '',
  model: '',
  isActive: false,
  isDefault: false
})

const showKey = ref(false)
const isTesting = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)

// 预设模型配置
const presetModels = [
  {
    id: 'aliyun-qwen',
    name: '阿里云 Qwen',
    apiUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    model: 'qwen3.5-plus'
  },
  {
    id: 'openai',
    name: 'OpenAI GPT',
    apiUrl: 'https://api.openai.com/v1',
    model: 'gpt-4o-mini'
  },
  {
    id: 'volcengine',
    name: '火山引擎 Doubao',
    apiUrl: 'https://ark.cn-beijing.volces.com/api/v3',
    model: 'doubao-seed-1-6-flash-250828'
  }
]

function loadConfig() {
  try {
    const saved = localStorage.getItem('simple_llm_config')
    if (saved) {
      config.value = JSON.parse(saved)
    } else {
      // 默认使用第一个预设
      const defaultPreset = presetModels[0]
      if (defaultPreset) {
        config.value = {
          id: defaultPreset.id,
          name: defaultPreset.name,
          apiUrl: defaultPreset.apiUrl,
          apiKey: '',
          model: defaultPreset.model,
          isActive: true,
          isDefault: true
        }
      }
    }
  } catch (e) {
    console.error('Failed to load config:', e)
    // 设置默认值
    const defaultPreset = presetModels[0]
    if (defaultPreset) {
      config.value = {
        id: defaultPreset.id,
        name: defaultPreset.name,
        apiUrl: defaultPreset.apiUrl,
        apiKey: '',
        model: defaultPreset.model,
        isActive: true,
        isDefault: true
      }
    }
  }
}

function saveConfig() {
  localStorage.setItem('simple_llm_config', JSON.stringify(config.value))
  emit('close')
}

function applyPreset(presetId: string) {
  const preset = presetModels.find(p => p.id === presetId)
  if (preset) {
    config.value = {
      id: preset.id,
      name: preset.name,
      apiUrl: preset.apiUrl,
      model: preset.model,
      apiKey: config.value.apiKey, // 保留API密钥
      isActive: true,
      isDefault: false
    }
  }
}

async function testConnection() {
  if (!config.value.apiKey.trim()) {
    testResult.value = {
      success: false,
      message: '请先输入 API Key'
    }
    return
  }

  isTesting.value = true
  testResult.value = null

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const success = await invoke<boolean>('test_model_connection', {
      apiUrl: config.value.apiUrl,
      apiKey: config.value.apiKey,
      model: config.value.model
    })

    testResult.value = {
      success,
      message: success ? '连接成功！' : '连接失败，请检查配置'
    }
  } catch (error: any) {
    testResult.value = {
      success: false,
      message: `测试失败：${error.message || error}`
    }
  } finally {
    isTesting.value = false
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
        <span class="modal-title">🐱 模型配置</span>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>
      
      <div class="modal-content">
        <!-- 预设选择 -->
        <div class="preset-section">
          <label>选择服务商：</label>
          <div class="preset-grid">
            <button
              v-for="preset in presetModels"
              :key="preset.id"
              @click="applyPreset(preset.id)"
              :class="{ active: config.id === preset.id }"
              class="preset-button"
            >
              {{ preset.name }}
            </button>
          </div>
        </div>

        <!-- 配置表单 -->
        <div class="config-form">
          <div class="form-group">
            <label>API 地址</label>
            <input 
              v-model="config.apiUrl" 
              type="text" 
              :placeholder="`${config.name} API 地址`"
              class="form-input"
            />
          </div>
          
          <div class="form-group">
            <label>API 密钥</label>
            <div class="input-with-toggle">
              <input 
                v-model="config.apiKey" 
                :type="showKey ? 'text' : 'password'" 
                placeholder="输入你的 API Key"
                class="form-input"
              />
              <button 
                @click="showKey = !showKey"
                class="toggle-btn"
              >
                {{ showKey ? '🙈' : '👁️' }}
              </button>
            </div>
          </div>
          
          <div class="form-group">
            <label>模型名称</label>
            <input 
              v-model="config.model" 
              type="text" 
              :placeholder="`${config.name} 模型名称`"
              class="form-input"
            />
          </div>
        </div>

        <!-- 测试连接 -->
        <div class="test-section">
          <button 
            @click="testConnection" 
            :disabled="isTesting || !config.apiKey"
            class="test-button"
          >
            {{ isTesting ? '🧪 测试中...' : '🧪 测试连接' }}
          </button>
          
          <div v-if="testResult" class="test-result" :class="{ success: testResult.success, error: !testResult.success }">
            {{ testResult.message }}
          </div>
        </div>
      </div>
      
      <div class="modal-footer">
        <button @click="$emit('close')" class="cancel-btn">取消</button>
        <button @click="saveConfig" class="save-btn">保存配置</button>
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
  width: 400px;
  background: white;
  border-radius: 16px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.modal-title {
  font-weight: 600;
  font-size: 16px;
}

.close-btn {
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: white;
  width: 28px;
  height: 28px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.modal-content {
  padding: 24px;
}

.preset-section {
  margin-bottom: 24px;
}

.preset-section label {
  display: block;
  font-size: 14px;
  font-weight: 600;
  color: #333;
  margin-bottom: 12px;
}

.preset-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.preset-button {
  padding: 12px;
  background: #f8f9fa;
  border: 2px solid #e9ecef;
  border-radius: 10px;
  font-size: 13px;
  color: #495057;
  cursor: pointer;
  transition: all 0.2s;
  text-align: center;
}

.preset-button:hover {
  background: #e9ecef;
  border-color: #adb5bd;
}

.preset-button.active {
  background: #667eea;
  border-color: #667eea;
  color: white;
}

.config-form {
  margin-bottom: 24px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: #495057;
  margin-bottom: 8px;
}

.form-input {
  width: 100%;
  padding: 12px 14px;
  border: 1px solid #ced4da;
  border-radius: 10px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.form-input:focus {
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.input-with-toggle {
  display: flex;
  gap: 10px;
}

.input-with-toggle .form-input {
  flex: 1;
}

.toggle-btn {
  padding: 0 14px;
  background: #f8f9fa;
  border: 1px solid #ced4da;
  border-radius: 10px;
  cursor: pointer;
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.toggle-btn:hover {
  background: #e9ecef;
}

.test-section {
  margin-bottom: 24px;
}

.test-button {
  width: 100%;
  padding: 14px;
  background: #667eea;
  border: none;
  border-radius: 10px;
  color: white;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.test-button:hover:not(:disabled) {
  background: #5568d3;
  transform: translateY(-1px);
}

.test-button:disabled {
  background: #adb5bd;
  cursor: not-allowed;
  transform: none;
}

.test-result {
  margin-top: 12px;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
  text-align: center;
}

.test-result.success {
  background: #d1fae5;
  color: #065f46;
  border: 1px solid #a7f3d0;
}

.test-result.error {
  background: #fee2e2;
  color: #991b1b;
  border: 1px solid #fecaca;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  background: #f8f9fa;
  border-top: 1px solid #e9ecef;
}

.cancel-btn {
  padding: 10px 20px;
  background: transparent;
  border: 1px solid #ced4da;
  border-radius: 8px;
  font-size: 14px;
  color: #6c757d;
  cursor: pointer;
}

.cancel-btn:hover {
  background: #e9ecef;
}

.save-btn {
  padding: 10px 24px;
  background: #667eea;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  color: white;
  font-weight: 600;
  cursor: pointer;
}

.save-btn:hover {
  background: #5568d3;
}
</style>