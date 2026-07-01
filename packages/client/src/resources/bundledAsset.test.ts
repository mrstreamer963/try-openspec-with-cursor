import { describe, expect, it } from 'vitest';
import { bundledAssetUrl, isHtmlSpaFallbackBody, isMissingBundledAsset } from './bundledAsset';

function mockResponse(status: number, contentType: string | null): Response {
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: { get: (name: string) => (name === 'content-type' ? contentType : null) },
  } as Response;
}

describe('bundledAssetUrl', () => {
  it('uses root-relative paths for Tauri/Vite default base', () => {
    expect(bundledAssetUrl('base/needs.yaml', './')).toBe('/base/needs.yaml');
    expect(bundledAssetUrl('base/needs.yaml', '/')).toBe('/base/needs.yaml');
  });

  it('prefixes custom deploy bases', () => {
    expect(bundledAssetUrl('base/needs.yaml', '/myapp/')).toBe('/myapp/base/needs.yaml');
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
