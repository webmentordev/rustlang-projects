<template>
    <div class="max-w-2xl w-full mx-auto mb-3 pt-3 pb-5 bg-white" v-if="record">
        <NuxtLink to="/tasks"
            class="py-2 flex items-center justify-center px-4 rounded-lg m-auto bg-black text-white font-semibold mb-4 w-fit">
            Daily tasks
        </NuxtLink>
        <img :src="record.avatar_url" alt="Profile pic" class="rounded-full m-auto mb-2 w-25 h-25 object-cover">
        <div class="text-center pb-0 mb-3">
            <h1 class="text-xl font-bold tracking-wide uppercase mb-1">{{ record.name }}</h1>
            <div class="text-xs font-semibold uppercase tracking-widest mb-1">{{ record.profession }}</div>
            <div class="w-25 my-2 bg-gray-500 h-px m-auto"></div>
            <div class="text-xs font-semibold uppercase tracking-widest mb-1 text-gray-700">{{ record.work }}</div>
            <div class="text-xs text-gray-600 mb-1">{{ record.phone }} • {{ record.address }} • {{ record.email }}
            </div>
            <div v-if="record.dob" class="text-xs text-gray-600">Date of birth - {{ record.dob }}</div>
            <div class="flex m-auto w-fit mt-1">
                <a :href="social.link" target="_blank" v-if="social in record.socials"
                    class="text-xs text-blue-600 hover:underline pt-1">
                    <img :src="social.icon_url" width="70" :alt="social.title">
                </a>
                <span class="text-gray-600 mx-3">•</span>
            </div>
        </div>

        <div class="mb-5">
            <div class="text-base font-semibold uppercase tracking-widest border-b border-gray-800 pb-2 mb-3">
                Summary
            </div>
            <p class="text-sm leading-relaxed">
                {{ record.summary }}
            </p>
        </div>

        <div class="mb-12">
            <div class="font-semibold uppercase tracking-widest border-b border-gray-800 pb-2 mb-3">Skills
            </div>
            <div class="grid grid-cols-2 gap-2 text-sm" v-if="record.skills.length > 0">
                <div v-for="skill in record.skills">
                    <span class="font-medium">{{ skill.tags }}</span>
                    <span class="text-xs text-gray-600 ml-1">({{ skill.level }})</span>
                </div>
            </div>

            <div class="text-base font-semibold uppercase tracking-widest border-b border-gray-800 pb-2 mb-3 mt-6">
                Work
                Experience
            </div>

            <div class="mb-2 avoid-break" v-if="record.work_experience.length > 0"
                v-for="work in record.work_experience">
                <div class="flex justify-between items-baseline mb-1">
                    <span class="font-semibold text-sm">{{ work.designation }}</span>
                    <span class="text-xs font-semibold">{{ work.start }} - {{ work.end }} | {{ work.type }}</span>
                </div>
                <div class="text-gray-600 text-sm mb-2">{{ work.org }}, {{ work.location }}</div>
                <div class="text-sm mb-2">{{ work.summary }}</div>
            </div>
        </div>
        <div class="mb-4 page-break-before">
            <div class="text-lg font-semibold uppercase tracking-widest border-b border-gray-800 pb-2 mb-3">Projects
            </div>

            <div class="mb-2 avoid-break" v-if="record.projects.length > 0" v-for="project in record.projects">
                <div class="flex justify-between items-baseline mb-1">
                    <div class="flex items-center gap-2">
                        <span class="font-semibold text-sm">{{ project.title }}</span>
                        <a v-if="project.github_link" :href="project.github_link" target="_blank" class="inline-block">
                            <img src="https://api.iconify.design/skill-icons:github-dark.svg" width="16" alt="GitHub">
                        </a>
                        <a v-if="project.working_link" :href="project.working_link" target="_blank"
                            class="inline-block">
                            <img src="https://api.iconify.design/pepicons-pop:internet.svg?color=%231a1a19" width="19"
                                alt="Website">
                        </a>
                    </div>
                    <span class="text-xs font-semibold">{{ project.start }} - {{ project.end == '' ? 'Present' :
                        project.end }} </span>
                </div>
                <div class="text-xs text-gray-600 mb-2">{{ project.stack }}</div>
                <div class="text-sm mb-1">{{ project.summary }}</div>
                <ul class="text-sm list-disc list-inside ml-2 space-y-1" v-if="project.summary_points.length > 0">
                    <li v-for="line in project.summary_points">{{ line }}</li>
                </ul>
            </div>
        </div>

        <div class="mb-12">
            <div class="text-lg font-semibold uppercase tracking-widest border-b border-gray-800 pb-2 mb-3">
                Education
            </div>

            <div class="mb-4 avoid-break" v-for="(education, index) in record.education" :key="index">
                <div class="flex justify-between items-baseline mb-1">
                    <span class="font-semibold text-sm">{{ education.degree }}</span>
                    <span class="text-xs font-semibold">{{ education.start }} - {{ education.end }}</span>
                </div>
                <div class="text-gray-600 text-sm mb-2">{{ education.institute }}, {{ education.location }}</div>
                <div class="text-sm">{{ education.summary }}</div>
            </div>
        </div>
        <button class="py-2 px-4 bg-black rounded-lg text-white font-semibold cursor-pointer" @click="refresh_data()">{{
            processing ?
                'Fetching...' : 'Update data' }}</button>
    </div>
</template>
<script setup lang="js">
const record = ref(null);
const processing = ref(false);
const avatar = ref(null);

try {
    const { data } = await useFetch("/api/info/get");
    record.value = data.value.data;
    avatar.value = data.value.data.avatar_url;
    localStorage.setItem('avatar', JSON.stringify({
        "avatar": avatar.value,
        "name": data.value.data.nickname
    }));
} catch (e) {
    console.log(e);
}

async function refresh_data() {
    try {
        processing.value = true;
        const data = await $fetch("/api/info/update", {
            method: "POST"
        });
        record.value = data.data;
        avatar.value = data.data.avatar_url;
        localStorage.setItem('avatar', JSON.stringify({
            "avatar": avatar.value,
            "name": data.data.nickname
        }));
    } catch (e) {
        console.log(e);
    } finally {
        processing.value = false;
    }
}

</script>