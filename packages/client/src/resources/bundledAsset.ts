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
