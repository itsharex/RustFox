/**
 * EnvironmentManager 新建环境回归：
 * - 新建环境后未做任何编辑，直接关闭不得弹出「未保存修改」确认——
 *   创建本身不算修改（历史 bug：点击新建后关闭必弹确认）；
 *   未编辑的新环境从未落库，随关闭静默丢弃。
 * - 一旦实际编辑（如改名称），关闭必须弹出确认条且弹窗保持打开。
 * - 编辑后保存，关闭同样不弹确认（保存即清脏）。
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import EnvironmentManager from './EnvironmentManager.vue'
import { useLocaleStore } from '../stores/locale'
import type { Environment } from '../types/foxApi'

const ENVS: Environment[] = [
  {
    id: 'e-dev',
    name: '开发环境',
    modules: [{ id: 'm1', module_name: '默认', base_url: 'https://api.dev.example.com', is_default: true }],
    variables: [],
    created_at: '',
    updated_at: '',
  },
]

const updateEnvironment = vi.fn(async (env: Environment) => ({ ...env }))

vi.mock('../stores/workspace', () => ({
  useWorkspaceStore: () => ({
    environments: ENVS,
    activeEnvId: 'e-dev',
    project: { id: 'p1' },
    globalVariables: [],
    globalParams: [],
    updateEnvironment,
  }),
}))

vi.mock('../composables/useToast', () => ({
  useToast: () => ({ success: vi.fn(), error: vi.fn(), info: vi.fn(), warning: vi.fn() }),
}))

vi.mock('../composables/useFoxApi', () => ({
  useFoxApi: () => new Proxy({}, { get: () => vi.fn() }),
}))

function $(sel: string): Element | null {
  return document.querySelector(sel)
}

async function mountDlg() {
  const wrapper = mount(EnvironmentManager, { props: { open: false }, attachTo: document.body })
  // 打开 watch 只在 open 变化时触发：挂载后再置 true
  await wrapper.setProps({ open: true })
  await nextTick()
  return wrapper
}

async function clickAddEnv() {
  const btn = $('.em-add')
  expect(btn).toBeTruthy()
  ;(btn as HTMLElement).click()
  await nextTick()
}

async function clickClose() {
  const btn = document.querySelector<HTMLButtonElement>('.m-head button.ib')
  expect(btn).toBeTruthy()
  btn!.click()
  await nextTick()
}

function confirmShown(): boolean {
  return $('.em-confirm') != null
}

beforeEach(() => {
  setActivePinia(createPinia())
  useLocaleStore().setMode('zh')
  updateEnvironment.mockClear()
  document.body.innerHTML = ''
})

describe('EnvironmentManager 新建环境未保存确认', () => {
  it('新建后未编辑：直接关闭不弹确认，弹窗正常关闭', async () => {
    const wrapper = await mountDlg()
    await clickAddEnv()
    expect(confirmShown()).toBe(false)
    expect(document.body.textContent).toContain('新环境')

    await clickClose()
    expect(confirmShown()).toBe(false)
    expect(wrapper.emitted('update:open')?.at(-1)).toEqual([false])
  })

  it('新建后编辑名称：关闭弹出确认，弹窗保持打开', async () => {
    const wrapper = await mountDlg()
    await clickAddEnv()

    const name = document.querySelector<HTMLInputElement>('.em-name')
    expect(name).toBeTruthy()
    name!.value = '预发环境'
    name!.dispatchEvent(new Event('input'))
    await nextTick()

    await clickClose()
    expect(confirmShown()).toBe(true)
    expect(wrapper.emitted('update:open')).toBeUndefined()
  })

  it('新建后保存：关闭不弹确认', async () => {
    const wrapper = await mountDlg()
    await clickAddEnv()

    const save = document.querySelector<HTMLButtonElement>('.m-foot .rf-btn-primary')
    expect(save).toBeTruthy()
    ;(save as HTMLButtonElement).click()
    await flushPromises()
    expect(updateEnvironment).toHaveBeenCalledTimes(1)

    await clickClose()
    expect(confirmShown()).toBe(false)
    expect(wrapper.emitted('update:open')?.at(-1)).toEqual([false])
  })
})
