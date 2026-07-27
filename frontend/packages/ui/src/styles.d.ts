// TypeScript 7 wants a declaration for a side-effect stylesheet import; the
// apps get this from vite/client, which a library package does not pull in.
declare module '*.css'
