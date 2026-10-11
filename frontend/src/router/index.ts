import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { getAccessToken } from '../api/client'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/dashboard' },
    { path: '/login', component: () => import('../views/LoginView.vue'), meta: { public: true } },
    { path: '/dashboard', component: () => import('../views/DashboardView.vue') },

    // Agent routes
    { path: '/agent', component: () => import('../views/AgentHomeView.vue'), meta: { requiresAuth: true } },
    { path: '/agent/calls', component: () => import('../views/AgentCallsView.vue'), meta: { requiresAuth: true } },
    { path: '/agent/contacts', component: () => import('../views/AgentContactsView.vue'), meta: { requiresAuth: true } },

    // Admin routes
    { path: '/admin/agents', component: () => import('../views/AdminAgentsView.vue'), meta: { requiresRole: 'admin' } },
    { path: '/admin/users', component: () => import('../views/AdminUsersView.vue'), meta: { requiresRole: 'admin' } },
    { path: '/admin/extensions', component: () => import('../views/AdminExtensionsView.vue'), meta: { requiresRole: 'admin' } },
    { path: '/admin/queues', component: () => import('../views/AdminQueuesView.vue'), meta: { requiresRole: 'admin' } },
    { path: '/admin/calls', component: () => import('../views/AdminCallsView.vue'), meta: { requiresRole: 'admin' } },
    { path: '/admin/calls/:id', component: () => import('../views/CallDetailView.vue'), meta: { requiresRole: 'admin' } },
    { path: '/admin/contacts', component: () => import('../views/AdminContactsView.vue'), meta: { requiresRole: 'admin' } },
    { path: '/admin/contacts/:id', component: () => import('../views/ContactDetailView.vue'), meta: { requiresRole: 'admin' } },
    { path: '/admin/settings', component: () => import('../views/AdminSettingsView.vue'), meta: { requiresRole: 'admin' } },

    { path: '/:pathMatch(.*)*', component: () => import('../views/NotFoundView.vue'), meta: { public: true } },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (getAccessToken() && !auth.user) await auth.fetchMe()

  if (!to.meta.public && !auth.isAuthenticated) {
    return { path: '/login', query: { redirect: to.fullPath } }
  }

  const isAgent = auth.user?.role === 'agent'
  const isAdmin = auth.user?.role === 'admin'

  if (isAgent && to.path.startsWith('/admin')) return { path: '/agent/calls' }
  if (!isAgent && to.path.startsWith('/agent')) return { path: '/dashboard' }

  if (to.meta.requiresRole && auth.user?.role !== to.meta.requiresRole) {
    if (isAgent) return { path: '/agent/calls' }
    return { path: '/dashboard' }
  }

  if (to.path === '/login' && auth.isAuthenticated) {
    if (isAgent) return { path: '/agent/calls' }
    if (isAdmin) return { path: '/dashboard' }
  }
})

export default router
