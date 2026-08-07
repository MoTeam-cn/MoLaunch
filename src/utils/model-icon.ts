/**
 * 根据模型名称识别品牌，并提供对应的单色 URL 与彩色 SVG 源码。
 * 规则按优先级匹配；无法识别时返回 null，由调用方显示兜底图标。
 */
import { BRAND_ICON_COLOR_RAW, BRAND_ICON_MONO_URLS } from '@/utils/model-brand-icons'

/** 品牌识别规则：[品牌, 正则]，按顺序逐个匹配 */
const BRAND_RULES: Array<[string, RegExp]> = [
  // 具体模型家族 / 品牌优先
  ['CommandA', /\bcommand-a\b/],
  ['GLMV', /\bglmv\b/],
  ['Claude', /\bclaude\b/],
  ['Anthropic', /\banthropic\b/],
  ['Gemma', /\bgemma\b/],
  ['Gemini', /\bgemini\b|\bnano-banana\b|\bnana[-_]?banana\b/],
  ['Grok', /\bgrok\b/],
  ['DeepSeek', /\bdeepseek\b/],
  ['Qwen', /\bqwen[\d.-]*|\btongyi\b/],
  ['Mistral', /\bmistral\b|\bmixtral\b|\bcodestral\b|\bministral\b|\bmathstral\b/],
  ['Cohere', /\bcohere\b|\bcommand[-\s]r\b|\bcoral\b/],
  ['Kimi', /\bkimi\b|\bmoonshot\b/],
  ['Zhipu', /\bglm[-\d\s]|\bchatglm\b|\bzhipu\b/],
  ['Doubao', /\bdoubao\b|\bvolcengine\b|\bseed[-\d]/],
  ['Hunyuan', /\bhunyuan\b|\bhy[-\d]|\btxhunyuan\b/],
  ['Bailian', /\bling[\w.-]*|\bbailian\b/],
  ['XiaomiMiMo', /\bmimo[\w.-]*|\bxiaomi[\w.-]*\b/],
  ['Poolside', /\blaguna\b|\bpoolside\b/],
  ['Baidu', /\bernierne\b|\bwenxin\b|\bbaidu\b/],
  ['Spark', /\bspark\b|\biflytek\b|\bxinghuo\b|\bi\s?fly\s?tek\b/],
  ['Minimax', /\bminimax\b|\babab\b/],
  ['Nvidia', /\bnemotron\b|\bnvidia\b/],
  ['Microsoft', /^phi[\s\d-]|\bwizardlm\b|\bmicrosoft\b|\bphi-[0-9]/],
  ['InternLM', /\binternlm\b/],
  ['Baichuan', /\bbaichuan\b/],
  ['Aya', /\baya[:\-\d]|\baya-expanse\b/],
  ['Dbrx', /\bdbrx\b/],
  ['Rwkv', /\brwkv\b/],
  ['Yi', /\byi[:\-\d]|\byi-(large|medium|small)/],
  ['Dalle', /\bdall-?e\b/],
  ['Midjourney', /\bmidjourney\b/],
  ['Stability', /\bstable-diffusion\b|\bsdxl\b|\bstability\b/],
  ['SenseNova', /\bsensenova\b|\bsense[-\s]?nova\b/],
  ['Skywork', /\bskywork\b/],
  ['Stepfun', /\bstepfun\b|\bstep-\d\b/],
  ['Tencent', /\btencent\b/],
  // 通用 / 开源模型家族
  ['OpenAI', /(^|[/:._-])(gpt|chatgpt|o[1-9])|\bgpt[-\d]|\bdavinci\b|\btext-davinci\b/],
  ['Meta', /\bllama\b|\blama\b/],
  ['HuggingFace', /\bhuggingface\b|^hf[:-]?/],
  // 提供商（放在最后：模型品牌优先，提供商兜底）
  ['SiliconCloud', /\bsilicon\b/],
  ['Vllm', /\bvllm\b/],
  ['Xinference', /\bxinference\b/],
  ['Groq', /\bgroq\b/],
  ['Together', /\btogether\b/],
  ['DeepInfra', /\bdeepinfra\b/],
  ['Fireworks', /\bfireworks\b/],
  ['Novita', /\bnovita\b/],
  ['Hyperbolic', /\bhyperbolic\b/],
  ['OpenRouter', /\bopenrouter\b/],
  ['LmStudio', /\blm-?studio\b/],
  ['Ollama', /\bollama\b/],
  ['Copilot', /\bcopilot\b/],
  ['Cursor', /\bcursor\b/],
  ['Perplexity', /\bperplexity\b/],
]

/** 识别模型名对应的品牌 key（无法识别返回 null） */
export function resolveModelBrand(modelName: string | null | undefined): string | null {
  const name = (modelName ?? '').trim().toLowerCase()
  if (!name) return null
  for (const [brand, rule] of BRAND_RULES) {
    if (rule.test(name)) return brand
  }
  return null
}

/** 识别模型名对应的品牌单色 SVG 图标 URL（无法识别返回 null，由调用方兜底） */
export function resolveModelIconUrl(modelName: string | null | undefined): string | null {
  const brand = resolveModelBrand(modelName)
  if (!brand) return null
  return BRAND_ICON_MONO_URLS[brand] ?? null
}

/** 识别模型名对应的品牌彩色 SVG 源码（`?raw` 内联用；无彩色变体或无品牌时返回 null） */
export function resolveModelColorRaw(modelName: string | null | undefined): string | null {
  const brand = resolveModelBrand(modelName)
  if (!brand) return null
  return BRAND_ICON_COLOR_RAW[brand] ?? null
}
