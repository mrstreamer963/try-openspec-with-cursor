import { describe, expect, it } from 'vitest';
import { isMissingYamlAsset } from './loadContent';

function mockResponse(status: number, contentType: string | null): Response {
  return { status, headers: { get: (name: string) => (name === 'content-type' ? contentType : null) } } as Response;
}

describe('isMissingYamlAsset', () => {
  it('detects 404', () => {
    expect(isMissingYamlAsset(mockResponse(404, null), '')).toBe(true);
  });

  it('detects HTML content-type', () => {
    expect(isMissingYamlAsset(mockResponse(200, 'text/html'), 'needs:\n  - id: food')).toBe(true);
  });

  it('detects HTML body from SPA fallback', () => {
    const html = '<!DOCTYPE html><html><head><title>Idle Colony Sim</title></head></html>';
    expect(isMissingYamlAsset(mockResponse(200, 'text/html; charset=utf-8'), html)).toBe(true);
  });

  it('accepts real YAML', () => {
    expect(isMissingYamlAsset(mockResponse(200, 'text/yaml'), 'needs:\n  - id: food')).toBe(false);
  });
});
