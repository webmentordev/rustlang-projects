<template>
  <div class="max-w-xl m-auto w-full p-6">
    <div class="flex items-center mb-1" v-if="!ws">
      <div class="flex items-center">
        <input type="text" v-model="room_id" placeholder="RoomID"
          class="py-1 px-2 bg-gray-100 border border-gray-200 w-full outline-none focus:outline-none">
        <input type="text" v-model="name" placeholder="Your name"
          class="py-1 px-2 bg-gray-100 border border-gray-200 w-full outline-none focus:outline-none">
      </div>
      <button class="py-1 px-3 bg-indigo-600 text-white rounded-sm ml-2" @click="connect">Connect</button>
    </div>
    <div class="flex flex-col" v-else>
      <div class="flex items-center">
        <input type="text" v-model="message" placeholder="Message"
          class="py-1 px-2 bg-gray-100 border border-gray-200 w-full outline-none focus:outline-none">
        <button class="py-1 px-3 bg-black text-white rounded-sm ml-2" @click="send">Send</button>
      </div>
      <div v-if="messages.length" class="my-2 ml-6">
        <li class="mb-1" v-for="message in messages"><strong>{{ message.user_id == userID ? 'You' : message.name
            }}</strong>: {{ message.message }}</li>
      </div>
    </div>
    <div v-if="connected" class="flex items-center mt-3">
      <p class="py-2 bg-green-500/10 border border-green-500 px-4">WS Connected ✅</p>
      <button class="py-1 px-3 bg-red-600 text-white rounded-sm ml-2" @click="disconnect">Disconnect</button>
    </div>
    <p v-else>No connected ❌</p>
  </div>
</template>

<script setup lang="js">
const message = ref("");
const room_id = ref("");
const name = ref("");
const userID = ref("");

const ws = ref(null);
const connected = ref(false);
const messages = ref([]);

const connect = () => {
  setupUUID();
  if (room_id.value) {
    ws.value = new WebSocket("http://127.0.0.1:3099/ws/" + room_id.value)
    ws.value.onmessage = (event) => {
      messages.value.push(JSON.parse(event.data))
    }
    ws.value.onopen = () => {
      connected.value = true;
    };
    ws.value.onerror = () => {
      connected.value = false;
    };
  }
}

const send = () => {
  if (ws.value && message.value) {
    ws.value.send(JSON.stringify({
      message: message.value,
      user_id: userID.value,
      name: name.value
    }));
    message.value = ''
  }
}

const disconnect = () => {
  if (ws.value) {
    ws.value.close(1000, 'Work complete');
    connected.value = false;
    room_id.value = "";
    ws.value = null;
  }
}

function setupUUID() {
  const user_id = useCookie('user_id');
  if (!user_id.value) {
    const uniqueId = generateUUID();
    user_id.value = uniqueId;
    userID.value = uniqueId;
  } else {
    userID.value = user_id.value;
  }
}

function generateUUID() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}
</script>