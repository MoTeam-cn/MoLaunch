/**
 * 模型品牌 → 官方 SVG 图标映射库（双模式）
 *
 * 图标资源来自 @lobehub/icons-static-svg（官方品牌 SVG 集合），两种变体：
 * - mono：`<brand>.svg`（fill="currentColor"，渲染为黑白单色），经 `?url` 静态打包，
 *   输出到 assets/@lobehub/ 目录（见 vite.config.ts assetFileNames），以 `<img>` 渲染；
 * - color：`<brand>-color.svg`（官方彩色，部分含 SVG 渐变），经 `?raw` 内联注入，
 *   以真实 DOM 渲染（`<img>` 渲染 SVG 时渐变会失效，故彩色必须内联）。
 *
 * 品牌 key 与 model-icon.ts 中 BRAND_RULES 的品牌名保持一致。
 * 个别品牌无官方彩色变体（anthropic / grok / xiaomimimo / midjourney / openai /
 * groq / lmstudio / ollama / cursor），彩色模式下由 ModelIcon 自动退回单色渲染。
 */
import commandaUrl from '@lobehub/icons-static-svg/icons/commanda.svg?url'
import glmvUrl from '@lobehub/icons-static-svg/icons/glmv.svg?url'
import claudeUrl from '@lobehub/icons-static-svg/icons/claude.svg?url'
import anthropicUrl from '@lobehub/icons-static-svg/icons/anthropic.svg?url'
import gemmaUrl from '@lobehub/icons-static-svg/icons/gemma.svg?url'
import geminiUrl from '@lobehub/icons-static-svg/icons/gemini.svg?url'
import grokUrl from '@lobehub/icons-static-svg/icons/grok.svg?url'
import deepseekUrl from '@lobehub/icons-static-svg/icons/deepseek.svg?url'
import qwenUrl from '@lobehub/icons-static-svg/icons/qwen.svg?url'
import mistralUrl from '@lobehub/icons-static-svg/icons/mistral.svg?url'
import cohereUrl from '@lobehub/icons-static-svg/icons/cohere.svg?url'
import kimiUrl from '@lobehub/icons-static-svg/icons/kimi.svg?url'
import zhipuUrl from '@lobehub/icons-static-svg/icons/zhipu.svg?url'
import doubaoUrl from '@lobehub/icons-static-svg/icons/doubao.svg?url'
import hunyuanUrl from '@lobehub/icons-static-svg/icons/hunyuan.svg?url'
import bailianUrl from '@lobehub/icons-static-svg/icons/bailian.svg?url'
import xiaomimimoUrl from '@lobehub/icons-static-svg/icons/xiaomimimo.svg?url'
import poolsideUrl from '@lobehub/icons-static-svg/icons/poolside.svg?url'
import baiduUrl from '@lobehub/icons-static-svg/icons/baidu.svg?url'
import sparkUrl from '@lobehub/icons-static-svg/icons/spark.svg?url'
import minimaxUrl from '@lobehub/icons-static-svg/icons/minimax.svg?url'
import nvidiaUrl from '@lobehub/icons-static-svg/icons/nvidia.svg?url'
import microsoftUrl from '@lobehub/icons-static-svg/icons/microsoft.svg?url'
import internlmUrl from '@lobehub/icons-static-svg/icons/internlm.svg?url'
import baichuanUrl from '@lobehub/icons-static-svg/icons/baichuan.svg?url'
import ayaUrl from '@lobehub/icons-static-svg/icons/aya.svg?url'
import dbrxUrl from '@lobehub/icons-static-svg/icons/dbrx.svg?url'
import rwkvUrl from '@lobehub/icons-static-svg/icons/rwkv.svg?url'
import yiUrl from '@lobehub/icons-static-svg/icons/yi.svg?url'
import dalleUrl from '@lobehub/icons-static-svg/icons/dalle.svg?url'
import midjourneyUrl from '@lobehub/icons-static-svg/icons/midjourney.svg?url'
import stabilityUrl from '@lobehub/icons-static-svg/icons/stability.svg?url'
import sensenovaUrl from '@lobehub/icons-static-svg/icons/sensenova.svg?url'
import skyworkUrl from '@lobehub/icons-static-svg/icons/skywork.svg?url'
import stepfunUrl from '@lobehub/icons-static-svg/icons/stepfun.svg?url'
import tencentUrl from '@lobehub/icons-static-svg/icons/tencent.svg?url'
import openaiUrl from '@lobehub/icons-static-svg/icons/openai.svg?url'
import metaUrl from '@lobehub/icons-static-svg/icons/meta.svg?url'
import huggingfaceUrl from '@lobehub/icons-static-svg/icons/huggingface.svg?url'
import siliconcloudUrl from '@lobehub/icons-static-svg/icons/siliconcloud.svg?url'
import vllmUrl from '@lobehub/icons-static-svg/icons/vllm.svg?url'
import xinferenceUrl from '@lobehub/icons-static-svg/icons/xinference.svg?url'
import groqUrl from '@lobehub/icons-static-svg/icons/groq.svg?url'
import togetherUrl from '@lobehub/icons-static-svg/icons/together.svg?url'
import deepinfraUrl from '@lobehub/icons-static-svg/icons/deepinfra.svg?url'
import fireworksUrl from '@lobehub/icons-static-svg/icons/fireworks.svg?url'
import novitaUrl from '@lobehub/icons-static-svg/icons/novita.svg?url'
import hyperbolicUrl from '@lobehub/icons-static-svg/icons/hyperbolic.svg?url'
import openrouterUrl from '@lobehub/icons-static-svg/icons/openrouter.svg?url'
import lmstudioUrl from '@lobehub/icons-static-svg/icons/lmstudio.svg?url'
import ollamaUrl from '@lobehub/icons-static-svg/icons/ollama.svg?url'
import copilotUrl from '@lobehub/icons-static-svg/icons/copilot.svg?url'
import cursorUrl from '@lobehub/icons-static-svg/icons/cursor.svg?url'
import perplexityUrl from '@lobehub/icons-static-svg/icons/perplexity.svg?url'

