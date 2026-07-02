/** Eager-bundled game content YAML (reliable in Tauri release; fetch is not). */
const modules = import.meta.glob('../../../../content/**/*.yaml', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function toResourcePath(globPath: string): string {
  const marker = '/content/';
  const idx = globPath.indexOf(marker);
  if (idx === -1) return globPath;
  return globPath.slice(idx + marker.length);
}

const store = new Map<string, string>(
  Object.entries(modules).map(([globPath, raw]) => [toResourcePath(globPath), raw]),
);

export function getBundledContent(path: string): string | undefined {
  const normalized = path.startsWith('/') ? path.slice(1) : path;
  return store.get(normalized);
}

export function hasBundledContent(path: string): boolean {
  return getBundledContent(path) !== undefined;
}
