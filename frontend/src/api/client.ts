const BASE = 'http://localhost:8080'

/**
 * Prepend the backend base URL to a relative path. Used for URLs that
 * need to be handed to a browser element (e.g. `<audio src>`) where the
 * base can't be applied by `apiFetch`.
 */
export function absoluteUrl(path: string): string {
  if (path.startsWith('http://') || path.startsWith('https://')) return path
  return `${BASE}${path.startsWith('/') ? path : '/' + path}`
}


type Tokens = {
  access: string | null
  refresh: string | null
}

const tokens: Tokens = {
  access: localStorage.getItem('access_token'),
  refresh: localStorage.getItem('refresh_token'),
}

export function setTokens(access: string | null, refresh: string | null) {
  tokens.access = access
  tokens.refresh = refresh
  if (access) localStorage.setItem('access_token', access)
  else localStorage.removeItem('access_token')
  if (refresh) localStorage.setItem('refresh_token', refresh)
  else localStorage.removeItem('refresh_token')
}

export function getAccessToken() {
  return tokens.access
}

export function getRefreshToken() {
  return tokens.refresh
}

async function refreshAccessToken(): Promise<boolean> {
  if (!tokens.refresh) return false
  const res = await fetch(`${BASE}/api/auth/refresh`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ refresh_token: tokens.refresh }),
  })
  if (!res.ok) return false
  const data = await res.json()
  setTokens(data.access_token, tokens.refresh)
  return true
}

export async function apiFetch(path: string, init: RequestInit = {}): Promise<Response> {
  const headers = new Headers(init.headers)
  if (tokens.access) headers.set('Authorization', `Bearer ${tokens.access}`)
  if (init.body && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json')

  let res = await fetch(`${BASE}${path}`, { ...init, headers })

  if (res.status === 401 && tokens.refresh) {
    const refreshed = await refreshAccessToken()
    if (refreshed) {
      headers.set('Authorization', `Bearer ${tokens.access}`)
      res = await fetch(`${BASE}${path}`, { ...init, headers })
    }
  }
  return res
}
