<script setup lang="ts">
import { ref, watch, provide, type InjectionKey } from 'vue'
import { NConfigProvider, NMessageProvider, NButton, NIcon, darkTheme } from 'naive-ui'
import { MoonOutline, SunnyOutline } from '@vicons/ionicons5'
import Sidebar from './components/Sidebar.vue'
import NcmConverter from './components/NcmConverter.vue'
import AudioConverter from './components/AudioConverter.vue'

interface ThemeContext {
  isDark: { value: boolean }
  toggleTheme: () => void
}

const ThemeKey: InjectionKey<ThemeContext> = Symbol('theme')

const isDark = ref(false)
const activeItem = ref('ncm')

function toggleTheme() {
  isDark.value = !isDark.value
}

watch(isDark, (dark) => {
  document.body.style.backgroundColor = dark ? '#1a1a1a' : '#f5f5f5'
}, { immediate: true })

provide(ThemeKey, { isDark, toggleTheme })
</script>

<template>
  <n-config-provider :theme="isDark ? darkTheme : null">
    <n-message-provider>
      <div class="app-layout" :class="{ dark: isDark }">
        <Sidebar 
          :is-dark="isDark" 
          :active-item="activeItem"
          @select-item="activeItem = $event"
        />
        <div class="main-wrapper">
          <header class="top-bar" :class="{ dark: isDark }">
            <div class="top-bar-content">
              <span class="page-title">
                {{ activeItem === 'ncm' ? 'NCM 解密' : '音频转换' }}
              </span>
              <n-button 
                quaternary 
                circle 
                size="medium"
                class="theme-btn"
                @click="toggleTheme"
              >
                <template #icon>
                  <n-icon :size="20">
                    <SunnyOutline v-if="isDark" />
                    <MoonOutline v-else />
                  </n-icon>
                </template>
              </n-button>
            </div>
          </header>
          <main class="main-content">
            <NcmConverter v-if="activeItem === 'ncm'" :is-dark="isDark" />
            <AudioConverter v-else-if="activeItem === 'audio'" :is-dark="isDark" />
          </main>
        </div>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
}

.app-layout {
  display: flex;
  height: 100%;
  transition: background-color 0.3s ease;
}

.app-layout:not(.dark) {
  background-color: #f5f5f5;
  color: #1a1a1a;
}

.app-layout.dark {
  background-color: #1a1a1a;
  color: #e0e0e0;
}

.main-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.top-bar {
  height: 48px;
  border-bottom: 1px solid;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.top-bar:not(.dark) {
  background: #ffffff;
  border-color: #e8e8e8;
}

.top-bar.dark {
  background: #252525;
  border-color: #333;
}

.top-bar-content {
  width: 100%;
  padding: 0 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.page-title {
  font-size: 15px;
  font-weight: 600;
}

.top-bar:not(.dark) .page-title {
  color: #1a1a1a;
}

.top-bar.dark .page-title {
  color: #e0e0e0;
}

.theme-btn {
  transition: all 0.2s ease;
}

.top-bar:not(.dark) .theme-btn {
  color: #666;
}

.top-bar.dark .theme-btn {
  color: #888;
}

.theme-btn:hover {
  background: rgba(24, 160, 88, 0.1);
}

.main-content {
  flex: 1;
  height: calc(100% - 48px);
  overflow: hidden;
}
</style>
