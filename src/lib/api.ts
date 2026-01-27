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
  visibility: 'public' | 'private';
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

export interface Segment {
  id: string;
  creator_id: string;
  name: string;
  description: string | null;
  activity_type: string;
  distance_meters: number;
  elevation_gain_meters: number | null;
  elevation_loss_meters: number | null;
  visibility: 'public' | 'private';
  created_at: string;
}

export interface SegmentEffort {
  id: string;
  segment_id: string;
  activity_id: string;
  user_id: string;
  started_at: string;
  elapsed_time_seconds: number;
  moving_time_seconds: number | null;
  average_speed_mps: number | null;
  max_speed_mps: number | null;
  is_personal_record: boolean;
  created_at: string;
}

export interface CreateSegmentRequest {
  name: string;
  description?: string;
  activity_type: string;
  points: { lat: number; lon: number; ele?: number }[];
  visibility?: 'public' | 'private';
}

export interface SegmentTrackPoint {
  lat: number;
  lon: number;
  ele: number | null;
}

export interface SegmentTrackData {
  points: SegmentTrackPoint[];
  bounds: {
    min_lat: number;
    max_lat: number;
    min_lon: number;
    max_lon: number;
  };
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

  async updateActivity(
    id: string,
    data: { name?: string; activity_type?: string; visibility?: 'public' | 'private' }
  ): Promise<Activity> {
    return this.request<Activity>(`/activities/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteActivity(id: string): Promise<void> {
    await this.request<void>(`/activities/${id}`, {
      method: 'DELETE',
    });
  }

  async uploadActivity(
    userId: string,
    file: File,
    name: string,
    activityType: string,
    visibility: 'public' | 'private' = 'public'
  ): Promise<Activity> {
    const token = this.getToken();
    const formData = new FormData();
    formData.append('file', file);

    const params = new URLSearchParams({
      user_id: userId,
      activity_type: activityType,
      name: name,
      visibility: visibility,
    });

    const response = await fetch(
      `${API_BASE}/activities/new?${params.toString()}`,
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

  // Segment endpoints
  async listSegments(activityType?: string): Promise<Segment[]> {
    const params = activityType ? `?activity_type=${activityType}` : '';
    return this.request<Segment[]>(`/segments${params}`);
  }

  async getSegment(id: string): Promise<Segment> {
    return this.request<Segment>(`/segments/${id}`);
  }

  async getSegmentLeaderboard(id: string): Promise<SegmentEffort[]> {
    return this.request<SegmentEffort[]>(`/segments/${id}/leaderboard`);
  }

  async getSegmentTrack(id: string): Promise<SegmentTrackData> {
    return this.request<SegmentTrackData>(`/segments/${id}/track`);
  }

  async createSegment(data: CreateSegmentRequest): Promise<Segment> {
    return this.request<Segment>('/segments', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }
}

export const api = new ApiClient();
