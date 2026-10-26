import { describe, expect, it } from 'vitest'
import { createSSRApp, h } from 'vue'
import { renderToString } from '@vue/server-renderer'
import RecipeItemIcon from '../RecipeItemIcon.vue'
import atlasLayoutRaw from '@/utils/recipe-generator/assets/texture-atlas.json?raw'
import atlasPngUrl from '@/utils/recipe-generator/assets/texture-atlas.png?url'

const atlasLayout = JSON.parse(atlasLayoutRaw) as {
  size: [number, number]
  layout: Record<string, [number, number, number, number]>
}

type IconProps = {
  texture: string | null
  atlasUrl: string
  atlas: { size: [number, number]; layout: Record<string, [number, number, number, number]> }
  size?: number
  label?: string
}

async function render(props: IconProps): Promise<string> {
  const app = createSSRApp({ render: () => h(RecipeItemIcon, props) })
  return renderToString(app)
}

describe('RecipeItemIcon', () => {
  it('should render a sized element with an atlas background when the texture exists', async () => {
    const html = await render({
      texture: 'blocks/stone',
      atlasUrl: atlasPngUrl,
      atlas: atlasLayout,
      size: 30,
      label: '石头（Stone）',
    })
    // 尺寸必须落在元素自身，不能靠 CSS 变量（v-bind 拼表达式会让变量名非法）
    expect(html).toContain('width:30px')
    expect(html).toContain('height:30px')
    expect(html).toContain('background-image:url(')
    expect(html).toContain('background-size:3840px 1110px')
    expect(html).not.toContain('v-bind')
  })

  it('should keep the size and show the initial when the texture is missing', async () => {
    const html = await render({
      texture: 'blocks/not_a_real_block',
      atlasUrl: atlasPngUrl,
      atlas: atlasLayout,
      size: 24,
      label: 'Unknown（未知）',
    })
    expect(html).toContain('width:24px')
    expect(html).toContain('height:24px')
    expect(html).not.toContain('background-image')
    expect(html).toContain('U')
  })

  it('should not throw when the atlas is not ready yet', async () => {
    const html = await render({
      texture: 'blocks/stone',
      atlasUrl: '',
      atlas: { size: [0, 0], layout: {} },
      size: 32,
      label: '石头',
    })
    expect(html).toContain('width:32px')
  })
})
