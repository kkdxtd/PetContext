<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { PhysicalSize, PhysicalPosition } from '@tauri-apps/api/dpi'
import { exit } from '@tauri-apps/plugin-process'
import live2d from './utils/live2d'
import DialogPanel from './components/DialogPanel.vue'

interface MouseButtonEvent {
  kind: 'MousePress' | 'MouseRelease'
  value: string
}

interface MouseMoveEvent {
  kind: 'MouseMove'
  value: { x: number; y: number }
}

interface KeyboardEvent {
  kind: 'KeyboardPress' | 'KeyboardRelease'
  value: string
}

type DeviceEvent = MouseButtonEvent | MouseMoveEvent | KeyboardEvent

const isLoading = ref(true)
const errorMsg = ref('')
const isDialogOpen = ref(false)
const backgroundImage = ref('')
const windowScale = ref(1)
const isDragging = ref(false)
const petSize = ref({ width: 0, height: 0 })

// 猫猫气泡
const bubbleText = ref('')
const bubbleVisible = ref(false)
let bubbleTimer: ReturnType<typeof setTimeout> | null = null

// 右键菜单
const contextMenuVisible = ref(false)
const contextMenuPos = ref({ x: 0, y: 0 })

const MODEL_WIDTH = 612
const MODEL_HEIGHT = 354


let unlistenDevice: UnlistenFn | null = null
let unlistenBubble: UnlistenFn | null = null
let windowPosition = { x: 0, y: 0 }
let dragStartPos = { x: 0, y: 0 }
let windowStartPos = { x: 0, y: 0 }
let appWindow = getCurrentWebviewWindow()
let lastClickTime = 0

async function updateWindowPosition() {
  try {
    const pos = await appWindow.outerPosition()
    windowPosition = { x: pos.x, y: pos.y }
  } catch (e) {
    console.error('Failed to get position:', e)
  }
}

async function initLive2D() {
  try {
    console.log('Initializing Live2D...')
    const { width, height } = await live2d.load('dummy')
    console.log('Model loaded:', width, height)
    
    backgroundImage.value = '/models/standard/resources/background.png'
    
    const scaledWidth = Math.round(width * windowScale.value)
    const scaledHeight = Math.round(height * windowScale.value)
    await appWindow.setSize(new PhysicalSize(scaledWidth, scaledHeight))
    petSize.value = { width: scaledWidth, height: scaledHeight }
    
    await updateWindowPosition()
    
    live2d.resizeModel({ width, height })
    isLoading.value = false

    // 启动后显示欢迎气泡
    setTimeout(() => {
      showBubble('你好呀~ 双击我可以打开任务面板!')
    }, 800)
  } catch (e) {
    console.error('Failed to load Live2D:', e)
    errorMsg.value = String(e)
    isLoading.value = false
  }
}

async function startDeviceListening() {
  try {
    await invoke('start_device_listening')
    console.log('Device listening started')
  } catch (e) {
    console.error('Failed to start device listening:', e)
  }
}

async function handleDeviceEvent(event: DeviceEvent) {
  const { kind, value } = event
  const key = String(value)

  if (kind === 'KeyboardPress') {
    live2d.setParameterValue('CatParamLeftHandDown', 1)
    live2d.setParameterValue('CatParamRightHandDown', 1)
  } else if (kind === 'KeyboardRelease') {
    live2d.setParameterValue('CatParamLeftHandDown', 0)
    live2d.setParameterValue('CatParamRightHandDown', 0)
  } else if (kind === 'MousePress') {
    if (key.includes('Left')) {
      live2d.setParameterValue('ParamMouseLeftDown', 1)
    } else if (key.includes('Right')) {
      live2d.setParameterValue('ParamMouseRightDown', 1)
    }
  } else if (kind === 'MouseRelease') {
    live2d.setParameterValue('ParamMouseLeftDown', 0)
    live2d.setParameterValue('ParamMouseRightDown', 0)
  } else if (kind === 'MouseMove') {
    const moveValue = value as { x: number; y: number }
    const mouseX = moveValue.x
    const mouseY = moveValue.y
    
    const relX = (mouseX - windowPosition.x) / (MODEL_WIDTH * windowScale.value)
    const relY = (mouseY - windowPosition.y) / (MODEL_HEIGHT * windowScale.value)
    
    const clampedX = Math.max(0, Math.min(1, relX))
    const clampedY = Math.max(0, Math.min(1, relY))
    
    live2d.setParameterValue('ParamMouseX', clampedX)
    live2d.setParameterValue('ParamMouseY', clampedY)
    
    const angleX = (clampedX - 0.5) * 30
    const angleY = (clampedY - 0.5) * 30
    live2d.setParameterValue('ParamAngleX', angleX)
    live2d.setParameterValue('ParamAngleY', angleY)
  }
}

