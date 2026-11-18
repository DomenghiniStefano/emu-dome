// The development-only pages, by their hash. `main.ts` mounts them only when
// `import.meta.env.DEV`, so the production build carries none of them.
export const DevRoute = {
  Kit: '#kit',
} as const
export type DevRoute = (typeof DevRoute)[keyof typeof DevRoute]
