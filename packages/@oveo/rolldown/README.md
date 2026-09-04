# @oveo/rolldown

Vite/Rolldown plugin for the [oveo](https://github.com/localvoid/oveo) JavaScript optimizer.

```js
import { oveo } from '@oveo/rolldown';

export default {
  plugins: [oveo({ dedupe: true, globals: true, url: true })],
};
```

Build-mode only (`apply: 'build'`). All optimizations are disabled by default — see [Quick setup](https://github.com/localvoid/oveo#quick-setup), [Plugin options](https://github.com/localvoid/oveo#plugin-options), and [Caveats](https://github.com/localvoid/oveo#caveats-and-safety) in the main README.