import commandaColorRaw from '@lobehub/icons-static-svg/icons/commanda-color.svg?raw'
import glmvColorRaw from '@lobehub/icons-static-svg/icons/glmv-color.svg?raw'
import claudeColorRaw from '@lobehub/icons-static-svg/icons/claude-color.svg?raw'
import gemmaColorRaw from '@lobehub/icons-static-svg/icons/gemma-color.svg?raw'
import geminiColorRaw from '@lobehub/icons-static-svg/icons/gemini-color.svg?raw'
import deepseekColorRaw from '@lobehub/icons-static-svg/icons/deepseek-color.svg?raw'
import qwenColorRaw from '@lobehub/icons-static-svg/icons/qwen-color.svg?raw'
import mistralColorRaw from '@lobehub/icons-static-svg/icons/mistral-color.svg?raw'
import cohereColorRaw from '@lobehub/icons-static-svg/icons/cohere-color.svg?raw'
import zhipuColorRaw from '@lobehub/icons-static-svg/icons/zhipu-color.svg?raw'
import doubaoColorRaw from '@lobehub/icons-static-svg/icons/doubao-color.svg?raw'
import hunyuanColorRaw from '@lobehub/icons-static-svg/icons/hunyuan-color.svg?raw'
import bailianColorRaw from '@lobehub/icons-static-svg/icons/bailian-color.svg?raw'
import poolsideColorRaw from '@lobehub/icons-static-svg/icons/poolside-color.svg?raw'
import baiduColorRaw from '@lobehub/icons-static-svg/icons/baidu-color.svg?raw'
import sparkColorRaw from '@lobehub/icons-static-svg/icons/spark-color.svg?raw'
import minimaxColorRaw from '@lobehub/icons-static-svg/icons/minimax-color.svg?raw'
import nvidiaColorRaw from '@lobehub/icons-static-svg/icons/nvidia-color.svg?raw'
import microsoftColorRaw from '@lobehub/icons-static-svg/icons/microsoft-color.svg?raw'
import internlmColorRaw from '@lobehub/icons-static-svg/icons/internlm-color.svg?raw'
import baichuanColorRaw from '@lobehub/icons-static-svg/icons/baichuan-color.svg?raw'
import ayaColorRaw from '@lobehub/icons-static-svg/icons/aya-color.svg?raw'
import dbrxColorRaw from '@lobehub/icons-static-svg/icons/dbrx-color.svg?raw'
import rwkvColorRaw from '@lobehub/icons-static-svg/icons/rwkv-color.svg?raw'
import yiColorRaw from '@lobehub/icons-static-svg/icons/yi-color.svg?raw'
import dalleColorRaw from '@lobehub/icons-static-svg/icons/dalle-color.svg?raw'
import stabilityColorRaw from '@lobehub/icons-static-svg/icons/stability-color.svg?raw'
import sensenovaColorRaw from '@lobehub/icons-static-svg/icons/sensenova-color.svg?raw'
import skyworkColorRaw from '@lobehub/icons-static-svg/icons/skywork-color.svg?raw'
import stepfunColorRaw from '@lobehub/icons-static-svg/icons/stepfun-color.svg?raw'
import tencentColorRaw from '@lobehub/icons-static-svg/icons/tencent-color.svg?raw'
import metaColorRaw from '@lobehub/icons-static-svg/icons/meta-color.svg?raw'
import huggingfaceColorRaw from '@lobehub/icons-static-svg/icons/huggingface-color.svg?raw'
import siliconcloudColorRaw from '@lobehub/icons-static-svg/icons/siliconcloud-color.svg?raw'
import vllmColorRaw from '@lobehub/icons-static-svg/icons/vllm-color.svg?raw'
import xinferenceColorRaw from '@lobehub/icons-static-svg/icons/xinference-color.svg?raw'
import togetherColorRaw from '@lobehub/icons-static-svg/icons/together-color.svg?raw'
import deepinfraColorRaw from '@lobehub/icons-static-svg/icons/deepinfra-color.svg?raw'
import fireworksColorRaw from '@lobehub/icons-static-svg/icons/fireworks-color.svg?raw'
import novitaColorRaw from '@lobehub/icons-static-svg/icons/novita-color.svg?raw'
import hyperbolicColorRaw from '@lobehub/icons-static-svg/icons/hyperbolic-color.svg?raw'
import openrouterColorRaw from '@lobehub/icons-static-svg/icons/openrouter-color.svg?raw'
import copilotColorRaw from '@lobehub/icons-static-svg/icons/copilot-color.svg?raw'
import perplexityColorRaw from '@lobehub/icons-static-svg/icons/perplexity-color.svg?raw'

