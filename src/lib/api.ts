const API_BASE = '/api';

export interface User {
  id: string;
  email: string;
  name: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface Activity {
  id: string;
  user_id: string;
  activity_type: string;
  name: string;
  object_store_path: string;
  submitted_at: string;
}

export interface TrackPoint {
  lat: number;
  lon: number;
  ele: number | null;
  time: string | null;
}

export interface TrackBounds {
  min_lat: number;
  max_lat: number;
  min_lon: number;
  max_lon: number;
}

export interface TrackData {
  points: TrackPoint[];
  bounds: TrackBounds;
}

class ApiClient {
  private token: string | null = null;

  setToken(token: string | null) {
    this.token = token;
    if (token) {
      localStorage.setItem('token', token);
    } else {
      localStorage.removeItem('token');
    }
  }

  getToken(): string | null {
    if (this.token) return this.token;
    if (typeof window !== 'undefined') {
      this.token = localStorage.getItem('token');
    }
    return this.token;
  }

  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
    const token = this.getToken();
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...(options.headers || {}),
    };

    if (token) {
      (headers as Record<string, string>)['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Request failed' }));
      throw new Error(error.error || `Request failed with status ${response.status}`);
    }

    return response.json();
  }

  // Auth endpoints
  async register(email: string, password: string, name: string): Promise<AuthResponse> {
    const result = await this.request<AuthResponse>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password, name }),
    });
    this.setToken(result.token);
    return result;
  }

  async login(email: string, password: string): Promise<AuthResponse> {
    const result = await this.request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
    this.setToken(result.token);
    return result;
  }

  async me(): Promise<User> {
    return this.request<User>('/auth/me');
  }

  logout() {
    this.setToken(null);
  }

  // Activity endpoints
  async getUserActivities(userId: string): Promise<Activity[]> {
    return this.request<Activity[]>(`/users/${userId}/activities`);
  }

  async getActivity(id: string): Promise<Activity> {
    return this.request<Activity>(`/activities/${id}`);
  }

  async getActivityTrack(id: string): Promise<TrackData> {
    return this.request<TrackData>(`/activities/${id}/track`);
  }

  async uploadActivity(
    userId: string,
    file: File,
    name: string,
    activityType: string
  ): Promise<Activity> {
    const token = this.getToken();
    const formData = new FormData();
    formData.append('file', file);

    const response = await fetch(
      `${API_BASE}/activities/new?user_id=${encodeURIComponent(userId)}&activity_type=${encodeURIComponent(activityType)}&name=${encodeURIComponent(name)}`,
      {
        method: 'POST',
        headers: token ? { Authorization: `Bearer ${token}` } : {},
        body: formData,
      }
    );

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Upload failed' }));
      throw new Error(error.error || 'Upload failed');
    }

    return response.json();
  }
}

export const api = new ApiClient();
