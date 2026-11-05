<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { isAxiosError } from 'axios'
import { getCaptcha, getStatus, keepAlive } from './api/DirVigil'
import type { StatusResponse } from './api/DirVigil'

const WARNING_SECONDS = 60 * 60
const CRITICAL_SECONDS = 10 * 60
const STATUS_POLL_INTERVAL = 20_000
const CODE_LENGTH = 6
const DEFAULT_COMMENT = '^-^'

type Phase = 'normal' | 'warning' | 'critical' | 'expired'

const hasStatus = ref(false)
const initialLoading = ref(true)
const refreshing = ref(false)
const offline = ref(false)
const status = ref<StatusResponse>({ remainsSeconds: 0, comment: '' })
const deadline = ref(0)
const now = ref(Date.now())
const lastKeptAt = ref<number | null>(null)

const captchaCode = ref('')
const captchaInput = ref('')
const captchaLoading = ref(false)
const captchaError = ref('')
const captchaInputEl = ref<HTMLInputElement | null>(null)

const otpInput = ref('')
const otpInputEl = ref<HTMLInputElement | null>(null)
const submitting = ref(false)
const submitError = ref('')

const toastMessage = ref('')
const toastVisible = ref(false)
let toastTimer: ReturnType<typeof setTimeout> | undefined

function showToast(message: string): void {
  toastMessage.value = message
  toastVisible.value = true
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => {
    toastVisible.value = false
  }, 2_600)
}

const pad = (value: number): string => String(value).padStart(2, '0')

const remainsSeconds = computed(() => Math.max(0, Math.ceil((deadline.value - now.value) / 1000)))

const timeParts = computed(() => {
  const total = remainsSeconds.value
  return {
    days: Math.floor(total / 86_400),
    hours: Math.floor((total % 86_400) / 3_600),
    minutes: Math.floor((total % 3_600) / 60),
    seconds: total % 60,
  }
})

const phase = computed<Phase>(() => {
  if (remainsSeconds.value <= 0) return 'expired'
  if (remainsSeconds.value <= CRITICAL_SECONDS) return 'critical'
  if (remainsSeconds.value <= WARNING_SECONDS) return 'warning'
  return 'normal'
})

const statusLabel = computed(() => {
  switch (phase.value) {
    case 'expired':
      return '已到期'
    case 'critical':
      return '请立即续命'
    case 'warning':
      return '即将到期'
    default:
      return '保护中'
  }
})

const urgencyHint = computed(() => {
  if (phase.value === 'critical') return '剩余不足 10 分钟，请立即完成续命'
  if (phase.value === 'warning') return '剩余不足 1 小时，建议尽快续命'
  return ''
})

const commentText = computed(() => {
  const value = status.value.comment?.trim() ?? ''
  return value && value !== DEFAULT_COMMENT ? value : ''
})

const countdownAria = computed(() => {
  const { days, hours, minutes, seconds } = timeParts.value
  const parts = [days > 0 ? `${days} 天` : '', `${hours} 小时`, `${minutes} 分`, `${seconds} 秒`]
  return `距离自动清理还有 ${parts.filter(Boolean).join(' ')}`
})

const lastKeptText = computed(() => {
  if (lastKeptAt.value === null) return ''
  const seconds = Math.max(0, Math.floor((now.value - lastKeptAt.value) / 1000))
  if (seconds < 60) return '刚刚续命成功'
  if (seconds < 3_600) return `${Math.floor(seconds / 60)} 分钟前续命成功`
  return `${Math.floor(seconds / 3_600)} 小时前续命成功`
})

const captchaReady = computed(() => captchaCode.value.length > 0)
const captchaValid = computed(
  () => captchaReady.value && captchaInput.value.toUpperCase() === captchaCode.value,
)
const canSubmit = computed(
  () =>
    captchaValid.value &&
    otpInput.value.length === CODE_LENGTH &&
    !submitting.value &&
    !offline.value &&
    phase.value !== 'expired',
)

let statusRequestInFlight = false
let tickTimer: ReturnType<typeof setInterval> | undefined
let pollTimer: ReturnType<typeof setInterval> | undefined

const BACKEND_ERROR_MESSAGES: Record<string, string> = {
  'Incorrect captcha': '验证码不正确，请重新输入或换一个',
  'No captcha generated': '请先获取验证码',
  'No captcha code provided': '请输入验证码',
  'Invalid 2FA code': '2FA 动态码错误或已过期，请重新输入',
  'Invalid 2FA code, request a new captcha': '2FA 动态码错误，安全验证码已失效，请重新获取',
  'No 2FA code provided': '请输入 2FA 动态码',
  'Too many attempts, request a new captcha': '尝试次数过多，请重新获取验证码',
  'Failed to read TOTP secret': '服务端读取 2FA 密钥失败，请检查日志',
}