async function openChatWindow() {
  try {
    await invoke('create_chat_window')
    showBubble('双击我可以打开/关闭任务面板哦~')
  } catch (e) {
    console.error('Failed to create chat window:', e)
  }
}

function showBubble(text: string, duration = 6000) {
  bubbleText.value = text
  bubbleVisible.value = true
  if (bubbleTimer) clearTimeout(bubbleTimer)
  bubbleTimer = setTimeout(() => {
    bubbleVisible.value = false
  }, duration)
}

function handleMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  
  // 点击左键时关闭右键菜单
  contextMenuVisible.value = false
  
  e.preventDefault()
  e.stopPropagation()
  
  const now = Date.now()
  
  if (now - lastClickTime < 500) {
    openChatWindow()
    lastClickTime = 0
    return
  }
  lastClickTime = now
  
  isDragging.value = true
  dragStartPos = { x: e.screenX, y: e.screenY }
  
  appWindow.outerPosition().then(pos => {
    windowStartPos = { x: pos.x, y: pos.y }
  })
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragging.value) return
  
  const deltaX = e.screenX - dragStartPos.x
  const deltaY = e.screenY - dragStartPos.y
  const newX = windowStartPos.x + deltaX
  const newY = windowStartPos.y + deltaY
  
  appWindow.setPosition(new PhysicalPosition(newX, newY))
  windowPosition = { x: newX, y: newY }
}

function handleMouseUp() {
  isDragging.value = false
}

async function handleResizeStart(e: MouseEvent) {
  e.stopPropagation()
  if (e.button !== 0) return
  
  const startX = e.clientX
  const startY = e.clientY
  const startSize = await appWindow.innerSize()
  const startWidth = startSize.width
  const startHeight = startSize.height
  
  const onMouseMove = (moveEvent: MouseEvent) => {
    const deltaX = moveEvent.clientX - startX
    const deltaY = moveEvent.clientY - startY
    const newWidth = Math.max(200, startWidth + deltaX)
    const newHeight = Math.max(150, startHeight + deltaY)
    appWindow.setSize(new PhysicalSize(newWidth, newHeight))
    
    windowScale.value = newWidth / MODEL_WIDTH
    live2d.resizeModel({ width: MODEL_WIDTH, height: MODEL_HEIGHT })
  }
  
  const onMouseUp = () => {
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
    updateWindowPosition()
  }
  
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

async function handleMiddleClick(e: MouseEvent) {
  if (e.button === 1) {
    e.preventDefault()
    windowScale.value = 1
    await appWindow.setSize(new PhysicalSize(MODEL_WIDTH, MODEL_HEIGHT))
    live2d.resizeModel({ width: MODEL_WIDTH, height: MODEL_HEIGHT })
  }
}

async function handleWheel(e: WheelEvent) {
  if (e.ctrlKey) {
    e.preventDefault()
    const delta = e.deltaY > 0 ? -0.1 : 0.1
    const newScale = Math.max(0.3, Math.min(3, windowScale.value + delta))
    windowScale.value = newScale
    
    const newWidth = Math.round(MODEL_WIDTH * newScale)
    const newHeight = Math.round(MODEL_HEIGHT * newScale)
    await appWindow.setSize(new PhysicalSize(newWidth, newHeight))
    live2d.resizeModel({ width: MODEL_WIDTH, height: MODEL_HEIGHT })
  }
}

function handleContextMenu(e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  contextMenuPos.value = { x: e.clientX, y: e.clientY }
  contextMenuVisible.value = true
}

async function handleMenuOpenChat() {
  contextMenuVisible.value = false
  await openChatWindow()
}

async function handleExit() {
  contextMenuVisible.value = false
  await exit(0)
}

onMounted(async () => {
  await initLive2D()
  await startDeviceListening()
  
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
  
  unlistenDevice = await listen<DeviceEvent>('device-changed', (event) => {
    handleDeviceEvent(event.payload)
  })

  unlistenBubble = await listen<{ text: string }>('cat-bubble', (event) => {
    showBubble(event.payload.text)
  })
})

onUnmounted(() => {
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
  unlistenDevice?.()
  unlistenBubble?.()
  if (bubbleTimer) clearTimeout(bubbleTimer)
  live2d.destroy()
})
</script>

<template>
  <div 
    class="app-container" 
    :class="{ 'panel-open': isDialogOpen }"
    @mousedown="handleMouseDown"
    @contextmenu="handleContextMenu"
    @wheel="handleWheel"
    @auxclick="handleMiddleClick"
  >
    <img v-if="backgroundImage" :src="backgroundImage" class="background-img" alt="background" />
    
    <canvas id="live2dCanvas"></canvas>

    <!-- 右键菜单 -->
    <div 
      v-if="contextMenuVisible" 
      class="context-menu"
      :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
      @mousedown.stop
      @contextmenu.stop.prevent
    >
      <div class="context-menu-item" @click="handleMenuOpenChat">
        <span class="context-menu-icon">&#x1F4AC;</span> 打开面板
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item exit-item" @click="handleExit">
        <span class="context-menu-icon">&#x274C;</span> 退出程序
      </div>
    </div>

    <!-- 猫猫云朵气泡 -->
    <transition name="bubble">
      <div v-if="bubbleVisible" class="cat-bubble" @click="bubbleVisible = false">
        <div class="bubble-text">{{ bubbleText }}</div>
      </div>
    </transition>
    
    <div v-if="isLoading" class="loading">
      加载中...
    </div>
    
    <div v-if="errorMsg" class="error">
      {{ errorMsg }}
    </div>
    
    <div class="resize-handle" @mousedown.stop="handleResizeStart"></div>
    
    <div class="scale-indicator">{{ Math.round(windowScale * 100) }}%</div>
    
    <DialogPanel 
      :visible="isDialogOpen"
      :pet-x="windowPosition.x"
      :pet-y="windowPosition.y"
      :pet-width="petSize.width"
      :pet-height="petSize.height"
      @close="isDialogOpen = false"
    />
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
  background: transparent;
  overflow: hidden;
}

