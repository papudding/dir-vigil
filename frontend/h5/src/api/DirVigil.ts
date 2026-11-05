import request from './index'

export interface StatusResponse {
  /** 距离自动清理的剩余秒数 */
  remainsSeconds: number
  /** 服务端配置的备注，默认返回 ^-^ */
  comment: string
}

export function getStatus(): Promise<StatusResponse> {
  return request.get('/status') as unknown as Promise<StatusResponse>
}

/**
 * 刷新倒计时。使用 POST + JSON body，避免把 captcha / TOTP 放进 URL 查询串
 * （查询串会进入 nginx/browser 历史等日志）。
 */
export function keepAlive(twoFaCode: string, captcha: string): Promise<string> {
  return request.post('/keepalive', {
    captcha,
    tow_fa_code: twoFaCode,
  }) as unknown as Promise<string>
}

export function getCaptcha(): Promise<string> {
  return request.get('/captcha') as unknown as Promise<string>
}
