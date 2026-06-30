import { createApp } from 'vue';
import App from '@idle-colony/client/App.vue';
import { setDesktopHost } from '@idle-colony/client/desktop';
import { setResources } from '@idle-colony/client/resources';
import { setUi } from '@idle-colony/client/ui';
import { createDesktopHost } from './desktopHost';
import { createTauriResourceManager, createTauriUi } from './tauriResourceManager';

setResources(createTauriResourceManager());
setUi(createTauriUi());
setDesktopHost(createDesktopHost());
createApp(App).mount('#app');
