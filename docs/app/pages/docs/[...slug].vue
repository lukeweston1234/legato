<script lang="ts" setup>
const route = useRoute();
const pageId = computed(() => `/docs/${route.path}`);
const { data } = await useAsyncData(pageId, () => {
  return queryCollection("docs").path(route.path).first();
});
</script>

<template v>
  <div>
    <NuxtLink to="/">
      <small>Back</small>
    </NuxtLink>
    <div v-if="data">
      <h1>{{ data.title }}</h1>
      <ContentRenderer :value="data"> </ContentRenderer>
    </div>
  </div>
</template>
