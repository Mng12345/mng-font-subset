import { open } from '@tauri-apps/plugin-dialog';
import type { FontInfo } from '../types';
import './FontUploader.css';

interface FontUploaderProps {
  fontPath: string;
  fontInfo: FontInfo | null;
  selectedFontIndex: number;
  onSelect: (path: string) => void;
  onSelectFontIndex: (index: number) => void;
  disabled?: boolean;
  isLoadingPreview?: boolean;
}

export function FontUploader(props: FontUploaderProps) {
  const handleClick = async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [
        { name: '字体文件', extensions: ['ttf', 'otf', 'woff', 'woff2', 'ttc'] },
        { name: '所有文件', extensions: ['*'] },
      ],
    });

    if (selected && typeof selected === 'string') {
      props.onSelect(selected);
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  return (
    <div class="font-uploader">
      <button class="upload-btn" onClick={handleClick}>
        {props.fontPath ? '更换字体文件' : '选择字体文件'}
      </button>

      {props.fontPath && (
        <div class="font-info">
          <div class="info-row">
            <span class="label">路径:</span>
            <span class="value path">{props.fontPath}</span>
          </div>

          {props.fontInfo && (
            <>
              {props.fontInfo.is_collection && (
                <div class="info-row">
                  <span class="label">字体集合:</span>
                  <span class="value">{props.fontInfo.num_fonts} 个字体</span>
                </div>
              )}

              {!props.fontInfo.is_collection && props.fontInfo.fonts[0] && (
                <>
                  <div class="info-row">
                    <span class="label">字体名称:</span>
                    <span class="value">{props.fontInfo.fonts[0].family_name || '未知'}</span>
                  </div>
                  <div class="info-row">
                    <span class="label">PostScript名称:</span>
                    <span class="value">{props.fontInfo.fonts[0].post_script_name || '未知'}</span>
                  </div>
                  <div class="info-row">
                    <span class="label">字形数量:</span>
                    <span class="value">{props.fontInfo.fonts[0].num_glyphs}</span>
                  </div>
                </>
              )}

              {props.fontInfo.is_collection && (
                <div class="font-collection-list">
                  <div class="info-row">
                    <span class="label">选择字体:</span>
                  </div>
                  {props.isLoadingPreview && (
                    <div class="preview-loading-indicator">
                      <span class="loading-spinner-small"></span>
                      <span>加载字体预览中...</span>
                    </div>
                  )}
                  {props.fontInfo.fonts.map((font) => {
                    const isSelected = props.selectedFontIndex === font.index;
                    return (
                      <div
                        class="collection-item"
                        classList={{ selected: isSelected, disabled: props.disabled }}
                        onClick={() => !props.disabled && props.onSelectFontIndex(font.index)}
                      >
                        <input
                          type="radio"
                          name="font-index"
                          checked={isSelected}
                          disabled={props.disabled}
                          onChange={() => props.onSelectFontIndex(font.index)}
                        />
                        <span class="font-index">[{font.index}]</span>
                        <span class="font-name">{font.family_name || '未知'}</span>
                        <span class="font-glyphs">({font.num_glyphs} 字形)</span>
                      </div>
                    );
                  })}
                </div>
              )}

              <div class="info-row">
                <span class="label">文件大小:</span>
                <span class="value">{formatFileSize(props.fontInfo.file_size)}</span>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}
