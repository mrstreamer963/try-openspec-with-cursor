/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  const component: DefineComponent<object, object, unknown>;
  export default component;
}

declare module '*.yaml?raw' {
  const content: string;
  export default content;
}

declare module '@content/*?raw' {
  const content: string;
  export default content;
}
