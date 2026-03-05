/**
 * Returns all ancestor paths from root to the provided path.
 * Example: "/a/b/c" => ["/", "/a", "/a/b", "/a/b/c"]
 */
export function getPathAncestors(path: string): Array<string> {
  if (!path || path === '/') {
    return ['/']
  }

  const normalizedPath = path.startsWith('/') ? path : `/${path}`
  const segments = normalizedPath.split('/').filter(Boolean)
  const ancestors: Array<string> = ['/']

  let currentPath = ''
  for (const segment of segments) {
    currentPath = `${currentPath}/${segment}`
    ancestors.push(currentPath)
  }

  return ancestors
}

/**
 * Returns the parent directory path from a file/directory path.
 * "/a/b/c" => "/a/b"
 * "/a" => "/"
 * "/" => "/"
 */
export function getParentPath(path: string): string {
  if (!path || path === '/') {
    return '/'
  }

  const lastSlash = path.lastIndexOf('/')
  if (lastSlash <= 0) {
    return '/'
  }

  return path.slice(0, lastSlash)
}
