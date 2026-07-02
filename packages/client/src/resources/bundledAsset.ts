/** Build a fetch URL for a bundled asset path under the configured Vite base URL. */
export function bundledAssetUrl(path: string, baseUrl = import.meta.env.BASE_URL): string {
  const relative = path.startsWith('/') ? path.slice(1) : path;
  if (typeof window !== 'undefined' && window.location?.href) {
    return new URL(relative, window.location.href).href;
  }
  // Tauri release serves from a custom-protocol origin where `./…` resolves incorrectly.
  if (baseUrl === './' || baseUrl === '/') {
    return `/${relative}`;
  }
  const root = baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`;
  return `${root}${relative}`;
}

/** Fetch bundled asset text; rejects SPA index.html fallbacks. */
export async function fetchBundledText(url: string): Promise<string> {
  let response: Response;
  try {
    response = await fetch(url);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to fetch (${url}): ${message}`);
  }
  if (!response.ok) {
    throw new Error(`Failed to fetch (${url}): HTTP ${response.status} ${response.statusText}`);
  }
  const raw = await response.text();
  if (isMissingBundledAsset(response, raw)) {
    throw new Error(`Failed to fetch (${url}): file not found`);
  }
  return raw;
}

/** Check bundled asset availability via GET (HEAD is unreliable in Tauri release). */
export async function bundledAssetExists(url: string): Promise<boolean> {
  try {
    const response = await fetch(url);
    if (!response.ok) return false;
    const raw = await response.text();
    return !isMissingBundledAsset(response, raw);
  } catch {
    return false;
  }
}

/** True when the server returned SPA HTML instead of a static asset (common Vite dev fallback). */
export function isMissingBundledAsset(response: Response, raw?: string): boolean {
  if (response.status === 404) return true;
  const contentType = response.headers.get('content-type');
  if (contentType?.toLowerCase().includes('text/html')) return true;
  if (raw !== undefined) {
    const trimmed = raw.trimStart().toLowerCase();
    if (trimmed.startsWith('<!doctype') || trimmed.startsWith('<html')) return true;
  }
  return false;
}

export function isHtmlSpaFallbackBody(raw: string): boolean {
  const trimmed = raw.trimStart().toLowerCase();
  return trimmed.startsWith('<!doctype') || trimmed.startsWith('<html');
}
