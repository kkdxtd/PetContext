export function join(...paths: string[]): string {
  return paths.join('/').replace(/\/+/g, '/')
}
