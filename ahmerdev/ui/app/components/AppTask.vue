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
                    <span v-if="!record.is_completed"
                        class="py-0.5 px-2 rounded-full bg-yellow-400/20 border border-yellow-500 text-yellow-700 font-semibold text-[10px]">Pending</span>
                    <span v-else
                        class="py-0.5 px-2 rounded-full bg-green-400/20 border border-green-500 text-green-700 font-semibold text-[10px]">Completed</span>
                </div>
                <p class="text-gray-600 mt-1 text-sm">{{ record.summary }}</p>
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
        <div class="absolute top-1 right-1 hidden group-hover:block">
            <button class="cursor-pointer" @click="dropdown = true">
                <img src="https://api.iconify.design/material-symbols:delete-forever-outline-sharp.svg?color=%23e01b24"
                    width="15px">
            </button>
        </div>
    </div>
</template>

<script setup lang="js">
const dropdown = ref(false);
const processing = ref(false);
const success_message = ref("");
const props = defineProps({
    record: Object,
    avatar: Object,
    token: String
});
const emit = defineEmits(['task-deleted']);

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


const formatDate = (dateString) => {
    return new Date(dateString).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
    });
};

</script>