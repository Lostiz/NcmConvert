<script setup lang="ts">
import { ref } from 'vue'
import { NIcon, NButton, NTooltip } from 'naive-ui'
import {
  MusicalNotesOutline,
  SwapHorizontalOutline,
  ChevronBackOutline,
  ChevronForwardOutline
} from '@vicons/ionicons5'

interface MenuItem {
  id: string
  label: string
  icon: any
}

defineProps<{
  isDark: boolean
  activeItem: string
}>()

const emit = defineEmits<{
  (e: 'selectItem', id: string): void
  (e: 'toggleCollapse'): void
}>()

const isCollapsed = ref(false)

const menuItems = ref<MenuItem[]>([
  { id: 'ncm', label: 'NCM 解密', icon: MusicalNotesOutline },
  { id: 'audio', label: '音频转换', icon: SwapHorizontalOutline }
])

function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
  emit('toggleCollapse')
}

function selectItem(itemId: string) {
  emit('selectItem', itemId)
}
</script>

<template>
  <div class="sidebar" :class="{ dark: isDark, collapsed: isCollapsed }">
    <div class="sidebar-header">
      <div class="logo">
        <n-icon :size="24" color="#18a058">
          <MusicalNotesOutline />
        </n-icon>
        <span v-if="!isCollapsed" class="logo-text">NcmConvert</span>
      </div>
      <n-button 
        quaternary 
        circle 
        size="small"
        class="collapse-btn"
        @click="toggleCollapse"
      >
        <template #icon>
          <n-icon :size="16">
            <ChevronBackOutline v-if="!isCollapsed" />
            <ChevronForwardOutline v-else />
          </n-icon>
        </template>
      </n-button>
    </div>
    
    <div class="sidebar-menu">
      <n-tooltip 
        v-for="item in menuItems" 
        :key="item.id"
        :disabled="!isCollapsed"
        placement="right"
        :delay="300"
      >
        <template #trigger>
          <div
            class="menu-item"
            :class="{ active: activeItem === item.id }"
            @click="selectItem(item.id)"
          >
            <n-icon :size="20">
              <component :is="item.icon" />
            </n-icon>
            <span v-if="!isCollapsed" class="item-label">{{ item.label }}</span>
          </div>
        </template>
        {{ item.label }}
      </n-tooltip>
    </div>
  </div>
</template>

<style scoped>
.sidebar {
  width: 220px;
  height: 100%;
  display: flex;
  flex-direction: column;
  border-right: 1px solid;
  transition: width 0.3s ease;
  overflow: hidden;
}

.sidebar.collapsed {
  width: 56px;
}

.sidebar:not(.dark) {
  background: #fafafa;
  border-color: #e8e8e8;
}

.sidebar.dark {
  background: #1f1f1f;
  border-color: #333;
}

.sidebar-header {
  padding: 16px 12px;
  border-bottom: 1px solid;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 56px;
}

.sidebar:not(.dark) .sidebar-header {
  border-color: #e8e8e8;
}

.sidebar.dark .sidebar-header {
  border-color: #333;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.logo-text {
  font-size: 15px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
}

.sidebar:not(.dark) .logo-text {
  color: #1a1a1a;
}

.sidebar.dark .logo-text {
  color: #e0e0e0;
}

.collapse-btn {
  flex-shrink: 0;
}

.sidebar:not(.dark) .collapse-btn {
  color: #666;
}

.sidebar.dark .collapse-btn {
  color: #888;
}

.sidebar-menu {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  border-radius: 8px;
  margin-bottom: 4px;
  border-left: 3px solid transparent;
  min-height: 44px;
}

.sidebar.collapsed .menu-item {
  justify-content: center;
  padding: 12px 0;
  border-left: none;
}

.menu-item:hover {
  background: rgba(24, 160, 88, 0.08);
}

.menu-item.active {
  background: rgba(24, 160, 88, 0.12);
  border-left-color: #18a058;
}

.sidebar.collapsed .menu-item.active {
  border-left: none;
  background: rgba(24, 160, 88, 0.15);
}

.sidebar.dark .menu-item.active {
  background: rgba(24, 160, 88, 0.15);
}

.item-label {
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
}

.sidebar:not(.dark) .item-label {
  color: #333;
}

.sidebar.dark .item-label {
  color: #ccc;
}

.menu-item.active .item-label {
  color: #18a058;
  font-weight: 500;
}

.sidebar:not(.dark) .menu-item :deep(.n-icon) {
  color: #666;
}

.sidebar.dark .menu-item :deep(.n-icon) {
  color: #888;
}

.menu-item.active :deep(.n-icon) {
  color: #18a058;
}
</style>
