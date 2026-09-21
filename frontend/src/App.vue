/**
 * 根组件：全局一次性的 ToastHost（useToast 通知）与 ProgressBar（useProgress 顶部进度条）。
 * 视图通过 <router-view /> 渲染。
 * 另挂全局 error/unhandledrejection 监听：把未捕获的异常通过 console + toast 暴露，
 * 便于在没有 DevTools 的情况下定位前端问题。
 * 监听 macOS 原生菜单「About RustFox」事件以打开自定义关于弹窗。
 */
<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import ToastHost from './components/ToastHost.vue'
import ProgressBar from './components/ProgressBar.vue'
import Brand from './components/Brand.vue'
import AboutDialog from './components/AboutDialog.vue'
import { useToast } from './composables/useToast'
import { useLocaleStore } from './stores/locale'
import { startAutoUpdate, openAboutSignal } from './composables/useAutoUpdate'

const toast = useToast()
const locale = useLocaleStore()
const t = locale.t
const route = useRoute()

/**
 * 悬浮品牌：仅在无路由匹配时展示。四个正式视图（项目列表 / 工作区 /
 * GraphQL / 实时调试）自带顶栏，在此叠加浮层会盖住返回按钮等头部控件。
 */
const showFloatingBrand = computed(() => route.matched.length === 0)
const showAbout = ref(false)
let unlistenAbout: UnlistenFn | null = null
let stopAutoUpdate: (() => void) | null = null

/** 设置更新行 → 打开关于弹窗（一键下载安装）。 */
watch(openAboutSignal, () => {
  showAbout.value = true
})

onMounted(async () => {
  window.addEventListener('error', (event) => {
    console.error('[window.error]', event.message, event.error)
    const msg = String(event.error?.message ?? event.message)
    toast.error(t('app.pageError'), { message: msg, duration: 6000 })
  })
  window.addEventListener('unhandledrejection', (event) => {
    const reason = event.reason
    console.error('[unhandledrejection]', reason)
    const msg = reason instanceof Error ? reason.message : String(reason)
    toast.error(t('app.unhandledError'), { message: msg, duration: 6000 })
  })
  try {
    if ('__TAURI_INTERNALS__' in window) {
      unlistenAbout = await listen('rustfox://about', () => {
        showAbout.value = true
      })
      // 定时检查更新：仅主窗口启动（实时调试弹出窗不重复检查），
      // 发现新版弹一次提醒，点击直达关于弹窗一键下载安装。
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      if (getCurrentWindow().label === 'main') {
        stopAutoUpdate = startAutoUpdate({
          onUpdateAvailable: ({ version }) => {
            toast.info(t('app.updateFound', { v: version }), {
              duration: 15000,
              action: {
                label: t('app.viewDetails'),
                run: () => {
                  showAbout.value = true
                },
              },
            })
          },
        })
      }
    }
  } catch {
    // 非 Tauri（浏览器预览）环境：忽略
  }
})

onBeforeUnmount(() => {
  unlistenAbout?.()
  stopAutoUpdate?.()
})
</script>

<template>
  <div v-if="showFloatingBrand" class="app-brand" :aria-label="t('app.brandAria')">
    <Brand title="RustFox" :subtitle="t('app.tagline')" />
  </div>
  <ProgressBar />
  <ToastHost />
  <AboutDialog v-model:open="showAbout" />
  <main class="rf-app">
    <router-view />
  </main>
</template>

<style scoped>
.app-brand {
  position: fixed;
  top: 8px;
  left: 8px;
  z-index: 60;
  padding: 2px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow);
}
.app-brand :deep(.brand) {
  min-width: 0;
  width: 132px;
}
</style>