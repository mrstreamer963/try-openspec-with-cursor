import { describe, expect, it, vi } from 'vitest';
import {
  bundledAssetExists,
  bundledAssetUrl,
  fetchBundledText,
  isHtmlSpaFallbackBody,
  isMissingBundledAsset,
} from './bundledAsset';

function mockResponse(status: number, contentType: string | null): Response {
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: { get: (name: string) => (name === 'content-type' ? contentType : null) },
  } as Response;
}

describe('bundledAssetUrl', () => {
  it('uses root-relative paths when window is unavailable', () => {
    expect(bundledAssetUrl('base/needs.yaml', './')).toBe('/base/needs.yaml');
    expect(bundledAssetUrl('base/needs.yaml', '/')).toBe('/base/needs.yaml');
  });

  it('resolves against the current page in the browser', () => {
    vi.stubGlobal('window', { location: { href: 'https://tauri.localhost/index.html' } });
    expect(bundledAssetUrl('base/needs.yaml', './')).toBe('https://tauri.localhost/base/needs.yaml');
    vi.unstubAllGlobals();
  });

  it('prefixes custom deploy bases', () => {
    expect(bundledAssetUrl('base/needs.yaml', '/myapp/')).toBe('/myapp/base/needs.yaml');
  });
});

describe('fetchBundledText', () => {
  it('returns yaml body on success', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        headers: { get: () => 'text/yaml' },
        text: async () => 'needs:\n  - id: food',
      })),
    );
    await expect(fetchBundledText('/base/needs.yaml')).resolves.toBe('needs:\n  - id: food');
    vi.unstubAllGlobals();
  });

  it('rejects SPA html fallback', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        headers: { get: () => 'text/html' },
        text: async () => '<!DOCTYPE html><html></html>',
      })),
    );
    await expect(fetchBundledText('/base/needs.yaml')).rejects.toThrow(/file not found/);
    vi.unstubAllGlobals();
  });
});

describe('bundledAssetExists', () => {
  it('returns true for yaml asset', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        headers: { get: () => 'text/yaml' },
        text: async () => 'needs:\n  - id: food',
      })),
    );
    await expect(bundledAssetExists('/base/needs.yaml')).resolves.toBe(true);
    vi.unstubAllGlobals();
  });

  it('returns false for html fallback', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        headers: { get: () => 'text/html' },
        text: async () => '<!DOCTYPE html><html></html>',
      })),
    );
    await expect(bundledAssetExists('/base/needs.yaml')).resolves.toBe(false);
    vi.unstubAllGlobals();
  });
});

describe('isMissingBundledAsset', () => {
  it('treats 404 as missing', () => {
    expect(isMissingBundledAsset(mockResponse(404, null))).toBe(true);
  });

  it('treats 200 text/html as missing', () => {
    expect(isMissingBundledAsset(mockResponse(200, 'text/html'))).toBe(true);
  });

  it('treats 200 text/html with charset as missing', () => {
    expect(isMissingBundledAsset(mockResponse(200, 'text/html; charset=utf-8'))).toBe(true);
  });

  it('accepts 200 with yaml content-type', () => {
    expect(isMissingBundledAsset(mockResponse(200, 'text/yaml'))).toBe(false);
  });

  it('detects HTML body when content-type is absent', () => {
    const html = '<!DOCTYPE html><html><head><title>Idle Colony Sim</title></head></html>';
    expect(isMissingBundledAsset(mockResponse(200, null), html)).toBe(true);
  });

  it('accepts yaml body when content-type is absent', () => {
    expect(isMissingBundledAsset(mockResponse(200, null), 'needs:\n  - id: food')).toBe(false);
  });
});

describe('isHtmlSpaFallbackBody', () => {
  it('detects doctype prefix', () => {
    expect(isHtmlSpaFallbackBody('<!DOCTYPE html><html></html>')).toBe(true);
  });

  it('detects html prefix without doctype', () => {
    expect(isHtmlSpaFallbackBody('<html lang="en"></html>')).toBe(true);
  });

  it('ignores leading whitespace', () => {
    expect(isHtmlSpaFallbackBody('  \n<!DOCTYPE html>')).toBe(true);
  });

  it('rejects yaml', () => {
    expect(isHtmlSpaFallbackBody('needs:\n  - id: food')).toBe(false);
  });
});