function resolveErrorMessage(error: unknown): string {
  if (isAxiosError(error)) {
    if (!error.response) return '无法连接守护服务，请确认后端或容器正在运行。'
    const raw = typeof error.response.data === 'string' ? error.response.data.trim() : ''
    if (BACKEND_ERROR_MESSAGES[raw]) return BACKEND_ERROR_MESSAGES[raw]
    if (error.response.status === 403) return '验证未通过，请重新获取验证码后再试。'
    if (error.response.status === 429) return '请求过于频繁，请稍后再试。'
    return `请求失败（HTTP ${error.response.status}），请稍后重试。`
  }
  return '发生未知错误，请稍后重试。'
}

async function loadStatus(options: { silent?: boolean } = {}): Promise<void> {
  if (statusRequestInFlight) return
  statusRequestInFlight = true
  if (!options.silent) refreshing.value = true
  try {
    const response = await getStatus()
    const remaining = Number(response?.remainsSeconds)
    status.value = {
      remainsSeconds: Number.isFinite(remaining) ? Math.max(0, remaining) : 0,
      comment: typeof response?.comment === 'string' ? response.comment : '',
    }
    deadline.value = Date.now() + status.value.remainsSeconds * 1000
    now.value = Date.now()
    hasStatus.value = true
    offline.value = false
  } catch {
    offline.value = true
  } finally {
    statusRequestInFlight = false
    refreshing.value = false
    initialLoading.value = false
  }
}

function refreshStatus(): void {
  void loadStatus()
}

async function fetchCaptcha(): Promise<void> {
  if (captchaLoading.value) return
  captchaLoading.value = true
  captchaError.value = ''
  submitError.value = ''
  try {
    const code = await getCaptcha()
    const normalized = String(code ?? '').trim().toUpperCase()
    if (normalized.length !== CODE_LENGTH) {
      captchaError.value = '验证码获取失败，请重试'
      return
    }
    captchaCode.value = normalized
    captchaInput.value = ''
    await nextTick()
    captchaInputEl.value?.focus()
  } catch (error) {
    captchaError.value = resolveErrorMessage(error)
  } finally {
    captchaLoading.value = false
  }
}

function onCaptchaInput(event: Event): void {
  const input = event.target as HTMLInputElement
  const cleaned = input.value.replace(/[^0-9a-zA-Z]/g, '').slice(0, CODE_LENGTH).toUpperCase()
  input.value = cleaned
  captchaInput.value = cleaned

  if (cleaned.length === CODE_LENGTH && cleaned !== captchaCode.value) {
    captchaError.value = '验证码不匹配，请核对后重试'
    return
  }
  captchaError.value = ''

  if (cleaned.length === CODE_LENGTH) {
    void nextTick(() => {
      otpInputEl.value?.focus()
    })
  }
}

function onOtpInput(event: Event): void {
  const input = event.target as HTMLInputElement
  const cleaned = input.value.replace(/\D/g, '').slice(0, CODE_LENGTH)
  input.value = cleaned
  otpInput.value = cleaned
  if (submitError.value) submitError.value = ''
}

async function submit(): Promise<void> {
  if (submitting.value) return

  if (offline.value) {
    submitError.value = '与服务端的连接已断开，请先刷新状态。'
    return
  }
  if (!captchaReady.value) {
    submitError.value = '请先点击“获取验证码”。'
    return
  }
  if (!captchaValid.value) {
    captchaError.value = '请输入正确的验证码'
    captchaInputEl.value?.focus()
    return
  }
  if (otpInput.value.length !== CODE_LENGTH) {
    submitError.value = `请输入 ${CODE_LENGTH} 位 2FA 动态码。`
    otpInputEl.value?.focus()
    return
  }

  submitting.value = true
  submitError.value = ''
  try {
    await keepAlive(otpInput.value, captchaInput.value)
    showToast('续命成功，倒计时已重置')
    lastKeptAt.value = Date.now()
    captchaCode.value = ''
    captchaInput.value = ''
    otpInput.value = ''
    captchaError.value = ''
    await loadStatus()
  } catch (error) {
    const status = isAxiosError(error) ? error.response?.status : undefined
    const raw =
      isAxiosError(error) && typeof error.response?.data === 'string'
        ? error.response.data.trim()
        : ''
    submitError.value = resolveErrorMessage(error)

    if (status === 403 || status === 429) {
      otpInput.value = ''
      // 2FA 错误后服务端已立即作废验证码；429 也会清空验证码。
      const captchaInvalidated =
        status === 429 ||
        raw.startsWith('Invalid 2FA code') ||
        raw === 'No captcha generated'
      if (captchaInvalidated) {
        captchaCode.value = ''
        captchaInput.value = ''
        captchaError.value = ''
      }
      await nextTick()
      if (raw === 'Incorrect captcha') {
        captchaInputEl.value?.focus()
      } else if (!captchaInvalidated) {
        otpInputEl.value?.focus()
      }
    }
  } finally {
    submitting.value = false
  }
}

