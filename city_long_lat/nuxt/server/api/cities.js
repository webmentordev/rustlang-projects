export default defineEventHandler(async () => {
    try {
        const config = useRuntimeConfig();
        const data = await $fetch(`${config.apiUrl}/get-all`, {
            method: "GET",
            headers: {
                "Accept": "application/json",
                "Content-Type": "application/json",
            }
        });
        return data;
    } catch (e) {
        return {
            "error": "APi failed"
        }
    }
})