<template>
    <section class="max-w-4xl m-auto w-full px-4 py-3">
        <h1>Welcome to notes app</h1>
        <div>

        </div>
    </section>
</template>
<script setup lang="js">
const record = ref([]);

try {
    const { data } = await useFetch("http://127.0.0.1:8787/api/notes/get");
    record.value = data.value;
    console.log(record.value);
} catch (e) {
    console.log(e);
}
</script>