/** 品牌 → 单色 SVG URL（官方 mono 变体，fill=currentColor，`<img>` 渲染为黑色） */
export const BRAND_ICON_MONO_URLS: Record<string, string> = {
  CommandA: commandaUrl,
  GLMV: glmvUrl,
  Claude: claudeUrl,
  Anthropic: anthropicUrl,
  Gemma: gemmaUrl,
  Gemini: geminiUrl,
  Grok: grokUrl,
  DeepSeek: deepseekUrl,
  Qwen: qwenUrl,
  Mistral: mistralUrl,
  Cohere: cohereUrl,
  Kimi: kimiUrl,
  Zhipu: zhipuUrl,
  Doubao: doubaoUrl,
  Hunyuan: hunyuanUrl,
  Bailian: bailianUrl,
  XiaomiMiMo: xiaomimimoUrl,
  Poolside: poolsideUrl,
  Baidu: baiduUrl,
  Spark: sparkUrl,
  Minimax: minimaxUrl,
  Nvidia: nvidiaUrl,
  Microsoft: microsoftUrl,
  InternLM: internlmUrl,
  Baichuan: baichuanUrl,
  Aya: ayaUrl,
  Dbrx: dbrxUrl,
  Rwkv: rwkvUrl,
  Yi: yiUrl,
  Dalle: dalleUrl,
  Midjourney: midjourneyUrl,
  Stability: stabilityUrl,
  SenseNova: sensenovaUrl,
  Skywork: skyworkUrl,
  Stepfun: stepfunUrl,
  Tencent: tencentUrl,
  OpenAI: openaiUrl,
  Meta: metaUrl,
  HuggingFace: huggingfaceUrl,
  SiliconCloud: siliconcloudUrl,
  Vllm: vllmUrl,
  Xinference: xinferenceUrl,
  Groq: groqUrl,
  Together: togetherUrl,
  DeepInfra: deepinfraUrl,
  Fireworks: fireworksUrl,
  Novita: novitaUrl,
  Hyperbolic: hyperbolicUrl,
  OpenRouter: openrouterUrl,
  LmStudio: lmstudioUrl,
  Ollama: ollamaUrl,
  Copilot: copilotUrl,
  Cursor: cursorUrl,
  Perplexity: perplexityUrl,
}

/** 品牌 → 官方彩色 SVG 源码（`?raw` 内联注入，避免 `<img>` 渲染 SVG 渐变失效） */
export const BRAND_ICON_COLOR_RAW: Record<string, string> = {
  CommandA: commandaColorRaw,
  GLMV: glmvColorRaw,
  Claude: claudeColorRaw,
  Gemma: gemmaColorRaw,
  Gemini: geminiColorRaw,
  DeepSeek: deepseekColorRaw,
  Qwen: qwenColorRaw,
  Mistral: mistralColorRaw,
  Cohere: cohereColorRaw,
  Zhipu: zhipuColorRaw,
  Doubao: doubaoColorRaw,
  Hunyuan: hunyuanColorRaw,
  Bailian: bailianColorRaw,
  Poolside: poolsideColorRaw,
  Baidu: baiduColorRaw,
  Spark: sparkColorRaw,
  Minimax: minimaxColorRaw,
  Nvidia: nvidiaColorRaw,
  Microsoft: microsoftColorRaw,
  InternLM: internlmColorRaw,
  Baichuan: baichuanColorRaw,
  Aya: ayaColorRaw,
  Dbrx: dbrxColorRaw,
  Rwkv: rwkvColorRaw,
  Yi: yiColorRaw,
  Dalle: dalleColorRaw,
  Stability: stabilityColorRaw,
  SenseNova: sensenovaColorRaw,
  Skywork: skyworkColorRaw,
  Stepfun: stepfunColorRaw,
  Tencent: tencentColorRaw,
  Meta: metaColorRaw,
  HuggingFace: huggingfaceColorRaw,
  SiliconCloud: siliconcloudColorRaw,
  Vllm: vllmColorRaw,
  Xinference: xinferenceColorRaw,
  Together: togetherColorRaw,
  DeepInfra: deepinfraColorRaw,
  Fireworks: fireworksColorRaw,
  Novita: novitaColorRaw,
  Hyperbolic: hyperbolicColorRaw,
  OpenRouter: openrouterColorRaw,
  Copilot: copilotColorRaw,
  Perplexity: perplexityColorRaw,
}

/** 全局兜底图标（未识别到品牌时使用 HuggingFace）：单色 URL + 彩色源码 */
export const HUGGINGFACE_MONO_URL = huggingfaceUrl
export const HUGGINGFACE_COLOR_RAW = huggingfaceColorRaw
