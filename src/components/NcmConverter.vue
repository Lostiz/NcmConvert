<script setup lang="ts">
import { ref, computed, h, onMounted, onUnmounted } from 'vue'
import {
  NButton,
  NDataTable,
  NSpace,
  NIcon,
  NEmpty,
  NPopconfirm,
  useMessage,
  type DataTableColumns
} from 'naive-ui'
import {
  MusicalNotesOutline,
  TrashOutline,
  CloudUploadOutline,
  FolderOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  TimeOutline
} from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, TauriEvent } from '@tauri-apps/api/event'

interface FileItem {
  id: string
  name: string
  path: string
  size: number
  status: 'pending' | 'processing' | 'success' | 'error'
  progress: number
  outputFormat?: string
  outputPath?: string
  errorMsg?: string
}

defineProps<{
  isDark: boolean
}>()

const message = useMessage()
const fileList = ref<FileItem[]>([])
const isProcessing = ref(false)
const outputDir = ref<string>('')
const isDragOver = ref(false)

const columns: DataTableColumns<FileItem> = [
  {
    title: '文件名',
    key: 'name',
    ellipsis: { tooltip: true },
    render(row) {
      return h('div', { class: 'file-name-cell' }, [
        h(NIcon, { size: 20, color: '#18a058' }, { default: () => h(MusicalNotesOutline) }),
        h('span', { style: { marginLeft: '8px' } }, row.name)
      ])
    }
  },
  {
    title: '大小',
    key: 'size',
    width: 100,
    render(row) {
      return formatFileSize(row.size)
    }
  },
  {
    title: '状态',
    key: 'status',
    width: 150,
    render(row) {
      const statusMap: Record<string, { text: string; color: string; icon: any }> = {
        pending: { text: '等待处理', color: '#909399', icon: TimeOutline },
        processing: { text: '处理中', color: '#2080f0', icon: TimeOutline },
        success: { text: '转换成功', color: '#18a058', icon: CheckmarkCircleOutline },
        error: { text: '转换失败', color: '#d03050', icon: CloseCircleOutline }
      }
      const status = statusMap[row.status]
      return h('div', { class: 'status-cell' }, [
        h(NIcon, { size: 18, color: status.color }, { default: () => h(status.icon) }),
        h('span', { style: { marginLeft: '6px', color: status.color } }, status.text)
      ])
    }
  },
  {
    title: '输出格式',
    key: 'outputFormat',
    width: 100,
    render(row) {
      return row.outputFormat ? row.outputFormat.toUpperCase() : '-'
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 80,
    render(row) {
      return h(
        NPopconfirm,
        {
          onPositiveClick: () => removeFile(row.id)
        },
        {
          trigger: () => h(NButton, { size: 'small', quaternary: true, type: 'error' }, {
            default: () => h(NIcon, { size: 18 }, { default: () => h(TrashOutline) })
          }),
          default: () => '确定删除此文件？'
        }
      )
    }
  }
]

const pendingCount = computed(() => fileList.value.filter(f => f.status === 'pending').length)
const successCount = computed(() => fileList.value.filter(f => f.status === 'success').length)
const errorCount = computed(() => fileList.value.filter(f => f.status === 'error').length)

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}

function generateId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

async function selectFiles() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'NCM Files', extensions: ['ncm'] }]
    })
    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected]
      for (const filePath of files) {
        await addFile(filePath as string)
      }
    }
  } catch (e) {
    message.error('选择文件失败: ' + e)
  }
}

async function selectFolder() {
  try {
    const selected = await open({
      directory: true
    })
    if (selected) {
      const files = await invoke<string[]>('scan_ncm_files', { path: selected })
      for (const filePath of files) {
        await addFile(filePath)
      }
      message.success(`找到 ${files.length} 个 NCM 文件`)
    }
  } catch (e) {
    message.error('扫描文件夹失败: ' + e)
  }
}

async function selectOutputDir() {
  try {
    const selected = await open({
      directory: true
    })
    if (selected) {
      outputDir.value = selected as string
      message.success(`输出目录已设置为: ${selected}`)
    }
  } catch (e) {
    message.error('选择输出目录失败: ' + e)
  }
}

async function addFile(filePath: string) {
  const exists = fileList.value.some(f => f.path === filePath)
  if (exists) return
  
  if (!filePath.toLowerCase().endsWith('.ncm')) return
  
  try {
    const info = await invoke<{ name: string; size: number }>('get_file_info', { path: filePath })
    fileList.value.push({
      id: generateId(),
      name: info.name,
      path: filePath,
      size: info.size,
      status: 'pending',
      progress: 0
    })
  } catch (e) {
    message.error(`无法读取文件: ${filePath}`)
  }
}

function removeFile(id: string) {
  const index = fileList.value.findIndex(f => f.id === id)
  if (index !== -1) {
    fileList.value.splice(index, 1)
  }
}

function clearAll() {
  fileList.value = []
}

