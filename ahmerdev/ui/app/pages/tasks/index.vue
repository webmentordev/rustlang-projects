<template>
    <section class="max-w-2xl m-auto w-full px-4 py-3" v-if="avatar">
        <div class="flex flex-col mb-5 border-b border-gray-200 pb-3">
            <div class="flex items-center justify-between mb-4">
                <h1 class="font-black text-2xl">Daily tasks progress</h1>
                <div class="flex items-center">
                    <NuxtLink to="/" class="px-4 w-fit">
                        Profile
                    </NuxtLink>
                    <button @click="create_task = !create_task"
                        class="flex items-center bg-black text-white py-1 px-3 rounded-lg font-semibold cursor-pointer">
                        <span class="mr-2">Add task</span>
                        <img src="https://api.iconify.design/material-symbols:add-2.svg?color=%23ffffff" width="18px">
                    </button>
                </div>
            </div>
            <div v-if="success_message"
                class="border z-50 bg-white border-green-600 min-w-87.5 text-green-600 rounded-lg fixed bottom-4 left-[40%]">
                <div class="p-4 text-center w-full h-full relative">
                    <button class="absolute top-1 right-1 z-10" @click="success_message = ''">
                        <img src="https://api.iconify.design/basil:cross-solid.svg?color=%23e01b24" width="25px">
                    </button>
                    <p>{{ success_message }}</p>
                </div>
            </div>

            <div class="flex flex-col mb-4 p-4 bg-gray-50 border border-gray-200 rounded-lg" v-if="create_task">
                <textarea v-model="summary" rows="8"
                    class="border border-gray-200 bg-gray-100 p-3 w-full h-full rounded-lg mb-3"
                    placeholder="Write task summary here..." required></textarea>
                <input type="text" v-model="token" required
                    class="border border-gray-200 bg-gray-100 p-3 w-full h-full rounded-lg mb-3"
                    placeholder="Auth token">
                <div class="flex items-center">
                    <button @click="task_create()"
                        class="bg-blue-500 text-white w-fit py-1 px-3 rounded-lg font-semibold cursor-pointer">{{
                            processing
                                ? 'processing...' : 'Create note' }}</button>
                    <button @click="setLocalStorage()"
                        class="bg-blue-500 text-white ml-3 w-fit py-1 px-3 rounded-lg font-semibold cursor-pointer">Save
                        token</button>
                </div>
            </div>
            <p class="text-gray-600">This task board is for my personal and office-related work and progress tracking.
                This is where I record
                my
                daily
                activities, such as what I did today and what I plan to do. If tasks are in a pending state, it means
                that I am either working on them or they depend on another task that is currently in progress.</p>
        </div>
        <div class="flex flex-col w-full">
            <div class="w-full mb-3" v-for="record in records" :key="record.id">
                <AppTask @task-deleted="taskDeleteHandler" :record="record" :avatar="avatar" :token="token" />
            </div>
        </div>
    </section>
</template>
<script setup lang="js">
const records = ref([]);
const avatar = ref(null);
const create_task = ref(false);
const token = ref(null);
const summary = ref(null);
const processing = ref(false);
const success_message = ref("");

const saved = localStorage.getItem('avatar');
const auth_token = localStorage.getItem('auth_token');
token.value = auth_token;
avatar.value = saved ? JSON.parse(saved) : null;

await fetchNotes();

async function fetchNotes() {
    try {
        const data = await $fetch("http://127.0.0.1:8787/api/notes/get");
        records.value = data.data;
    } catch (e) {
        console.log(e);
    }
}

async function task_create() {
    try {
        processing.value = true;
        success_message.value = "";
        const data = await $fetch("http://127.0.0.1:8787/api/notes/create", {
            method: "POST",
            headers: {
                "Authorization": "Bearer " + token.value
            },
            body: {
                summary: summary.value
            }
        });
        if (data.status == 200) {
            summary.value = "";
            localStorage.setItem('auth_token', token.value)
            success_message.value = data.message;
            await fetchNotes();
        }
    } catch (e) {
        console.log(e);
    } finally {
        processing.value = false;
    }
}


const taskDeleteHandler = async (message) => {
    success_message.value = message;
    await fetchNotes();
};


function setLocalStorage() {
    localStorage.setItem('auth_token', token.value);
}
</script>