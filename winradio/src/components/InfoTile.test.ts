import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import InfoTile from './InfoTile.vue'

// First component-level test in the repo (epic-1-retro item 3) — proves the
// @vue/test-utils + jsdom setup actually mounts and renders a real
// component, not just exercises pure store logic.
describe('InfoTile', () => {
  it('renders the default slot when no placeholder is set', () => {
    const wrapper = mount(InfoTile, {
      props: { title: 'Location' },
      slots: { default: '<p>Belgium</p>' },
    })

    expect(wrapper.text()).toContain('Location')
    expect(wrapper.text()).toContain('Belgium')
  })

  it('renders the placeholder instead of the slot when set (degraded state, same shell)', () => {
    const wrapper = mount(InfoTile, {
      props: { title: 'Location', placeholder: 'Location unknown' },
      slots: { default: '<p>Belgium</p>' },
    })

    expect(wrapper.text()).toContain('Location unknown')
    expect(wrapper.text()).not.toContain('Belgium')
  })
})
