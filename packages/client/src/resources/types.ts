export type ResourceLocation = 'bundled' | 'data';

export interface ResourceManager {
  readText(location: ResourceLocation, path: string): Promise<string>;
  exists(location: ResourceLocation, path: string): Promise<boolean>;
  listDir(location: ResourceLocation, path: string): Promise<string[]>;
  writeText(location: ResourceLocation, path: string, content: string): Promise<void>;
  rename(location: ResourceLocation, fromPath: string, toPath: string): Promise<void>;
  delete(location: ResourceLocation, path: string): Promise<void>;
  initialize(): Promise<void>;
  revealInFileManager(location: ResourceLocation, path: string): Promise<void>;
}
