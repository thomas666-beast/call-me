<script setup lang="ts">
import { useAuthStore } from '../stores/auth'
import EventLog from '../components/EventLog.vue'

const auth = useAuthStore()
</script>

<template>
  <div>
    <h2>Dashboard</h2>
    <p>Logged in as <strong>{{ auth.user?.username }}</strong> (role: {{ auth.user?.role }})</p>
    <h3>Live AMI events</h3>
    <EventLog />
  </div>
</template>
