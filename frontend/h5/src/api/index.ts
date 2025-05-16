import axios from 'axios'

const instance = axios.create({
  baseURL: process.env.API_BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// 请求拦截器
instance.interceptors.request.use(
  config => {
    // 在这里可以添加请求前的处理逻辑
    return config
  },
  error => {
    return Promise.reject(error)
  }
)

// 响应拦截器
instance.interceptors.response.use(
  response => {
    // 在这里可以添加响应后的处理逻辑
    return response.data
  },
  error => {
    return Promise.reject(error)
  }
)

export default instance