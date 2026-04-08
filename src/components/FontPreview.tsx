import { createEffect, createSignal, Show, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import './FontPreview.css';

// 辅助函数：将日志发送到终端
function logToTerminal(message: string) {
  invoke('log_to_terminal', { message }).catch(() => {
    // 忽略错误，降级到 console
  });
}

interface FontPreviewProps {
  fontPath: string;
  fontIndex: number;
  text: string;
  uniqueChars: string[];
  onLoadingChange?: (loading: boolean) => void;
}

interface Deps {
  fontPath: string;
  fontIndex: number;
}

export function FontPreview(props: FontPreviewProps) {
  const [fontUrl, setFontUrl] = createSignal<string>('');
  const [isLoading, setIsLoading] = createSignal(false);
  const [currentFontFamily, setCurrentFontFamily] = createSignal<string>('PreviewFont-initial');

  // 使用 onCleanup 来管理清理逻辑
  let prevCleanup: (() => void) | null = null;

  // 通知父组件加载状态变化
  const notifyLoadingChange = (loading: boolean) => {
    props.onLoadingChange?.(loading);
  };

  createEffect((prevDeps: Deps | null) => {
    // 读取追踪的依赖
    const fontPath = props.fontPath;
    const fontIndex = props.fontIndex;

    // 比较依赖是否真正变化
    const depsChanged = !prevDeps || prevDeps.fontPath !== fontPath || prevDeps.fontIndex !== fontIndex;

    if (!depsChanged) {
      // 依赖没有变化，返回之前的依赖，不执行加载
      return prevDeps;
    }

    // 执行之前的清理函数 - 立即中断之前的加载
    if (prevCleanup) {
      prevCleanup();
      prevCleanup = null;
    }

    if (fontPath) {
      // 生成唯一的 font-family 名称，使用 fontPath 和 fontIndex 的组合
      // 这样只有在真正切换字体或子字体时才会变化
      const fontFamily = `PreviewFont-${fontIndex}-${fontPath.replace(/[^a-zA-Z0-9]/g, '_')}`;
      setCurrentFontFamily(fontFamily);
      setIsLoading(true);
      notifyLoadingChange(true);

      let cancelled = false;

      const loadFontData = async () => {
        const startTime = performance.now();
        try {
          // 调用 Rust 后端获取字体数据（支持 TTC 指定索引）
          // 注意：Tauri 的 invoke 无法直接中断，我们通过 cancelled 标志来忽略结果
          const fontData = await invoke<number[]>('get_font_data_for_preview', {
            fontPath,
            fontIndex,
          });

          // 如果已被取消，不更新状态
          if (cancelled) {
            return;
          }

          const endTime = performance.now();
          logToTerminal(`[FontPreview] 加载字体数据 #${fontIndex}: ${(endTime - startTime).toFixed(0)}ms`);

          // 将字节数组转换为 Blob URL
          const uint8Array = new Uint8Array(fontData);
          const blob = new Blob([uint8Array], { type: 'font/ttf' });
          const url = URL.createObjectURL(blob);
          setFontUrl(url);
          setIsLoading(false);
          notifyLoadingChange(false);
        } catch (err) {
          // 如果是取消导致的错误，静默处理
          if (cancelled) {
            return;
          }
          console.error('加载字体预览失败:', err);
          // 回退到 file:// 协议
          setFontUrl(`file://${fontPath}`);
          setIsLoading(false);
          notifyLoadingChange(false);
        }
      };

      // 使用 requestIdleCallback 延迟加载字体，让 UI 先更新
      const idleCallbackId = requestIdleCallback(() => {
        if (!cancelled) {
          loadFontData();
        }
      });

      // 设置清理函数
      prevCleanup = () => {
        cancelled = true;
        cancelIdleCallback(idleCallbackId);
        const currentUrl = fontUrl();
        if (currentUrl && currentUrl.startsWith('blob:')) {
          URL.revokeObjectURL(currentUrl);
        }
      };
    }

    // 返回当前依赖，用于下次比较
    return { fontPath, fontIndex };
  }, null);

  // 组件卸载时执行清理
  onCleanup(() => {
    if (prevCleanup) {
      prevCleanup();
    }
  });

  return (
    <div class="font-preview">
      <div class="preview-section">
        <h3>文本预览</h3>
        <div
          class="preview-text"
          classList={{ 'loading': isLoading() }}
          style={{
            'font-family': fontUrl() ? `"${currentFontFamily()}", sans-serif` : 'sans-serif',
          }}
        >
          <Show when={!isLoading()}>
            <style>{`
              @font-face {
                font-family: '${currentFontFamily()}';
                src: url('${fontUrl()}');
              }
            `}</style>
          </Show>
          <Show when={isLoading()}>
            <div class="font-loading-indicator">
              <span class="loading-spinner-small"></span>
              <span>加载字体中...</span>
            </div>
          </Show>
          <Show when={!isLoading()}>
            {props.text || '（无预览文本）'}
          </Show>
        </div>
      </div>

      <div class="preview-section">
        <h3>唯一字符列表 ({props.uniqueChars.length} 个)</h3>
        <div class="char-list">
          <Show when={props.uniqueChars.length > 0}>
            {props.uniqueChars.map((char) => (
              <span class="char-item" title={`U+${char.charCodeAt(0).toString(16).toUpperCase().padStart(4, '0')}`}>
                {char}
              </span>
            ))}
          </Show>
        </div>
      </div>
    </div>
  );
}
