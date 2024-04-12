import { createApp, reactive } from 'vue'
import './style.css'
import App from './App.vue'

//const socket = new WebSocket('ws://localhost:3000');

type BombData = {
  pos: { x: number, y: number, z: number },
  isPlanted: boolean,
}

enum PlayerType {
  Unknown,
  Spectator,
  Local,
  Enemy,
  Team
}

type PlayerData = {
  pos: { x: number, y: number, z: number },
  yaw: number,
  playerType: PlayerType,
  hasBomb: boolean,
  hasAwp: boolean,
  isScoped: boolean
}

type Radardata = {
  freq: number,
  ingame: boolean,

  bombPlanted: boolean,
  bombExploded: boolean,
  bombBeingDefused: boolean,
  bombCanDefuse: boolean,
  bombDefuseLength: number,
  bombDefuseTimeleft: number,
  bombDefuseEnd: number,

  mapName: string
  playerData: [BombData | PlayerData]
}

export const radardata = reactive({
    freq: 144,
    ingame: false
})

createApp(App).mount('#app')