async function startConversion() {
  if (fileList.value.length === 0) {
    message.warning('请先添加文件')
    return
  }
  
  isProcessing.value = true
  const pendingFiles = fileList.value.filter(f => f.status === 'pending' || f.status === 'error')
  
  for (const file of pendingFiles) {
    file.status = 'processing'
    file.progress = 0
    
    try {
      const result = await invoke<{ format: string; outputPath: string }>('convert_ncm', {
        inputPath: file.path,
        outputDir: outputDir.value || null
      })
      file.status = 'success'
      file.outputFormat = result.format
      file.outputPath = result.outputPath
      file.progress = 100
    } catch (e) {
      file.status = 'error'
      file.errorMsg = String(e)
      message.error(`${file.name} 转换失败: ${e}`)
    }
  }
  
  isProcessing.value = false
  message.success(`转换完成！成功: ${successCount.value}, 失败: ${errorCount.value}`)
}

function handleDragEnter() {
  isDragOver.value = true
}

function handleDragLeave() {
  isDragOver.value = false
}

async function handleFileDrop(payload: unknown) {
  isDragOver.value = false
  
  let paths: string[] = []
  if (Array.isArray(payload)) {
    paths = payload
  } else if (payload && typeof payload === 'object') {
    const p = payload as Record<string, unknown>
    if (Array.isArray(p.paths)) {
      paths = p.paths as string[]
    } else if (Array.isArray(p.uris)) {
      paths = (p.uris as string[]).map(uri => {
        try {
          return decodeURIComponent(uri.replace(/^file:\/\/\//, ''))
        } catch {
          return uri
        }
      })
    }
  }
  
  if (!paths || paths.length === 0) {
    message.warning('未检测到文件')
    return
  }
  
  const ncmPaths = paths.filter(filePath => 
    filePath && filePath.toLowerCase().endsWith('.ncm')
  )
  
  if (ncmPaths.length === 0) {
    message.warning('未找到 NCM 文件，请拖入 .ncm 格式的文件')
    return
  }
  
  let addedCount = 0
  for (const filePath of ncmPaths) {
    const beforeCount = fileList.value.length
    await addFile(filePath)
    if (fileList.value.length > beforeCount) {
      addedCount++
    }
  }
  
  if (addedCount > 0) {
    message.success(`已添加 ${addedCount} 个 NCM 文件`)
  } else if (ncmPaths.length > 0) {
    message.info('所选文件已存在于列表中')
  }
}

let unlistenDragEnter: (() => void) | null = null
let unlistenDragLeave: (() => void) | null = null
let unlistenDragDrop: (() => void) | null = null

async function setupDragDropListeners() {
  try {
    unlistenDragEnter = await listen(TauriEvent.DRAG_ENTER, () => {
      handleDragEnter()
    })
    
    unlistenDragLeave = await listen(TauriEvent.DRAG_LEAVE, () => {
      handleDragLeave()
    })
    
    unlistenDragDrop = await listen(TauriEvent.DRAG_DROP, (event) => {
      handleFileDrop(event.payload)
    })
  } catch (e) {
    console.error('Failed to setup drag drop listeners:', e)
  }
}

function removeDragDropListeners() {
  if (unlistenDragEnter) {
    unlistenDragEnter()
    unlistenDragEnter = null
  }
  if (unlistenDragLeave) {
    unlistenDragLeave()
    unlistenDragLeave = null
  }
  if (unlistenDragDrop) {
    unlistenDragDrop()
    unlistenDragDrop = null
  }
}

onMounted(() => {
  setupDragDropListeners()
})

onUnmounted(() => {
  removeDragDropListeners()
})
</script>

<template>
  <div class="ncm-converter">
    <div class="action-bar" :class="{ dark: isDark }">
      <n-button 
        type="primary" 
        :loading="isProcessing"
        :disabled="fileList.length === 0 || (pendingCount === 0 && fileList.every(f => f.status !== 'error'))"
        @click="startConversion"
      >
        开始转换
      </n-button>
      <n-button quaternary @click="selectOutputDir">
        <template #icon>
          <n-icon :size="18"><FolderOutline /></n-icon>
        </template>
        {{ outputDir ? '更改输出目录' : '选择输出目录' }}
      </n-button>
    </div>
    
    <div class="upload-card" :class="{ dark: isDark }">
      <div 
        class="upload-zone" 
        :class="{ 'drag-over': isDragOver, dark: isDark }"
        @click="selectFiles"
      >
        <div class="upload-content">
          <div class="upload-icon-wrapper">
            <n-icon :size="40" color="#18a058">
              <CloudUploadOutline />
            </n-icon>
          </div>
          <div class="upload-text">
            <p class="upload-title">点击选择文件或拖放 NCM 文件到此处</p>
            <p class="upload-hint">支持批量选择和文件夹扫描</p>
          </div>
          <n-space style="margin-top: 20px;">
            <n-button type="primary" @click.stop="selectFiles">选择文件</n-button>
            <n-button @click.stop="selectFolder">选择文件夹</n-button>
          </n-space>
        </div>
      </div>
    </div>
    
    <div v-if="fileList.length > 0" class="file-section" :class="{ dark: isDark }">
      <div class="section-header">
        <div class="section-info">
          <span class="section-title">文件列表</span>
          <span class="file-count">{{ fileList.length }} 个文件</span>
          <span class="success-count">成功: {{ successCount }}</span>
          <span class="error-count">失败: {{ errorCount }}</span>
        </div>
        <n-button size="small" :disabled="isProcessing" @click="clearAll">
          清空列表
        </n-button>
      </div>
      <n-data-table 
        :columns="columns" 
        :data="fileList" 
        :bordered="false"
        :max-height="400"
        class="file-table"
      />
    </div>
    
    <div v-else class="empty-section" :class="{ dark: isDark }">
      <n-empty description="暂无文件，请添加 NCM 文件" />
    </div>
  </div>
</template>

<style scoped>
.ncm-converter {
  padding: 20px 24px;
  height: 100%;
  overflow-y: auto;
}

.action-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding: 12px 16px;
  border-radius: 8px;
}

.action-bar:not(.dark) {
  background: #ffffff;
  border: 1px solid #e8e8e8;
}

.action-bar.dark {
  background: #252525;
  border: 1px solid #333;
}

.upload-card {
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 24px;
  transition: all 0.3s ease;
}

.upload-card:not(.dark) {
  background: #ffffff;
  border: 1px solid #e8e8e8;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.upload-card.dark {
  background: #252525;
  border: 1px solid #333;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.upload-zone {
  border: 2px dashed;
  border-radius: 8px;
  padding: 48px 32px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s ease;
}

.upload-zone:not(.dark) {
  border-color: #d9d9d9;
  background: #fafafa;
}

.upload-zone.dark {
  border-color: #444;
  background: #2a2a2a;
}

.upload-zone:not(.dark):hover,
.upload-zone:not(.dark).drag-over {
  border-color: #18a058;
  background: #f6ffed;
}

.upload-zone.dark:hover,
.upload-zone.dark.drag-over {
  border-color: #18a058;
  background: #1a2a1f;
}

.upload-zone.drag-over {
  border-style: solid;
  border-width: 3px;
  transform: scale(1.01);
}

.upload-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.upload-content :deep(button) {
  pointer-events: auto;
}

.upload-icon-wrapper {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 16px;
  transition: transform 0.3s ease;
}

.upload-zone:not(.dark) .upload-icon-wrapper {
  background: linear-gradient(135deg, #e6f7ef 0%, #d9f2e8 100%);
}

.upload-zone.dark .upload-icon-wrapper {
  background: linear-gradient(135deg, #1a3a2a 0%, #0d2a1a 100%);
}

.upload-zone.drag-over .upload-icon-wrapper {
  transform: scale(1.1);
}

.upload-text {
  text-align: center;
}

.upload-title {
  font-size: 16px;
  font-weight: 500;
  margin: 0 0 8px 0;
}

.upload-zone:not(.dark) .upload-title {
  color: #333;
}

.upload-zone.dark .upload-title {
  color: #e0e0e0;
}

.upload-hint {
  font-size: 14px;
  margin: 0;
}

.upload-zone:not(.dark) .upload-hint {
  color: #999;
}

.upload-zone.dark .upload-hint {
  color: #888;
}

.file-section {
  border-radius: 12px;
  padding: 20px 24px;
  transition: all 0.3s ease;
}

.file-section:not(.dark) {
  background: #ffffff;
  border: 1px solid #e8e8e8;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.file-section.dark {
  background: #252525;
  border: 1px solid #333;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 16px;
  border-bottom: 1px solid;
}

.file-section:not(.dark) .section-header {
  border-color: #f0f0f0;
}

.file-section.dark .section-header {
  border-color: #333;
}

.section-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
}

.file-section:not(.dark) .section-title {
  color: #1a1a1a;
}

.file-section.dark .section-title {
  color: #e0e0e0;
}

.file-count {
  font-size: 14px;
  padding: 2px 8px;
  border-radius: 4px;
}

.file-section:not(.dark) .file-count {
  color: #666;
  background: #f5f5f5;
}

.file-section.dark .file-count {
  color: #aaa;
  background: #333;
}

.success-count {
  font-size: 14px;
  color: #18a058;
}

.error-count {
  font-size: 14px;
  color: #d03050;
}

.empty-section {
  border-radius: 12px;
  padding: 60px 24px;
  text-align: center;
  transition: all 0.3s ease;
}

.empty-section:not(.dark) {
  background: #ffffff;
  border: 1px solid #e8e8e8;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.empty-section.dark {
  background: #252525;
  border: 1px solid #333;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.file-name-cell {
  display: flex;
  align-items: center;
}

.status-cell {
  display: flex;
  align-items: center;
}
</style>