body {
  background: transparent !important;
}

.app-container {
  position: relative;
  width: 100%;
  height: 100%;
  cursor: move;
  pointer-events: auto;
}

.background-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  pointer-events: none;
  z-index: 0;
}

#live2dCanvas {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: block;
  z-index: 1;
}

.resize-handle {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 20px;
  height: 20px;
  cursor: se-resize;
  z-index: 20;
  background: linear-gradient(135deg, transparent 50%, rgba(255,255,255,0.3) 50%);
}

.scale-indicator {
  position: absolute;
  bottom: 5px;
  left: 5px;
  font-size: 10px;
  color: rgba(255,255,255,0.5);
  z-index: 20;
  pointer-events: none;
}

.loading, .error {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: white;
  font-size: 14px;
  text-shadow: 0 1px 2px rgba(0,0,0,0.5);
  z-index: 20;
}

.error {
  color: #ff6b6b;
}

/* 猫猫云朵气泡 - 右上角显示 */
.cat-bubble {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 30;
  pointer-events: auto;
  cursor: pointer;
  max-width: 55%;
}

.bubble-text {
  background: white;
  color: #444;
  font-size: 12px;
  line-height: 1.5;
  padding: 6px 12px;
  border-radius: 12px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.15);
  word-break: break-word;
  position: relative;
}

/* 云朵尾巴指向左下方（猫猫方向） */
.bubble-text::after {
  content: '';
  position: absolute;
  bottom: -7px;
  left: 16px;
  width: 12px;
  height: 12px;
  background: white;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0,0,0,0.08);
}

.cat-bubble::after {
  content: '';
  position: absolute;
  bottom: -15px;
  left: 10px;
  width: 7px;
  height: 7px;
  background: white;
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0,0,0,0.06);
}

/* 气泡出入动画 */
.bubble-enter-active {
  animation: bubblePop 0.35s ease-out;
}

.bubble-leave-active {
  animation: bubbleFade 0.3s ease-in forwards;
}

@keyframes bubblePop {
  0% { opacity: 0; transform: scale(0.5) translateY(10px); }
  60% { transform: scale(1.05) translateY(-2px); }
  100% { opacity: 1; transform: scale(1) translateY(0); }
}

@keyframes bubbleFade {
  to { opacity: 0; transform: scale(0.8) translateY(-5px); }
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  z-index: 100;
  background: rgba(255, 255, 255, 0.96);
  backdrop-filter: blur(12px);
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.25);
  padding: 4px 0;
  min-width: 140px;
  animation: menuFadeIn 0.15s ease-out;
}

@keyframes menuFadeIn {
  from { opacity: 0; transform: scale(0.9); }
  to { opacity: 1; transform: scale(1); }
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  font-size: 13px;
  color: #333;
  cursor: pointer;
  transition: background 0.15s;
  white-space: nowrap;
  user-select: none;
  pointer-events: auto;
}

.context-menu-item:hover {
  background: rgba(102, 126, 234, 0.12);
}

.context-menu-item.exit-item:hover {
  background: rgba(255, 80, 80, 0.12);
  color: #e53e3e;
}

.context-menu-icon {
  font-size: 14px;
  width: 18px;
  text-align: center;
}

.context-menu-divider {
  height: 1px;
  background: rgba(0, 0, 0, 0.1);
  margin: 4px 8px;
}
</style>
