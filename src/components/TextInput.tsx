import './TextInput.css';

interface TextInputProps {
  value: string;
  onChange: (value: string) => void;
  charCount: number;
  disabled?: boolean;
}

export function TextInput(props: TextInputProps) {
  return (
    <div class="text-input">
      <textarea
        value={props.value}
        onInput={(e) => props.onChange(e.currentTarget.value)}
        placeholder="在此输入需要提取的文本...&#10;例如：你好世界 Hello World 123"
        rows={6}
        {...(props.disabled ? { disabled: true } : {})}
      />
      <div class="char-count">
        唯一字符数: <strong>{props.charCount}</strong>
      </div>
    </div>
  );
}
