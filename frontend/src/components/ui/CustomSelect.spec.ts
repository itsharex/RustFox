/**
 * CustomSelect 单测：弹窗最小宽度 / open-close 事件 / 选项 title（长标签悬停可读）。
 */
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import CustomSelect from './CustomSelect.vue'
import { useLocaleStore } from '../../stores/locale'

const OPTIONS = [
  { value: '', label: '默认模块' },
  { value: 'm1', label: '小奏技术·用户服务（很长的模块名会被截断）' },
]

function mountSelect(props: Record<string, unknown> = {}) {
  return mount(CustomSelect, {
    props: { options: OPTIONS, ...props },
    attachTo: document.body,
  })
}

beforeEach(() => {
  setActivePinia(createPinia())
  // 文案断言锁定中文（jsdom 默认语言为英文，跟随系统会解析出英文）
  useLocaleStore().setMode('zh')
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('CustomSelect：下拉行为', () => {
  it('打开菜单触发 open，选择后触发 change/update 与 close', async () => {
    const wrapper = mountSelect({ modelValue: '' })
    await wrapper.find('.cs-trigger').trigger('click')
    await nextTick()
    expect(wrapper.emitted('open')).toHaveLength(1)
    expect(document.querySelector('.cs-pop')).toBeTruthy()

    const opts = Array.from(document.querySelectorAll<HTMLElement>('.cs-opt'))
    expect(opts).toHaveLength(2)
    opts[1].click()
    await nextTick()
    expect(wrapper.emitted('change')?.[0]).toEqual(['m1'])
    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual(['m1'])
    expect(wrapper.emitted('close')).toHaveLength(1)
    expect(document.querySelector('.cs-pop')).toBeNull()
    wrapper.unmount()
  })

  it('默认弹窗与触发器等宽；popMinWidth 撑宽窄触发器', async () => {
    const narrow = mountSelect({ modelValue: '' })
    await narrow.find('.cs-trigger').trigger('click')
    await nextTick()
    // jsdom 无布局，触发器宽为 0：默认等宽即 0
    expect((document.querySelector('.cs-pop') as HTMLElement).style.width).toBe('0px')
    narrow.unmount()
    document.body.innerHTML = ''

    const wide = mountSelect({ modelValue: '', popMinWidth: 220 })
    await wide.find('.cs-trigger').trigger('click')
    await nextTick()
    expect((document.querySelector('.cs-pop') as HTMLElement).style.width).toBe('220px')
    wide.unmount()
  })

  it('选项带 title，截断的长标签悬停可查看全文', async () => {
    const wrapper = mountSelect({ modelValue: '' })
    await wrapper.find('.cs-trigger').trigger('click')
    await nextTick()
    const labels = Array.from(document.querySelectorAll('.cs-opt-label'))
    expect(labels[1].getAttribute('title')).toBe(OPTIONS[1].label)
    wrapper.unmount()
  })
})
