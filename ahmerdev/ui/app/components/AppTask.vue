<template>
    <div class="flex flex-col relative group">
        <div class="flex">
            <img :src="avatar ? avatar.avatar : '/avatar.jpg'" alt="Avatar" class="rounded-full h-11 w-11 object-fill">
            <div class="flex flex-col bg-gray-50 rounded-lg border border-gray-200 ml-2 p-3 w-full">
                <div class="flex items-center gap-2">
                    <div class="flex items-center">
                        <h3 class="font-bold">{{ avatar?.name || 'Admin' }}</h3>
                        <img src="https://api.iconify.design/mage:verified-check-fill.svg?color=%230ac2ff" width="20"
                            alt="Verified" class="ml-1">
                    </div>
                    <span class="text-gray-600 text-sm">{{ formatDate(record.created_at) }} UTC</span>
                    <span v-if="!is_completed"
                        class="py-0.5 px-2 rounded-full bg-yellow-400/20 border border-yellow-500 text-yellow-700 font-semibold text-[10px]">Pending</span>
                    <span v-else
                        class="py-0.5 px-2 rounded-full bg-green-400/20 border border-green-500 text-green-700 font-semibold text-[10px]">Completed</span>
                </div>
                <p class="text-gray-600 mt-1 text-sm">{{ summary }}</p>
                <div v-if="dropdown"
                    class="fixed z-50 inset-0 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
                    <div class="bg-white rounded-xl shadow-lg p-6 w-full max-w-sm">
                        <h2 class="text-lg font-semibold text-gray-900 mb-4">Delete task?</h2>
                        <p class="text-sm text-gray-600 mb-6">This action cannot be undone.</p>
                        <div class="flex gap-3">
                            <button @click="delete_task()"
                                class="flex-1 px-4 py-2 bg-red-600 text-white font-medium rounded-lg hover:bg-red-700 transition-colors">
                                {{ processing ? 'Deleting' : 'Delete' }}
                            </button>
                            <button @click="dropdown = false"
                                class="flex-1 px-4 py-2 bg-gray-100 text-gray-900 font-medium rounded-lg hover:bg-gray-200 transition-colors">
                                Cancel
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <div v-if="update_dropdown"
            class="fixed z-50 inset-0 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
            <div class="bg-white rounded-xl shadow-lg p-6 w-full max-w-lg">
                <h2 class="text-lg font-semibold text-gray-900 mb-4">Update task</h2>

                <div v-if="success_message" class="border border-green-600 bg-white text-green-600 rounded-lg mb-4">
                    <div class="p-4 text-center relative">
                        <button class="absolute top-1 right-1" @click="success_message = ''">
                            <img src="https://api.iconify.design/basil:cross-solid.svg?color=%23e01b24" width="25px">
                        </button>
                        <p>{{ success_message }}</p>
                    </div>
                </div>

                <textarea v-model="new_summary" rows="8"
                    class="border border-gray-200 bg-gray-100 p-3 w-full rounded-lg mb-3"
                    placeholder="Write task summary here..."></textarea>

                <select v-model.number="is_completed"
                    class="border border-gray-200 bg-gray-100 p-3 w-full rounded-lg mb-3">
                    <option :value="null">Please select the status!</option>
                    <option :value="1">Task completed</option>
                    <option :value="0">Incomplete task</option>
                </select>

                <div class="flex gap-3">
                    <button @click="update_task()"
                        class="flex-1 px-4 py-2 bg-indigo-600 text-white font-medium rounded-lg hover:bg-indigo-700 transition-colors">
                        {{ processing ? 'Updating...' : 'Update' }}
                    </button>
                    <button @click="update_dropdown = false"
                        class="flex-1 px-4 py-2 bg-gray-100 text-gray-900 font-medium rounded-lg hover:bg-gray-200 transition-colors">
                        Close
                    </button>
                </div>
            </div>
        </div>
        <div class="absolute top-2 right-2 hidden group-hover:block">
            <div class="flex justify-end relative">
                <button class="cursor-pointer" @click="dropdown_box = !dropdown_box">
                    <img src="https://api.iconify.design/tabler:dots-vertical.svg" width="20px">
                </button>
                <div v-if="dropdown_box" class="absolute w-45 bg-white p-3 z-20 rounded-lg top-5 shadow-md right-0">
                    <h3 class="text-sm w-full border-b border-gray-200 pb-1 mb-1">Action</h3>
                    <button @click="update_dropdown = true" class="text-sm py-2 inline-block">
                        Edit task
                    </button>
                    <button
                        class="cursor-pointer flex items-center justify-center px-4 py-2 w-full bg-red-600 text-white font-medium rounded-lg hover:bg-red-700 transition-colors"
                        @click="dropdown = true">
                        <img src="https://api.iconify.design/material-symbols:delete-forever-outline-sharp.svg?color=%23ffffff"
                            width="18px">
                        <span class="ml-1 text-sm">Delete</span>
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="js">
const dropdown = ref(false);
const dropdown_box = ref(false);
const update_dropdown = ref(false);
const processing = ref(false);
const success_message = ref("");

const summary = ref("");
const new_summary = ref("");
const is_completed = ref(null);

const props = defineProps({
    record: Object,
    avatar: Object,
    token: String
});

const emit = defineEmits(['task-deleted']);

watchEffect(() => {
    if (props.record) {
        new_summary.value = props.record.summary;
        summary.value = props.record.summary;
        is_completed.value = props.record.is_completed ? 1 : 0;
    }
});

async function delete_task() {
    try {
        processing.value = true;
        const data = await $fetch("/api/notes/delete/" + props.record.id, {
            method: "DELETE",
            headers: {
                "Authorization": "Bearer " + props.token
            },
        });
        if (data.status == 200) {
            success_message.value = data.message;
            emit('task-deleted', data.message);
        }
    } catch (e) {
        console.log(e);
    } finally {
        processing.value = false;
    }
}

async function update_task() {
    try {
        processing.value = true;
        success_message.value = "";
        const data = await $fetch("/api/notes/update/" + props.record.id, {
            method: "PATCH",
            headers: {
                "Authorization": "Bearer " + props.token
            },
            body: {
                summary: new_summary.value,
                is_completed: is_completed.value == '1' ? true : false
            }
        });
        if (data.status == 200) {
            success_message.value = data.message;
            summary.value = new_summary.value;
        }
    } catch (e) {
        console.log(e);
    } finally {
        processing.value = false;
    }
}


const formatDate = (dateString) => {
    return new Date(dateString).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
    });
};

</script>