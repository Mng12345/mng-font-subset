import { invoke } from '@tauri-apps/api/core';
import type { ExtractResult } from '../types';
import './ResultDisplay.css';

interface ResultDisplayProps {
  result: ExtractResult;
}

export function ResultDisplay(props: ResultDisplayProps) {
  const handleOpenFolder = async () => {
    const outputPath = props.result.output_path;
    if (!outputPath) {
      console.error('[ResultDisplay] output_path is empty');
      return;
    }

    console.log('[ResultDisplay] Opening folder for path:', outputPath);

    try {
      // 使用 Rust 后端命令打开文件夹，更可靠
      await invoke('open_folder', { path: outputPath });
      console.log('[ResultDisplay] Folder opened successfully');
    } catch (error) {
      console.error('[ResultDisplay] Failed to open folder:', error);
    }
  };

  return (
    <div class={`result-display ${props.result.success ? 'success' : 'error'}`}>
      <div class="result-icon">{props.result.success ? '✓' : '✗'}</div>
      <div class="result-message">{props.result.message}</div>
      {props.result.output_path && (
        <>
          <div class="result-path">
            保存位置: <code>{props.result.output_path}</code>
          </div>
          {props.result.success && (
            <button class="open-folder-btn" onClick={handleOpenFolder}>
              打开所在文件夹
            </button>
          )}
        </>
      )}
    </div>
  );
}
