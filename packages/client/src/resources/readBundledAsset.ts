import { bundledAssetExists as fetchBundledAssetExists, bundledAssetUrl, fetchBundledText } from './bundledAsset';
import { getBundledContent, hasBundledContent } from './bundledContentStore';

export async function readBundledAsset(path: string, baseUrl = import.meta.env.BASE_URL): Promise<string> {
  const embedded = getBundledContent(path);
  if (embedded !== undefined) return embedded;
  return fetchBundledText(bundledAssetUrl(path, baseUrl));
}

export async function bundledAssetExists(path: string, baseUrl = import.meta.env.BASE_URL): Promise<boolean> {
  if (hasBundledContent(path)) return true;
  return fetchBundledAssetExists(bundledAssetUrl(path, baseUrl));
}
