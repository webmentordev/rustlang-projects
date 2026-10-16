<template>
    <section class="max-w-2xl m-auto w-full px-4 py-3">
        <div class="flex flex-col mb-5 border-b border-gray-200 pb-3">
            <div class="flex items-center justify-between mb-4 max-main-2:flex-col">
                <h1 class="font-black text-2xl">Professional todos</h1>
                <div class="flex items-center max-main-2:mt-2 gap-3">
                    <NuxtLink to="/" class="w-fit">
                        Profile
                    </NuxtLink>
                    <NuxtLink to="/tasks" class="w-fit">
                        Tasks
                    </NuxtLink>
                    <button @click="create_todo = !create_todo"
                        class="flex items-center bg-black text-white py-1 px-3 rounded-lg font-semibold cursor-pointer">
                        <span>Add todo</span>
                    </button>
                </div>
            </div>
            <div v-if="success_message"
                class="border z-50 bg-green-600/30 backdrop-blur-xl border-green-600 min-w-87.5 text-green-600 rounded-lg fixed bottom-4 left-[40%]">
                <div class="p-4 text-center w-full h-full relative">
                    <button class="absolute top-1 right-3 z-10 text-red-600 font-bold" @click="success_message = ''">
                        x
                    </button>
                    <p>{{ success_message }}</p>
                </div>
            </div>

            <div class="flex flex-col mb-4 p-4 bg-gray-50 border border-gray-200 rounded-lg" v-if="create_todo">
                <textarea v-model="todo" rows="3"
                    class="border border-gray-200 bg-gray-100 p-3 w-full h-full rounded-lg mb-3"
                    placeholder="Write todo here..." required></textarea>
                <select v-model="status" required class="border border-gray-200 bg-gray-100 p-3 w-full rounded-lg mb-3">
                    <option value="pending">Pending</option>
                    <option value="in_progress">In progress</option>
                    <option value="completed">Completed</option>
                </select>
                <input type="text" v-model="token" required
                    class="border border-gray-200 bg-gray-100 p-3 w-full h-full rounded-lg mb-3"
                    placeholder="Auth token">
                <div class="flex items-center">
                    <button @click="todo_create()"
                        class="bg-blue-500 text-white w-fit py-1 px-3 rounded-lg font-semibold cursor-pointer">{{
                            processing ? 'processing...' : 'Create todo' }}</button>
                    <button @click="setLocalStorage()"
                        class="bg-blue-500 text-white ml-3 w-fit py-1 px-3 rounded-lg font-semibold cursor-pointer">Save
                        token</button>
                </div>
            </div>
            <p class="text-gray-600">This board tracks my professional work related tasks or todos. Each todo is either
                pending, in progress
                or completed, and completed todos show the date they were finished.</p>
        </div>

        <div class="flex flex-wrap items-end gap-3 mb-4">
            <div>
                <label class="block text-xs font-semibold text-gray-600 mb-1">Completed from</label>
                <input v-model="start_date" type="date" :max="end_date || undefined" @change="applyFilters()"
                    class="border border-gray-200 bg-gray-100 px-3 py-1.5 rounded-lg text-sm">
            </div>
            <div>
                <label class="block text-xs font-semibold text-gray-600 mb-1">Completed to</label>
                <input v-model="end_date" type="date" :min="start_date || undefined" @change="applyFilters()"
                    class="border border-gray-200 bg-gray-100 px-3 py-1.5 rounded-lg text-sm">
            </div>
            <button v-if="start_date || end_date" @click="clearDates()"
                class="py-1.5 px-3 rounded-lg text-sm font-semibold border border-gray-300 hover:bg-gray-100 cursor-pointer">
                Clear dates
            </button>
        </div>

        <div class="flex flex-col w-full" v-if="records.length">
            <div class="group relative w-full mb-3 p-4 border border-gray-200 rounded-lg" v-for="record in records"
                :key="record.id">
                <div class="mb-3">
                    <p class="whitespace-pre-line">{{ record.todo }}</p>
                    <div class="absolute top-2 right-2">
                        <button @click="toggleMenu(record.id)"
                            class="todo-menu-trigger px-2 leading-none text-xl font-black cursor-pointer rounded-md bg-white/90 hover:bg-gray-100 transition-opacity max-sm:opacity-100 opacity-0 group-hover:opacity-100 focus:opacity-100"
                            :class="{ 'opacity-100': open_menu === record.id }">
                            &#8943;
                        </button>
                        <div v-if="open_menu === record.id" :ref="(el) => (menu_ref = el)"
                            class="absolute right-0 top-full mt-1 z-20 min-w-40 bg-white border border-gray-200 rounded-lg shadow-lg py-1 flex flex-col">
                            <button v-for="option in status_options" :key="option.value"
                                @click="todo_update(record, option.value)" :disabled="record.status === option.value"
                                class="text-left text-sm px-3 py-1.5 hover:bg-gray-100 cursor-pointer disabled:text-gray-400 disabled:cursor-default disabled:hover:bg-transparent">
                                Mark as {{ option.label.toLowerCase() }}
                            </button>
                            <button @click="todo_delete(record.id)"
                                class="text-left text-sm px-3 py-1.5 text-red-600 hover:bg-red-50 cursor-pointer border-t border-gray-100">
                                Delete
                            </button>
                        </div>
                    </div>
                </div>
                <div class="flex items-center gap-2">
                    <span class="text-xs font-semibold px-2 py-1 rounded-md" :class="statusClass(record.status)">{{
                        statusLabel(record.status) }}</span>
                    <span v-if="record.completed_at" class="text-xs text-gray-600">Completed on {{
                        formatCompletedAt(record.completed_at) }}</span>
                </div>
            </div>
        </div>
        <div v-else>
            <p class="text-gray-600">{{ start_date || end_date ? 'No todos' : 'No todo' }}</p>
        </div>

        <div v-if="pagination.length > 1"
            class="w-full flex items-center justify-between bg-gray-100 rounded-lg p-3 mt-3">
            <span class="text-sm">{{ current_range }}</span>
            <div class="flex items-center">
                <button @click="setNewPage(page)" class="py-1 px-2 ml-1 border border-gray-300 rounded-md"
                    :class="{ 'bg-gray-300': current_page == page }" v-for="page in pagination" :key="page">{{
                        page
                    }}</button>
            </div>
        </div>
    </section>
