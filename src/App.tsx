import { createSignal, createMemo, Show } from 'solid-js';
import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { FontUploader } from './components/FontUploader';
import { TextInput } from './components/TextInput';
import { FontPreview } from './components/FontPreview';
import { ResultDisplay } from './components/ResultDisplay';
import type { FontInfo, ExtractResult } from './types';
import './styles/App.css';

function App() {
  const [fontPath, setFontPath] = createSignal<string>('');
  const [fontInfo, setFontInfo] = createSignal<FontInfo | null>(null);
  const [selectedFontIndex, setSelectedFontIndex] = createSignal<number>(0);
  const [text, setText] = createSignal<string>('');
  const [isExtracting, setIsExtracting] = createSignal(false);
  const [isLoadingFont, setIsLoadingFont] = createSignal(false);
  const [isLoadingPreview, setIsLoadingPreview] = createSignal(false);
  const [result, setResult] = createSignal<ExtractResult | null>(null);
  const [error, setError] = createSignal<string>('');

  const uniqueChars = createMemo(() => {
    const chars = [...new Set(text().split(''))].filter(c => c.trim() !== '');
    return chars;
  });

  const handleFontSelect = async (path: string) => {
    setFontPath(path);
    setSelectedFontIndex(0);
    setError('');
    setResult(null);
    setFontInfo(null);
    setIsLoadingFont(true);

    try {
      const info = await invoke<FontInfo>('get_font_info', { fontPath: path });
      setFontInfo(info);
    } catch (err) {
      setError(`获取字体信息失败: ${String(err)}`);
      setFontInfo(null);
    } finally {
      setIsLoadingFont(false);
    }
  };

  const handleExtract = async () => {
    if (!fontPath()) {
      setError('请先选择字体文件');
      return;
    }

    if (!text()) {
      setError('请输入需要提取的文本');
      return;
    }

    setIsExtracting(true);
    setError('');
    setResult(null);

    try {
      const savePath = await save({
        filters: [
          { name: '字体文件', extensions: ['ttf', 'otf', 'woff', 'woff2'] },
          { name: '所有文件', extensions: ['*'] },
        ],
        defaultPath: 'subset-font.ttf',
      });

      if (!savePath) {
        setIsExtracting(false);
        return;
      }

      const extractResult = await invoke<ExtractResult>('extract_font_subset', {
        fontPath: fontPath(),
        text: text(),
        outputPath: savePath,
        fontIndex: selectedFontIndex(),
      });

      setResult(extractResult);
    } catch (err) {
      setError(`提取失败: ${String(err)}`);
    } finally {
      setIsExtracting(false);
    }
  };

  return (
    <div class="app">
      <header class="app-header">
        <h1>字体提取工具</h1>
        <p>从字体文件中提取指定字符的子集，减小字体文件大小</p>
      </header>

      <main class="app-main">
        <section class="section">
          <h2>1. 选择字体文件</h2>
          <FontUploader
            fontPath={fontPath()}
            fontInfo={fontInfo()}
            selectedFontIndex={selectedFontIndex()}
            onSelect={handleFontSelect}
            onSelectFontIndex={(index) => {
              if (!isLoadingPreview()) {
                setSelectedFontIndex(index);
              }
            }}
            disabled={isLoadingPreview()}
            isLoadingPreview={isLoadingPreview()}
          />
        </section>

        <section class="section">
          <h2>2. 输入需要提取的文本</h2>
          <TextInput
            value={text()}
            onChange={setText}
            charCount={uniqueChars().length}
            disabled={isLoadingFont()}
          />
        </section>

        <Show when={fontPath()}>
          <section class="section">
            <h2>3. 预览</h2>
            <FontPreview
              fontPath={fontPath()}
              fontIndex={selectedFontIndex()}
              text={text()}
              uniqueChars={uniqueChars()}
              onLoadingChange={setIsLoadingPreview}
            />
          </section>
        </Show>

        <Show when={error()}>
          <div class="error-message">{error()}</div>
        </Show>

        <Show when={result()}>
          <ResultDisplay result={result()!} />
        </Show>

        <Show when={isLoadingFont()}>
          <div class="loading-overlay">
            <div class="loading-spinner"></div>
            <p>正在解析字体文件，请稍候...</p>
          </div>
        </Show>

        <div class="actions">
          <button
            class="extract-btn"
            onClick={handleExtract}
            disabled={isExtracting() || !fontPath() || !text()}
          >
            {isExtracting() ? '提取中...' : '提取字体子集'}
          </button>
        </div>
      </main>

      <footer class="app-footer">
        <p>支持格式: TTF, OTF, WOFF, WOFF2, TTC</p>
      </footer>
    </div>
  );
}

export default App;
