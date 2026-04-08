export interface SingleFontInfo {
  index: number;
  family_name: string | null;
  post_script_name: string | null;
  num_glyphs: number;
  units_per_em: number;
  error?: string;
}

export interface FontInfo {
  is_collection: boolean;
  num_fonts: number;
  fonts: SingleFontInfo[];
  file_size: number;
}

export interface ExtractResult {
  success: boolean;
  message: string;
  output_path: string | null;
}

export interface ExtractOptions {
  fontPath: string;
  text: string;
  outputPath: string;
  fontIndex?: number;
}
