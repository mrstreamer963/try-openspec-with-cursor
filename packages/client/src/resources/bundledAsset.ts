/** Build a fetch URL for a bundled asset path under the configured Vite base URL. */
export function bundledAssetUrl(path: string, baseUrl = import.meta.env.BASE_URL): string {
  const relative = path.startsWith('/') ? path.slice(1) : path;
  // Tauri release serves from a custom-protocol origin where `./…` resolves incorrectly.
  if (baseUrl === './' || baseUrl === '/') {
    return `/${relative}`;
  }
  const root = baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`;
  return `${root}${relative}`;
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
