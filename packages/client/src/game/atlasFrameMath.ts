export interface FrameRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function resolveColumns(
  imageWidth: number,
  tileSize: number,
  spacing: number,
  columns: number | 'auto',
): number {
  if (columns !== 'auto') return columns;
  const stride = tileSize + spacing;
  if (stride <= 0) return 1;
  return Math.max(1, Math.floor((imageWidth + spacing) / stride));
}

export function rowCount(
  imageHeight: number,
  tileSize: number,
  spacing: number,
): number {
  const stride = tileSize + spacing;
  if (stride <= 0) return 1;
  return Math.max(1, Math.floor((imageHeight + spacing) / stride));
}

export function maxFrameCount(
  imageWidth: number,
  imageHeight: number,
  tileSize: number,
  spacing: number,
  columns: number | 'auto',
): number {
  const cols = resolveColumns(imageWidth, tileSize, spacing, columns);
  const rows = rowCount(imageHeight, tileSize, spacing);
  return cols * rows;
}

export function frameToRect(
  frame: number,
  columns: number,
  tileSize: number,
  spacing: number,
  rows?: number,
): FrameRect | null {
  if (frame < 0 || columns <= 0) return null;
  const col = frame % columns;
  const row = Math.floor(frame / columns);
  if (rows !== undefined && row >= rows) return null;
  const stride = tileSize + spacing;
  return {
    x: col * stride,
    y: row * stride,
    width: tileSize,
    height: tileSize,
  };
}
