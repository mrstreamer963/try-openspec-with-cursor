import 'pixi.js/unsafe-eval';
import { path } from 'pixi.js';

const CUSTOM_SCHEME = /^(?:https?|tauri|capacitor|asset):/i;

const originalIsUrl = path.isUrl.bind(path);
path.isUrl = (url: string) => CUSTOM_SCHEME.test(path.toPosix(url)) || originalIsUrl(url);

const originalRootname = path.rootname.bind(path);
path.rootname = (url: string) => {
  const posix = path.toPosix(url);
  const customRoot = /^([a-z+]+:\/\/[^/]+\/)/i.exec(posix);
  if (customRoot && !/^https?:/i.test(posix)) {
    return customRoot[1];
  }
  return originalRootname(url);
};
