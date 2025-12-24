<template>
    <div class="w-full">
        <div class="max-w-[98%] m-auto w-full p-2">
            <h1 class="mb-4 text-3xl">Welcome to City Latitude & Longitude Finder</h1>
            <input type="text" v-model="search" @input="search_text"
                placeholder="Search by city, provence or country name..."
                class="bg-gray-100 black rounded-md p-2 mb-3 placeholder:text-gray-500 w-full">
            <div v-if="status === 'pending'">
                Loading data...
            </div>
            <div class="grid grid-cols-4 gap-3" v-else>
                <div v-for="city in city_records" class="odd:bg-slate-100 p-2 rounded-xl">
                    <p>{{ city.location }}</p>
                </div>
            </div>
        </div>
    </div>
</template>


<script setup lang="js">
const search = ref("");
const city_records = ref([]);
const saved_cities = ref([]);
const { status, data: cities } = await useLazyFetch('/api/cities');
watch(cities, (newCities) => {
    console.log("Status: " + status.value);
    city_records.value = newCities.records;
    saved_cities.value = newCities.records;
})

function search_text() {
    city_records.value = saved_cities.value.filter(item => {
        return item.location.toLowerCase().includes(search.value.toLowerCase())
    });
}


</script>