</template>
<script setup lang="js">
const records = ref([]);
const create_todo = ref(false);
const token = ref(null);
const todo = ref("");
const status = ref("pending");
const processing = ref(false);
const success_message = ref("");
const open_menu = ref(null);
const menu_ref = ref(null);
const route = useRoute();
const router = useRouter();
const current_page = ref(route.query.page ?? 1);
const start_date = ref(route.query.start ?? "");
const end_date = ref(route.query.end ?? "");
const pagination = ref([]);
const current_range = ref("");

const status_options = [
    { value: "pending", label: "Pending" },
    { value: "in_progress", label: "In Process" },
    { value: "completed", label: "Completed" }
];

token.value = localStorage.getItem('auth_token');

onClickOutside(menu_ref, () => {
    open_menu.value = null;
}, { ignore: ['.todo-menu-trigger'] });

await fetchTodos();

function toggleMenu(id) {
    open_menu.value = open_menu.value === id ? null : id;
}

function buildQuery(page) {
    const query = { page };
    if (start_date.value) query.start = start_date.value;
    if (end_date.value) query.end = end_date.value;
    return query;
}

async function fetchTodos() {
    try {
        const params = new URLSearchParams(buildQuery(current_page.value));
        const data = await $fetch("/api/professional_todos/get?" + params.toString());
        records.value = data.data;
        pagination.value = data.pagination;
        current_range.value = data.current_range;
    } catch (e) {
        console.log(e);
    }
}

async function setNewPage(page) {
    if (page != '...') {
        current_page.value = page;
        router.push({ query: buildQuery(page) });
        await fetchTodos();
    }
}

async function applyFilters() {
    current_page.value = 1;
    router.push({ query: buildQuery(1) });
    await fetchTodos();
}

async function clearDates() {
    start_date.value = "";
    end_date.value = "";
    await applyFilters();
}

function showMessage(message) {
    success_message.value = message;
    setTimeout(() => {
        success_message.value = "";
    }, 3000);
}

async function todo_create() {
    try {
        if (!todo.value || todo.value.trim() == '') {
            return;
        }
        processing.value = true;
        success_message.value = "";
        const data = await $fetch("/api/professional_todos/create", {
            method: "POST",
            headers: {
                "Authorization": "Bearer " + token.value
            },
            body: {
                todo: todo.value,
                status: status.value
            }
        });
        if (data.status == 200) {
            todo.value = "";
            status.value = "pending";
            localStorage.setItem('auth_token', token.value);
            await fetchTodos();
            showMessage(data.message);
        }
    } catch (e) {
        console.log(e);
    } finally {
        processing.value = false;
    }
}

async function todo_update(record, new_status) {
    open_menu.value = null;
    try {
        const data = await $fetch("/api/professional_todos/update/" + record.id, {
            method: "PATCH",
            headers: {
                "Authorization": "Bearer " + token.value
            },
            body: {
                todo: record.todo,
                status: new_status
            }
        });
        if (data.status == 200) {
            await fetchTodos();
            showMessage(data.message);
        }
    } catch (e) {
        console.log(e);
        await fetchTodos();
    }
}

async function todo_delete(id) {
    open_menu.value = null;
    try {
        const data = await $fetch("/api/professional_todos/delete/" + id, {
            method: "DELETE",
            headers: {
                "Authorization": "Bearer " + token.value
            }
        });
        if (data.status == 200) {
            await fetchTodos();
            showMessage(data.message);
        }
    } catch (e) {
        console.log(e);
    }
}

function statusLabel(value) {
    return { pending: "Pending", in_progress: "In Process", completed: "Completed" }[value] ?? value;
}

function statusClass(value) {
    return {
        pending: "bg-gray-200 text-gray-700",
        in_progress: "bg-blue-100 text-blue-700",
        completed: "bg-green-100 text-green-700"
    }[value] ?? "bg-gray-200 text-gray-700";
}

function formatCompletedAt(value) {
    if (!value) {
        return "";
    }
    if (value.length <= 10) {
        return new Date(value + "T00:00:00").toLocaleDateString(undefined, { dateStyle: "full" });
    }
    return new Date(value.replace(" ", "T") + "Z").toLocaleString(undefined, {
        dateStyle: "full",
        timeStyle: "short"
    });
}

function setLocalStorage() {
    localStorage.setItem('auth_token', token.value);
}
</script>