function handleVisibilityChange(): void {
  if (document.visibilityState === 'visible') void loadStatus({ silent: true })
}

onMounted(() => {
  void loadStatus()
  tickTimer = setInterval(() => {
    now.value = Date.now()
  }, 250)
  pollTimer = setInterval(() => {
    if (document.visibilityState === 'visible' && !submitting.value) {
      void loadStatus({ silent: true })
    }
  }, STATUS_POLL_INTERVAL)
  document.addEventListener('visibilitychange', handleVisibilityChange)
})

onBeforeUnmount(() => {
  if (tickTimer) clearInterval(tickTimer)
  if (pollTimer) clearInterval(pollTimer)
  if (toastTimer) clearTimeout(toastTimer)
  document.removeEventListener('visibilitychange', handleVisibilityChange)
})
</script>

<template>
  <div class="app">
    <div class="backdrop" aria-hidden="true">
      <span class="backdrop__glow backdrop__glow--indigo"></span>
      <span class="backdrop__glow backdrop__glow--teal"></span>
      <span class="backdrop__grid"></span>
    </div>

    <main class="shell">
      <aside class="intro">
        <div class="brand">
          <span class="brand__mark" aria-hidden="true">
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M12 3.2l6.6 2.8v4.9c0 4.1-2.7 7-6.6 8.3-3.9-1.3-6.6-4.2-6.6-8.3V6L12 3.2z" />
              <path d="M9.3 11.6l1.9 1.9 3.5-3.7" />
            </svg>
          </span>
          <span class="brand__name">dir-vigil</span>
        </div>
        <h1 class="intro__title">为你的数据<br /><em>站最后一班岗</em></h1>
        <p class="intro__desc">
          类似“失能开关”的目录保护工具。在倒计时结束前回到这里完成续命，守护目录才会继续保留；超时未续命，它会被自动清空且无法恢复。
        </p>
        <ul class="intro__list">
          <li><span class="dot dot--teal"></span>验证码 + 2FA 双重校验</li>
          <li><span class="dot dot--indigo"></span>验证通过即重置倒计时</li>
          <li><span class="dot dot--amber"></span>到期自动清理，不可恢复</li>
        </ul>
        <p class="intro__foot">建议收藏本页，或添加到手机主屏幕，避免错过续命时间。</p>
      </aside>

      <section class="panel" :class="`panel--${phase}`">
        <header class="panel__head">
          <div class="panel__brand">
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path d="M12 3.2l6.6 2.8v4.9c0 4.1-2.7 7-6.6 8.3-3.9-1.3-6.6-4.2-6.6-8.3V6L12 3.2z" />
              <path d="M9.3 11.6l1.9 1.9 3.5-3.7" />
            </svg>
            <span>dir-vigil</span>
          </div>
          <div
            v-if="hasStatus"
            class="status"
            :class="`status--${phase}`"
            role="status"
            aria-live="polite"
          >
            <span class="status__dot"></span>
            <span>{{ statusLabel }}</span>
          </div>
          <button
            class="icon-btn"
            type="button"
            :disabled="refreshing"
            aria-label="刷新守护状态"
            @click="refreshStatus"
          >
            <svg
              class="icon"
              :class="{ 'icon--spin': refreshing }"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.9"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path d="M20 11.5a8 8 0 1 1-2.4-5.7" />
              <path d="M20 4.5v6h-6" />
            </svg>
          </button>
        </header>

        <div v-if="initialLoading" class="state state--loading" role="status">
          <span class="spinner" aria-hidden="true"></span>
          <p>正在连接守护服务…</p>
        </div>

        <div v-else-if="!hasStatus" class="state state--error">
          <div class="state__icon" aria-hidden="true">!</div>
          <h2>无法连接守护服务</h2>
          <p>请确认后端或容器仍在运行，然后重试。</p>
          <button class="btn btn--primary" type="button" :disabled="refreshing" @click="refreshStatus">
            {{ refreshing ? '连接中…' : '重新连接' }}
          </button>
        </div>

        <template v-else>
          <div v-if="offline" class="banner" role="alert">
            <span class="banner__icon" aria-hidden="true">!</span>
            <div class="banner__body">
              <strong>与守护服务的连接已断开</strong>
              <p>倒计时仅供参考，请确认服务状态后重试。</p>
            </div>
            <button class="banner__action" type="button" @click="refreshStatus">重试</button>
          </div>

          <div class="countdown" :class="`countdown--${phase}`">
            <p class="countdown__label">
              距离自动清理还有
              <span v-if="timeParts.days > 0" class="countdown__days">{{ timeParts.days }} 天</span>
            </p>
            <div class="clock" role="timer" :aria-label="countdownAria">
              <div class="clock__cell">
                <span class="clock__value">{{ pad(timeParts.hours) }}</span>
                <span class="clock__unit">时</span>
              </div>
              <span class="clock__colon" aria-hidden="true">:</span>
              <div class="clock__cell">
                <span class="clock__value">{{ pad(timeParts.minutes) }}</span>
                <span class="clock__unit">分</span>
              </div>
              <span class="clock__colon" aria-hidden="true">:</span>
              <div class="clock__cell">
                <span class="clock__value">{{ pad(timeParts.seconds) }}</span>
                <span class="clock__unit">秒</span>
              </div>
            </div>
            <p v-if="urgencyHint" class="countdown__hint" role="status">{{ urgencyHint }}</p>
            <p v-if="commentText" class="countdown__comment">备注：{{ commentText }}</p>
            <p v-if="lastKeptText" class="countdown__kept">{{ lastKeptText }}</p>
          </div>

          <div v-if="phase === 'expired'" class="state state--expired">
            <div class="state__icon" aria-hidden="true">⚠</div>
            <h2>倒计时已结束</h2>
            <p>监控目录可能已被自动清理。若服务已重启，刷新状态后会重新计时。</p>
            <button class="btn btn--primary" type="button" :disabled="refreshing" @click="refreshStatus">
              {{ refreshing ? '刷新中…' : '刷新状态' }}
            </button>
          </div>

          <form v-else class="keep-form" novalidate @submit.prevent="submit">
            <div class="form__head">
              <h2>续命验证</h2>
              <p>完成双重验证后，倒计时会立即重置。</p>
            </div>

            <div class="field">
              <div class="field__head">
                <label class="field__label" for="captcha-input">安全验证码</label>
                <button
                  class="field__action"
                  type="button"
                  :disabled="captchaLoading"
                  @click="fetchCaptcha"
                >
                  <svg
                    class="icon"
                    :class="{ 'icon--spin': captchaLoading }"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M20 11.5a8 8 0 1 1-2.4-5.7" />
                    <path d="M20 4.5v6h-6" />
                  </svg>
                  {{ captchaLoading ? '获取中…' : captchaReady ? '换一个' : '获取验证码' }}
                </button>
              </div>
              <div
                v-if="captchaReady"
                class="captcha"
                role="img"
                :aria-label="`验证码 ${captchaCode.split('').join(' ')}`"
              >
                {{ captchaCode }}
              </div>
              <input
                id="captcha-input"
                ref="captchaInputEl"
                class="input"
                :class="{ 'input--error': captchaError }"
                type="text"
                inputmode="text"
                autocomplete="off"
                autocapitalize="characters"
                spellcheck="false"
                :maxlength="CODE_LENGTH"
                :placeholder="captchaReady ? '请输入上方显示的验证码' : '点击右上角获取验证码'"
                :value="captchaInput"
                @input="onCaptchaInput"
                @keydown.enter.prevent="submit"
              />
              <p v-if="captchaError" class="field__error" role="alert">{{ captchaError }}</p>
            </div>

            <div class="field">
              <div class="field__head">
                <label class="field__label" for="otp-input">2FA 动态码</label>
                <span class="field__hint">6 位数字 · 每 30 秒刷新</span>
              </div>
              <input
                id="otp-input"
                ref="otpInputEl"
                class="input input--otp"
                type="text"
                inputmode="numeric"
                autocomplete="one-time-code"
                :maxlength="CODE_LENGTH"
                placeholder="000000"
                :value="otpInput"
                @input="onOtpInput"
                @keydown.enter.prevent="submit"
              />
            </div>

            <p v-if="submitError" class="form__error" role="alert">{{ submitError }}</p>

            <button class="btn btn--primary btn--block" type="submit" :disabled="!canSubmit">
              <span v-if="submitting" class="spinner spinner--sm" aria-hidden="true"></span>
              {{ submitting ? '验证中…' : '确认续命' }}
            </button>

            <p class="form__tip">续命成功后倒计时会重新开始；若服务重启，计时也会重置。</p>
          </form>

          <footer class="panel__foot">
            {{ phase === 'expired' ? '服务可能已停止运行' : '请定期回来续命，避免数据被自动清理' }}
          </footer>
        </template>
      </section>
    </main>

    <footer class="page-foot">dir-vigil · 数据无价，操作需谨慎</footer>

    <Transition name="toast">
      <div v-if="toastVisible" class="toast" role="status" aria-live="polite">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <circle cx="12" cy="12" r="9" />
          <path d="M8.5 12.4l2.3 2.3 4.7-4.9" />
        </svg>
        <span>{{ toastMessage }}</span>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.app {
  position: relative;
  isolation: isolate;
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  min-height: 100dvh;
  padding: max(12px, env(safe-area-inset-top)) clamp(14px, 4vw, 40px)
    max(12px, env(safe-area-inset-bottom));
  overflow: hidden;
}

