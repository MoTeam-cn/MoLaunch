/**
 * 沙箱桥接 SDK 注入脚本
 *
 * 此脚本会被注入到外部插件 iframe 的 HTML 末尾，为插件提供全局 `window.molaunch` 对象。
 * 通过 postMessage 与父窗口通信，所有 SDK 调用都转发到父级 pluginSdk 执行。
 *
 * 通信协议：
 * - 子 → 父：{ type: 'request', id: string, method: string, args: unknown[] }
 * - 父 → 子：{ type: 'response', id: string, result?: unknown, error?: string }
 * - 父 → 子：{ type: 'event', name: string, payload?: unknown }
 *
 * 安全设计：
 * - iframe 使用 `sandbox="allow-scripts"`（无 allow-same-origin），无法访问父窗口 DOM / cookie / localStorage
 * - 父级根据 manifest.permissions 白名单决定是否执行请求的方法
 */

/** 注入到 iframe 内的引导脚本（字符串形式，运行于插件 iframe 上下文） */
export const SANDBOX_BOOTSTRAP_SCRIPT = `
(function() {
  'use strict';

  /** 请求 ID 自增计数器 */
  let requestId = 0;

  /** 等待响应的 Promise 映射 */
  const pending = new Map();

  /** 事件监听器映射（事件名 → 回调数组） */
  const listeners = new Map();

  /** 父窗口引用（sandbox iframe 中 parent 仍可访问，只是无法直接读属性） */
  const parentWindow = window.parent;

  /** 安全执行同步函数，捕获异常并打印到控制台（沙箱内联版本，无法 import 父级 safeCallSync） */
  function safeCallSync(fn, label) {
    try { return fn(); } catch (e) { console.error('Failed to ' + label + ':', e); }
  }

  /**
   * 发送 SDK 调用请求到父窗口
   * @param {string} method 方法名
   * @param {unknown[]} args 参数数组
   * @returns {Promise<unknown>}
   */
  function callMethod(method, args) {
    return new Promise((resolve, reject) => {
      const id = 'req_' + (++requestId);
      pending.set(id, { resolve, reject });
      parentWindow.postMessage({
        type: 'request',
        id: id,
        method: method,
        args: args || [],
      }, '*');
    });
  }

  /**
   * 监听父窗口推送的事件
   * @param {string} name 事件名（如 'game-launch'）
   * @param {(payload?: unknown) => void} callback 回调
   */
  function onEvent(name, callback) {
    if (!listeners.has(name)) listeners.set(name, []);
    listeners.get(name).push(callback);
  }

  // 暴露给插件的全局 API
  window.molaunch = {
    // SDK 方法（与 PluginSdk 接口一致）
    getConfig: function() { return callMethod('getConfig', []); },
    listInstalledVersions: function() { return callMethod('listInstalledVersions', []); },
    listInstalledVersionsWithType: function() { return callMethod('listInstalledVersionsWithType', []); },
    listLaunchHistory: function() { return callMethod('listLaunchHistory', []); },
    getSystemMemory: function() { return callMethod('getSystemMemory', []); },
    getRunningGamePid: function() { return callMethod('getRunningGamePid', []); },
    getCacheStats: function() { return callMethod('getCacheStats', []); },

    // spawnProcess（高级权限，父级会注入 pluginId 上下文）
    spawnProcess: function(command, args, options) {
      return callMethod('spawnProcess', [command, args || [], options || {}]);
    },

    // createWindow（高级权限，父级会注入 pluginId 上下文）
    createWindow: function(options) {
      return callMethod('createWindow', [options || {}]);
    },

    // emit / log 始终允许（无敏感数据）
    emit: function(event, payload) {
      if (!event || typeof event !== 'string' || !event.startsWith('plugin:')) {
        console.warn('[Sandbox] emit 事件名必须以 "plugin:" 开头:', event);
        return;
      }
      callMethod('emit', [event, payload]);
    },
    log: function(level, message) {
      callMethod('log', [level, message]);
    },

    // 事件订阅
    onEvent: onEvent,

    // 插件元信息（由父级在加载时注入）
    pluginId: window.__PLUGIN_ID__ || '',
  };

  // 监听父窗口消息
  window.addEventListener('message', function(event) {
    const data = event.data;
    if (!data || typeof data !== 'object') return;

    // 响应消息：解析对应的 pending Promise
    if (data.type === 'response' && typeof data.id === 'string') {
      const p = pending.get(data.id);
      if (p) {
        pending.delete(data.id);
        if (data.error) {
          p.reject(new Error(data.error));
        } else {
          p.resolve(data.result);
        }
      }
      return;
    }

    // 事件消息：分发给对应监听器
    if (data.type === 'event' && typeof data.name === 'string') {
      const cbs = listeners.get(data.name);
      if (cbs) {
        for (const cb of cbs) {
          safeCallSync(function() { cb(data.payload); }, '[Sandbox] run event callback');
        }
      }
      return;
    }
  });

  // 通知父级沙箱已就绪
  parentWindow.postMessage({ type: 'ready' }, '*');
})();
`

/**
 * 构造注入到 iframe 的完整 HTML
 *
 * 把插件原始 HTML 与 bootstrap 脚本拼接，确保 bootstrap 在插件 HTML 之前执行，
 * 这样 window.molaunch 在用户内联脚本运行时已就绪。
 *
 * 同时注入 window.__TAURI_INTERNALS__ 桩，防止 Tauri 2 的内部 IPC 初始化脚本
 * 在 sandbox="allow-scripts"（无 allow-same-origin）的 iframe 中因
 * `window.__TAURI_INTERNALS__` 为 undefined 而抛出
 * "Cannot read properties of undefined (reading 'plugins')" 错误。
 *
 * @param pluginHtml 插件原始 HTML 内容
 * @param pluginId 插件 ID（注入到 window.__PLUGIN_ID__ 供 bootstrap 读取）
 */
export function buildSandboxHtml(pluginHtml: string, pluginId: string): string {
  const pluginIdSafe = JSON.stringify(pluginId)
  const bootstrap = `
<script>
  // 桩 Tauri 内部对象，防止 Tauri 注入的 IPC 脚本在沙箱 iframe 中崩溃
  if (!window.__TAURI_INTERNALS__) {
    window.__TAURI_INTERNALS__ = { plugins: {}, invoke: function() { return Promise.reject(new Error('sandbox: Tauri IPC 不可用')); } };
  }
  window.__PLUGIN_ID__ = ${pluginIdSafe};
</script>
<script>
${SANDBOX_BOOTSTRAP_SCRIPT}
</script>
`
  // 优先在 <head> 开头注入，确保 bootstrap 在用户脚本之前执行
  if (/<head[^>]*>/i.test(pluginHtml)) {
    return pluginHtml.replace(/<head[^>]*>/i, (match) => `${match}${bootstrap}`)
  }
  // 无 <head> 时尝试在 <html> 后注入
  if (/<html[^>]*>/i.test(pluginHtml)) {
    return pluginHtml.replace(/<html[^>]*>/i, (match) => `${match}${bootstrap}`)
  }
  // 无 <html> 时追加到开头
  return bootstrap + pluginHtml
}
