/// <reference types="vite/client" />

declare module '*?init' {
  const init: () => Promise<void>;
  export default init;
}
