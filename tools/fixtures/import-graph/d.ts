export const shared = 'd'

async function lazy() {
  return import('./b')
}

export { lazy }
