import axios from "axios";

const API_URL = process.env.REACT_APP_API_URL || "http://localhost:30050/v1";

const apiClient = axios.create({
  baseURL: API_URL,
  headers: {
    "Content-Type": "application/json",
  },
});

// 请求拦截器：在请求发送前添加 Authorization 头
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem("access_token");
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  },
);

// 响应拦截器：处理 401 错误（Token 过期）
apiClient.interceptors.response.use(
  (response) => {
    return response;
  },
  (error) => {
    if (error.response && error.response.status === 401) {
      // 处理 Token 过期，例如重定向到登录页面
      window.location.href = "/login";
    }
    return Promise.reject(error);
  },
);

export default apiClient;
