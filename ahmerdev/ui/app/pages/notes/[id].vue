<template>
    <h1>Update to notes app {{ note_id }}</h1>
</template>
<script setup lang="js">

const note_id = ref(0);
const route = useRoute().params;
note_id.value = route.id;
</script>