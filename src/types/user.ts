export interface User {
  id: number;
  username: string;
  email: string;
  phone?: string;
  avatarUrl?: string;
  nickname?: string;
  createdAt: string;
  updatedAt: string;
  lastLoginAt?: string;
  status: number;
}

export interface RegisterForm {
  username: string;
  password: string;
  email: string;
  phone?: string;
  nickname?: string;
}

export interface LoginForm {
  username: string;
  password: string;
}

export interface ApiResponse<T = any> {
  code: number;
  message: string;
  data?: T;
} 