.backdrop {
  position: fixed;
  inset: 0;
  z-index: -1;
  overflow: hidden;
  pointer-events: none;
}

.backdrop__glow {
  position: absolute;
  border-radius: 50%;
  filter: blur(90px);
}

.backdrop__glow--indigo {
  top: -180px;
  right: -140px;
  width: 560px;
  height: 560px;
  background: rgba(79, 70, 229, 0.34);
}

.backdrop__glow--teal {
  bottom: -220px;
  left: -160px;
  width: 520px;
  height: 520px;
  background: rgba(20, 184, 166, 0.22);
}

.backdrop__grid {
  position: absolute;
  inset: 0;
  background-image: linear-gradient(rgba(148, 163, 184, 0.07) 1px, transparent 1px),
    linear-gradient(90deg, rgba(148, 163, 184, 0.07) 1px, transparent 1px);
  background-size: 54px 54px;
  opacity: 0.55;
  -webkit-mask-image: radial-gradient(ellipse 90% 65% at 50% 40%, #000 20%, transparent 78%);
  mask-image: radial-gradient(ellipse 90% 65% at 50% 40%, #000 20%, transparent 78%);
}

.shell {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  align-items: center;
  width: 100%;
  max-width: 480px;
  margin: auto;
  padding: 4px 0 8px;
}

.intro {
  display: none;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand__mark {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  border-radius: 12px;
  color: #fff;
  background: linear-gradient(135deg, #6366f1, #22d3ee);
  box-shadow: 0 14px 34px -16px rgba(99, 102, 241, 0.9);
}

.brand__mark svg {
  width: 22px;
  height: 22px;
}

.brand__name {
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: #f8fafc;
}

.intro__title {
  margin: 30px 0 0;
  font-size: clamp(36px, 4.4vw, 54px);
  line-height: 1.14;
  font-weight: 700;
  letter-spacing: -0.03em;
  color: #f8fafc;
}

.intro__title em {
  font-style: normal;
  background: linear-gradient(100deg, #a5b4fc, #67e8f9 55%, #6ee7b7);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.intro__desc {
  margin: 20px 0 0;
  max-width: 470px;
  font-size: 15px;
  line-height: 1.78;
  color: rgba(203, 213, 225, 0.82);
}

.intro__list {
  display: flex;
  flex-direction: column;
  gap: 14px;
  margin: 30px 0 0;
  padding: 0;
  list-style: none;
}

.intro__list li {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  color: rgba(226, 232, 240, 0.9);
}

.dot {
  flex: none;
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.dot--teal {
  background: #2dd4bf;
  box-shadow: 0 0 0 4px rgba(45, 212, 191, 0.16);
}

.dot--indigo {
  background: #818cf8;
  box-shadow: 0 0 0 4px rgba(129, 140, 248, 0.16);
}

.dot--amber {
  background: #fbbf24;
  box-shadow: 0 0 0 4px rgba(251, 191, 36, 0.16);
}

.intro__foot {
  margin: 32px 0 0;
  max-width: 420px;
  font-size: 12.5px;
  line-height: 1.7;
  color: rgba(148, 163, 184, 0.75);
}

.panel {
  position: relative;
  width: 100%;
  padding: clamp(16px, 4vw, 22px);
  border: 1px solid rgba(255, 255, 255, 0.65);
  border-radius: clamp(20px, 4.6vw, 28px);
  background: linear-gradient(180deg, #ffffff 0%, #fbfcff 100%);
  color: var(--ink);
  box-shadow:
    0 34px 80px -38px rgba(2, 6, 23, 0.85),
    0 18px 44px -26px rgba(2, 6, 23, 0.6);
  overflow: hidden;
}

.panel::before {
  content: '';
  position: absolute;
  inset: 0 0 auto;
  height: 3px;
  background: linear-gradient(90deg, #6366f1, #22d3ee 55%, #34d399);
}

.panel--warning::before {
  background: linear-gradient(90deg, #f59e0b, #fb923c);
}

.panel--critical::before {
  background: linear-gradient(90deg, #ef4444, #f59e0b);
}

.panel--expired::before {
  background: linear-gradient(90deg, #94a3b8, #64748b);
}

.panel__head {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 28px;
}

.panel__brand {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 700;
  letter-spacing: -0.01em;
  color: var(--ink);
}

.panel__brand svg {
  width: 18px;
  height: 18px;
  color: var(--primary);
}

.status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-left: auto;
  padding: 5px 10px;
  border-radius: 999px;
  background: var(--primary-soft);
  color: var(--primary);
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.status--warning {
  background: #fff7ed;
  color: #b45309;
}

.status--critical {
  background: #fef2f2;
  color: #b91c1c;
}

.status--expired {
  background: #f1f5f9;
  color: #475569;
}

.status__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.status--normal .status__dot {
  animation: breathe 2.6s ease-in-out infinite;
}

.icon-btn {
  display: grid;
  place-items: center;
  flex: none;
  width: 34px;
  height: 34px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: #fff;
  color: var(--muted);
  cursor: pointer;
  transition:
    color 0.15s ease,
    border-color 0.15s ease,
    background 0.15s ease;
}

.icon-btn:hover:not(:disabled) {
  color: var(--ink);
  border-color: #cbd5e1;
  background: var(--surface-soft);
}

.icon-btn:disabled {
  opacity: 0.6;
  cursor: default;
}

.icon {
  width: 16px;
  height: 16px;
}

.icon--spin {
  animation: spin 0.9s linear infinite;
}

.state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 40px 8px;
  text-align: center;
}

.state h2 {
  margin: 2px 0 0;
  font-size: 17px;
  font-weight: 700;
  color: var(--ink);
}

.state p {
  margin: 0;
  max-width: 300px;
  font-size: 13px;
  line-height: 1.7;
  color: var(--muted);
}

.state__icon {
  display: grid;
  place-items: center;
  width: 44px;
  height: 44px;
  border-radius: 14px;
  background: #fef2f2;
  color: var(--danger);
  font-size: 20px;
  font-weight: 700;
}

.state--loading {
  padding: 56px 8px;
}

.state--expired .state__icon {
  background: #fff7ed;
  color: var(--amber);
}

.spinner {
  display: inline-block;
  width: 22px;
  height: 22px;
  border: 2.5px solid rgba(79, 70, 229, 0.18);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.85s linear infinite;
}

.spinner--sm {
  width: 15px;
  height: 15px;
  border-width: 2px;
  border-color: rgba(255, 255, 255, 0.35);
  border-top-color: #fff;
}

.banner {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin: 16px 0 2px;
  padding: 12px 14px;
  border: 1px solid #fed7aa;
  border-radius: 14px;
  background: #fff7ed;
}

.banner__icon {
  display: grid;
  place-items: center;
  flex: none;
  width: 20px;
  height: 20px;
  margin-top: 1px;
  border-radius: 50%;
  background: #f97316;
  color: #fff;
  font-size: 12px;
  font-weight: 700;
}

.banner__body {
  flex: 1;
  min-width: 0;
}

.banner__body strong {
  display: block;
  font-size: 12.5px;
  color: #9a3412;
}

.banner__body p {
  margin: 2px 0 0;
  font-size: 12px;
  line-height: 1.6;
  color: #c2410c;
}

.banner__action {
  flex: none;
  align-self: center;
  padding: 5px 10px;
  border: 0;
  border-radius: 8px;
  background: #ffedd5;
  color: #9a3412;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.countdown {
  padding: 14px 0 4px;
  text-align: center;
}

.countdown__label {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin: 0;
  font-size: 13px;
  font-weight: 500;
  color: var(--muted);
}

.countdown__days {
  padding: 2px 9px;
  border-radius: 999px;
  background: var(--primary-soft);
  color: var(--primary);
  font-size: 12px;
  font-weight: 600;
}

.clock {
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: clamp(2px, 1.2vw, 8px);
  margin-top: 8px;
  font-family: var(--font-mono);
}

.clock__cell {
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  min-width: clamp(58px, 19vw, 78px);
  padding: 7px 0 6px;
  border: 1px solid var(--line);
  border-radius: 16px;
  background: var(--surface-soft);
}

.clock__value {
  font-size: clamp(26px, 8.6vw, 36px);
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.02em;
  color: var(--ink);
  font-variant-numeric: tabular-nums;
}

.clock__unit {
  font-family: var(--font-sans);
  font-size: 9.5px;
  font-weight: 500;
  letter-spacing: 0.18em;
  color: var(--muted-soft);
}

.clock__colon {
  font-size: clamp(24px, 8vw, 34px);
  font-weight: 700;
  line-height: 1;
  color: #94a3b8;
}

.countdown--warning .clock__value {
  color: var(--amber);
}

.countdown--warning .clock__cell {
  border-color: #fde68a;
  background: #fffbeb;
}

.countdown--critical .clock__value {
  color: var(--danger);
  animation: breathe 1.8s ease-in-out infinite;
}

.countdown--critical .clock__cell {
  border-color: #fecaca;
  background: #fef2f2;
}

.countdown--expired .clock__value {
  color: var(--muted-soft);
}

.countdown__hint {
  margin: 8px 0 0;
  font-size: 12.5px;
  font-weight: 600;
  color: var(--amber);
}

.countdown--critical .countdown__hint {
  color: var(--danger);
}

.countdown__comment {
  margin: 6px 0 0;
  font-size: 12.5px;
  line-height: 1.6;
  color: var(--ink-soft);
}

.countdown__kept {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--teal);
}

.keep-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--line);
}

.form__head h2 {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--ink);
}

.form__head p {
  margin: 3px 0 0;
  font-size: 12.5px;
  color: var(--muted);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-height: 24px;
}

.field__label {
  font-size: 13px;
  font-weight: 600;
  color: var(--ink-soft);
}

.field__hint {
  font-size: 11.5px;
  color: var(--muted-soft);
}

.field__action {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 8px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--primary);
  font-size: 12.5px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease;
}

.field__action:hover:not(:disabled) {
  background: var(--primary-soft);
}

.field__action:disabled {
  opacity: 0.6;
  cursor: default;
}

.field__action .icon {
  width: 13px;
  height: 13px;
}

.captcha {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 9px 0;
  border: 1px dashed #c7d2fe;
  border-radius: 14px;
  background: linear-gradient(180deg, #eef2ff, #f8faff);
  font-family: var(--font-mono);
  font-size: 23px;
  font-weight: 700;
  letter-spacing: 0.42em;
  text-indent: 0.42em;
  color: #4338ca;
  user-select: none;
}

.input {
  width: 100%;
  height: 44px;
  padding: 0 14px;
  border: 1px solid #dbe3ee;
  border-radius: 12px;
  background: var(--surface-soft);
  color: var(--ink);
  font-size: 16px;
  line-height: 1;
  appearance: none;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease,
    background 0.15s ease;
}

.input::placeholder {
  color: var(--muted-soft);
  font-size: 14px;
}

.input:focus {
  outline: none;
  border-color: #818cf8;
  background: #fff;
  box-shadow: 0 0 0 4px rgba(99, 102, 241, 0.13);
}

.input--error {
  border-color: #fca5a5;
  background: #fff5f5;
}

.input--otp {
  font-family: var(--font-mono);
  font-size: 21px;
  font-weight: 600;
  letter-spacing: 0.36em;
  text-align: center;
  padding-left: 0.36em;
}

.field__error {
  margin: 0;
  font-size: 12px;
  line-height: 1.6;
  color: var(--danger);
}

.form__error {
  margin: 0;
  padding: 10px 12px;
  border: 1px solid #fecaca;
  border-radius: 10px;
  background: #fef2f2;
  font-size: 12.5px;
  line-height: 1.6;
  color: #b91c1c;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 0;
  border-radius: 12px;
  font-weight: 600;
  cursor: pointer;
  transition:
    transform 0.12s ease,
    box-shadow 0.15s ease,
    background 0.15s ease,
    opacity 0.15s ease;
}

.btn:active:not(:disabled) {
  transform: translateY(1px);
}

.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
  box-shadow: none;
}

.btn--primary {
  height: 46px;
  padding: 0 22px;
  background: linear-gradient(135deg, #4f46e5 0%, #6366f1 52%, #0ea5e9 100%);
  color: #fff;
  font-size: 16px;
  box-shadow: 0 14px 26px -14px rgba(79, 70, 229, 0.85);
}

.btn--primary:hover:not(:disabled) {
  box-shadow: 0 16px 30px -12px rgba(79, 70, 229, 0.9);
}

.btn--primary:disabled {
  opacity: 1;
  background: #eef1f6;
  color: #94a3b8;
  box-shadow: none;
}

.btn--block {
  width: 100%;
}

.form__tip {
  margin: 0;
  font-size: 11.5px;
  line-height: 1.7;
  text-align: center;
  color: var(--muted-soft);
}

.panel__foot {
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px dashed var(--line);
  text-align: center;
  font-size: 11.5px;
  color: var(--muted-soft);
}

.page-foot {
  position: relative;
  z-index: 1;
  padding: 10px 0 2px;
  text-align: center;
  font-size: 12px;
  color: rgba(148, 163, 184, 0.7);
}

.toast {
  position: fixed;
  top: max(20px, env(safe-area-inset-top));
  left: 50%;
  z-index: 100;
  display: flex;
  align-items: center;
  gap: 8px;
  width: max-content;
  max-width: calc(100vw - 32px);
  padding: 11px 16px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 999px;
  background: rgba(15, 23, 42, 0.96);
  color: #f8fafc;
  font-size: 13px;
  font-weight: 500;
  transform: translateX(-50%);
  box-shadow: 0 18px 40px -18px rgba(2, 6, 23, 0.9);
}

.toast svg {
  flex: none;
  width: 16px;
  height: 16px;
  color: #34d399;
}

.toast-enter-active,
.toast-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-8px);
}

.btn:focus-visible,
.icon-btn:focus-visible,
.field__action:focus-visible,
.banner__action:focus-visible {
  outline: 2px solid #6366f1;
  outline-offset: 2px;
}

.input:focus-visible {
  outline: none;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes breathe {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.55;
  }
}

@media (min-width: 960px) {
  .app {
    padding: clamp(18px, 3vw, 32px);
  }

  .shell {
    grid-template-columns: minmax(0, 1fr) minmax(380px, 440px);
    gap: clamp(48px, 7vw, 96px);
    max-width: 1080px;
    padding: 4px 0;
  }

  .intro {
    display: block;
  }

  .panel {
    padding: 22px;
  }

  .panel__brand {
    display: none;
  }

  .status {
    margin-right: auto;
    margin-left: 0;
  }

  .countdown {
    padding-top: 12px;
  }
}

@media (max-width: 360px) {
  .clock__cell {
    min-width: 54px;
  }

  .captcha {
    font-size: 20px;
    letter-spacing: 0.32em;
    text-indent: 0.32em;
  }
}


@media (max-height: 720px) {
  .app {
    padding-top: max(8px, env(safe-area-inset-top));
    padding-bottom: max(8px, env(safe-area-inset-bottom));
  }

  .shell {
    padding: 0 0 4px;
  }

  .page-foot,
  .form__tip {
    display: none;
  }

  .countdown {
    padding-top: 10px;
  }

  .keep-form {
    gap: 10px;
    margin-top: 10px;
    padding-top: 10px;
  }

  .panel__foot {
    margin-top: 10px;
    padding-top: 8px;
  }
}

@media (max-height: 620px) {
  .panel {
    padding: 14px;
  }

  .panel__head {
    min-height: 24px;
  }

  .countdown {
    padding-top: 6px;
  }

  .countdown__comment {
    margin-top: 4px;
  }

  .clock__unit {
    display: none;
  }

  .clock__cell {
    padding: 5px 0;
  }

  .field__head {
    min-height: 22px;
  }

  .input {
    height: 42px;
  }

  .btn--primary {
    height: 44px;
  }

  .keep-form {
    gap: 8px;
    margin-top: 8px;
    padding-top: 8px;
  }

  .panel__foot {
    display: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .status--normal .status__dot,
  .countdown--critical .clock__value,
  .spinner,
  .icon--spin {
    animation: none;
  }
}
</style>
