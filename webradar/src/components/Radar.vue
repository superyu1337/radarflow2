<script setup lang = "ts">
    import { onMounted, ref } from 'vue';
    import { radardata } from '../main';

    const canvas = ref(null);
    const localColor = "#109856"
    const teamColor = "#68a3e5"
    const enemyColor = "#ec040b"
    const bombColor = "#eda338"
    const textColor = "#d1d1d1"

    onMounted(() => {

        console.log(radardata)

        const ctx = canvas.value.getContext('2d');

        ctx.fillStyle = "#0f0f0f";
        ctx.fillRect(0, 0, 1024, 1024);

        if (radardata.ingame) { 

        } else {
            ctx.font = "100px Arial";
            ctx.textAlign = "center"
            ctx.fillStyle = textColor
            ctx.fillText("Disconnected", 1024/2, 1024/2); 
        }
    });

</script>

<template>
     <canvas id="canvas" ref="canvas" width="1024" height="1024"></canvas>
</template>

<style>
    canvas {
        aspect-ratio: 1 / 1;
        width: clamp(100vw, 100%, 100vh)
    }
</style>