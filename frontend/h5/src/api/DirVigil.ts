import request from './index'

export function getStatus(): Promise<string> {
  return request.get('/status')
}

export function keepAlive(tow_fa_code: string, captcha: string): Promise<string> {
  return request.get('/keepalive', {
    params: {
        tow_fa_code: tow_fa_code,
        captcha: captcha
    }
  })
}

export function getCaptcha(): Promise<string> {
  return request.get('/captcha')
}