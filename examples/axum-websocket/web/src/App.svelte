<script lang="ts">
import { Chat } from "./lib/api";
import { connect } from "./lib/client";
import { onMount } from "svelte";

let color = "#FF00FF";
let name = "John Doe";
let current_msg = "";

let chat = undefined;

$: messageStore = chat ? chat.messages(10n) : undefined;
$: counterStore = chat ? chat.count_to(10n) : undefined;

onMount(async () => {
    await reconnect();
});

async function reconnect() {
    const client = await connect("ws://localhost:8000/ws", () => {
        chat = undefined;
        setTimeout(() => reconnect(), 2000);
    });
    chat = Chat.init(client);
}

async function send() {
    try {
        await chat.send({ user: { name }, content: current_msg, color: color });
        current_msg = "";
    } catch (err) {
        alert(JSON.stringify(err));
    }
}

function onKeydown(e: KeyboardEvent) {
    if (e.key == "Enter") {
        send();
    }
}
</script>

<div class="container mx-auto h-screen flex flex-col items-center px-12">
    <div
        class="p:2 sm:p-6 justify-between flex flex-col bg-gray-200 my-32 rounded-xl shadow-xl w-full flex-grow">
        {#if !chat}
            <div
                class="self-center h-full flex items-center justify-center text-4xl text-gray-400">
                connecting...
            </div>
        {:else}
            {#if $counterStore && "err" in $counterStore}
                <span class="text-red-500">
                    {$counterStore.err.msg}
                </span>
            {:else if $counterStore && "data" in $counterStore}
                {$counterStore.data}
            {:else}
                0
            {/if}
            <div
                class="flex sm:items-center justify-between py-3 border-b-2 border-gray-300">
                <div class="relative flex items-center space-x-4">
                    <div class="relative">
                        <div
                            class="rounded-full overflow-hidden grid items-center justify-center w-10 h-10 sm:w-16 h-10 sm:h-16 border-2">
                            <input
                                type="color"
                                class="h-full border-none outline-none cursor-pointer"
                                bind:value="{color}" />
                            <div
                                class="absolute w-full h-full p-4 pointer-events-none">
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke-width="1.5"
                                    stroke="currentColor"
                                    class="w-full h-full mx-auto text-white pointer-events-none">
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        d="M4.098 19.902a3.75 3.75 0 005.304 0l6.401-6.402M6.75 21A3.75 3.75 0 013 17.25V4.125C3 3.504 3.504 3 4.125 3h5.25c.621 0 1.125.504 1.125 1.125v4.072M6.75 21a3.75 3.75 0 003.75-3.75V8.197M6.75 21h13.125c.621 0 1.125-.504 1.125-1.125v-5.25c0-.621-.504-1.125-1.125-1.125h-4.072M10.5 8.197l2.88-2.88c.438-.439 1.15-.439 1.59 0l3.712 3.713c.44.44.44 1.152 0 1.59l-2.879 2.88M6.75 17.25h.008v.008H6.75v-.008z"
                                    ></path>
                                </svg>
                            </div>
                        </div>
                    </div>
                    <div class="flex flex-col leading-tight">
                        <div class="text-2xl mt-1 flex items-center">
                            <input
                                class="bg-transparent text-gray-700 outline-none"
                                placeholder="Name"
                                bind:value="{name}" />
                        </div>
                    </div>
                </div>
            </div>
            <div
                id="messages"
                class="flex flex-col space-y-4 pt-10 overflow-y-auto scrollbar-thumb-blue scrollbar-thumb-rounded scrollbar-track-blue-lighter scrollbar-w-2 scrolling-touch flex-grow">
                {#if $messageStore && "data" in $messageStore}
                    {#each $messageStore.data as msg}
                        {#if msg.user.name == name}
                            <div class="chat-message">
                                <div class="flex items-end justify-end">
                                    <div
                                        class="flex flex-col space-y-2 text max-w-xs mx-2 order-1 items-end">
                                        <div>
                                            <span
                                                class="px-4 py-2 rounded-lg inline-block rounded-br-none bg-purple-300 text-gray-700"
                                                >{msg.content}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        {:else}
                            <div class="chat-message">
                                <div class="flex items-end">
                                    <div
                                        class="flex flex-col space-y-2 text max-w-xs mx-2 order-2 items-start">
                                        <div>
                                            <span
                                                class="rounded-lg inline-block rounded-bl-none bg-gray-200 border-4 text-gray-600 relative"
                                                style="border-color: {msg.color}">
                                                <div
                                                    class="absolute w-full h-full opacity-20"
                                                    style="background-color: {msg.color}">
                                                </div>
                                                <div
                                                    class="text-xs px-4 py-1 text-white bold"
                                                    style="background-color: {msg.color}">
                                                    {msg.user.name}
                                                </div>
                                                <div
                                                    class="px-4 py-2 inline-block text-gray-600 relative">
                                                    {msg.content}
                                                </div>
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        {/if}
                    {/each}
                {/if}
            </div>
            <div class="border-t-2 border-gray-300 pt-4 mb-2 sm:mb-0">
                <div class="relative flex">
                    <input
                        type="text"
                        placeholder="Write your message!"
                        class="w-full focus:outline-none focus:placeholder-gray-400 text-gray-600 placeholder-gray-600 bg-gray-300 rounded-md py-3 pl-4"
                        bind:value="{current_msg}"
                        on:keydown="{onKeydown}" />
                    <div
                        class="absolute right-0 items-center inset-y-0 hidden sm:flex">
                        <button
                            type="button"
                            class="inline-flex items-center justify-center rounded-lg px-4 py-3 transition duration-500 ease-in-out text-white bg-blue-600 enabled:hover:bg-blue-400 focus:outline-none disabled:opacity-50 enabled:hover:cursor-pointer"
                            on:click="{send}"
                            disabled="{name == '' || current_msg == ''}">
                            <span class="font-bold">Send</span>
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 20 20"
                                fill="currentColor"
                                class="h-6 w-6 ml-2 transform rotate-90">
                                <path
                                    d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z"
                                ></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        {/if}
    </div>
</div>
