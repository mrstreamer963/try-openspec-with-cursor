import type { InjectionKey, ShallowRef } from 'vue';
import type { ContentPack } from './types';

export const contentPackKey: InjectionKey<ShallowRef<ContentPack | null>> = Symbol('contentPack');
