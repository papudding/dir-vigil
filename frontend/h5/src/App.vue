<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { showNotify, showToast } from 'vant'
import { getStatus, keepAlive, getCaptcha } from './api/DirVigil'

const status = ref({ remainsSeconds: 0, comment: '' })
const two_fa_code = ref('')
onMounted(async () => {
  await parseStatus()
})

// 验证码相关逻辑
const captcha = ref('')
const captchaInput = ref('')
const showCaptcha = ref(false)

const isCaptchaValid = computed(() => {
  return captchaInput.value && captchaInput.value.toUpperCase() === captcha.value.toUpperCase()
})

// 生成随机验证码
const generateCaptcha = async () => {
  try {
    const res: string = await getCaptcha()
    captcha.value = res
    showCaptcha.value = true

  } catch (error) {
    interface ApiError {
      response: {
        status: number
        data: string
      }
    }

    if ((error as ApiError).response?.data) {
      showNotify({ type: 'warning', message: (error as ApiError).response?.data })
    } else {
      showToast('Request failed, please try again')
    }
  }

}

const parseStatus = async () => {
  const res: string = await getStatus()
  const json = JSON.parse(JSON.stringify(res))
  status.value = { remainsSeconds: json.remainsSeconds, comment: json.comment }
}

const resetAll = () => {
  two_fa_code.value = ''
  captcha.value = ''
  showCaptcha.value = false
  captchaInput.value = ''
}

const beAlive = async () => {
  if (!two_fa_code.value) {
    showNotify({ type: 'warning', message: '请输入2FA验证码' })
    return
  }
  try {
    const res: string = await keepAlive(two_fa_code.value, captchaInput.value)
    resetAll()
    showNotify({ type: 'success', message: res })
    parseStatus()
  } catch (error) {
    interface ApiError {
      response: {
        status: number
        data: string
      }
    }
    const resp = (error as ApiError).response;
    if (resp?.status === 403) {
      showNotify({ type: 'warning', message: resp?.data })
      resetAll()
    } else {
      showToast('Request failed, please try again')
    }
  }
}
</script>

<template>

  <div class="app">

    <div style="margin: 20px 0; text-align: center; font-size: 18px;">Remaining Time
      <van-count-down millisecond :time="status.remainsSeconds * 1000" format="HH:mm:ss:SSS" />
    </div>
    <div style="margin: 20px 0; text-align: center; font-size: 18px;">{{ status.comment }}</div>
    <!-- 2FA 输入框 -->
    <div class="two-fa-container">
      <div v-if="showCaptcha" class="captcha-display">{{ captcha }}</div>
      <van-field v-if="showCaptcha" v-model="captchaInput" placeholder="Please enter the captcha" :maxlength="6" />
      <van-field v-show="isCaptchaValid" v-model="two_fa_code" placeholder="Please enter the 2FA code" :maxlength="6"
        :formatter="(value) => value.replace(/[^0-9]/g, '')" />
      <van-button style="margin-top: 5px;" v-show="!isCaptchaValid" type="default" @click="generateCaptcha">Get
        captcha</van-button>
      <van-button style="margin-top: 5px;" v-show="isCaptchaValid" type="primary" @click="beAlive">Submit
        2FA</van-button>
    </div>


  </div>
</template>

<style scoped>
.app {
  height: 100vh;
  padding: 20px;
  background-color: rgb(255, 255, 255);
  color: black;
}

.captcha-display {
  font-size: 24px;
  font-weight: bold;
  letter-spacing: 5px;
  text-align: center;
  margin-bottom: 15px;
  user-select: none;
}

.two-fa-container {
  margin: 20px 0;
  padding: 20px;
  background-color: #f5f5f5;
  border-radius: 8px;
  display: flex;
  justify-content: center;
  align-items: center;
  flex-direction: column;
}
</style>
