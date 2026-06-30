import { createApp } from 'vue';
import { setResources } from '@idle-colony/client/resources';
import { setUi } from '@idle-colony/client/ui';
import DesktopApp from './DesktopApp.vue';
import { createTauriResourceManager, createTauriUi } from './tauriResourceManager';

setResources(createTauriResourceManager());
setUi(createTauriUi());
createApp(DesktopApp).mount('#app');
