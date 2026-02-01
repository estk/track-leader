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

export type ActivityVisibility = 'public' | 'private' | 'teams_only';

// Activity Types (UUID-based)
export interface ActivityType {
  id: string;
  name: string;
  is_builtin: boolean;
  created_by: string | null;
}

export type ResolvedActivityTypeStatus = 'exact' | 'ambiguous' | 'not_found';

export interface ResolveActivityTypeResponse {
  status: ResolvedActivityTypeStatus;
  type_id?: string;         // Present when status is 'exact'
  type_ids?: string[];      // Present when status is 'ambiguous'
}

// Built-in activity type IDs (fixed UUIDs)
export const ACTIVITY_TYPE_IDS = {
  WALK: '00000000-0000-0000-0000-000000000001',
  RUN: '00000000-0000-0000-0000-000000000002',
  HIKE: '00000000-0000-0000-0000-000000000003',
  ROAD: '00000000-0000-0000-0000-000000000004',
  MTB: '00000000-0000-0000-0000-000000000005',
  EMTB: '00000000-0000-0000-0000-000000000006',
  GRAVEL: '00000000-0000-0000-0000-000000000007',
  UNKNOWN: '00000000-0000-0000-0000-000000000008',
  DIG: '00000000-0000-0000-0000-000000000009',
} as const;

// Display names for built-in activity types
export const ACTIVITY_TYPE_NAMES: Record<string, string> = {
  [ACTIVITY_TYPE_IDS.WALK]: 'Walk',
  [ACTIVITY_TYPE_IDS.RUN]: 'Run',
  [ACTIVITY_TYPE_IDS.HIKE]: 'Hike',
  [ACTIVITY_TYPE_IDS.ROAD]: 'Road Cycling',
  [ACTIVITY_TYPE_IDS.MTB]: 'Mountain Biking',
  [ACTIVITY_TYPE_IDS.EMTB]: 'E-Mountain Biking',
  [ACTIVITY_TYPE_IDS.GRAVEL]: 'Gravel',
  [ACTIVITY_TYPE_IDS.UNKNOWN]: 'Unknown',
  [ACTIVITY_TYPE_IDS.DIG]: 'Trail Work',
};

// Activity type options for dropdowns
export const ACTIVITY_TYPE_OPTIONS = [
  { id: ACTIVITY_TYPE_IDS.RUN, name: 'Run' },
  { id: ACTIVITY_TYPE_IDS.ROAD, name: 'Road Cycling' },
  { id: ACTIVITY_TYPE_IDS.MTB, name: 'Mountain Biking' },
  { id: ACTIVITY_TYPE_IDS.HIKE, name: 'Hike' },
  { id: ACTIVITY_TYPE_IDS.WALK, name: 'Walk' },
  { id: ACTIVITY_TYPE_IDS.EMTB, name: 'E-Mountain Biking' },
  { id: ACTIVITY_TYPE_IDS.GRAVEL, name: 'Gravel' },
  { id: ACTIVITY_TYPE_IDS.DIG, name: 'Trail Work' },
  { id: ACTIVITY_TYPE_IDS.UNKNOWN, name: 'Other' },
];

// Get display name for an activity type ID
export function getActivityTypeName(id: string): string {
  return ACTIVITY_TYPE_NAMES[id] || 'Unknown';
}

export interface Activity {
  id: string;
  user_id: string;
  activity_type_id: string;
  name: string;
  object_store_path: string;
  started_at: string;       // When the activity actually occurred (from GPX track data)
  submitted_at: string;     // When the activity was uploaded to the system
  visibility: ActivityVisibility;
  // Multi-sport support
  type_boundaries: (string | number[])[] | null;  // ISO8601 timestamps or Rust OffsetDateTime arrays
  segment_types: string[] | null;     // Activity type UUIDs
}

// Activity filter types
export type DateRangeFilter = 'all' | 'week' | 'month' | 'year';
export type VisibilityFilter = 'all' | 'public' | 'private' | 'teams_only';
export type ActivitySortBy = 'recent' | 'oldest' | 'distance' | 'duration';

export interface UserActivitiesFilters {
  activityTypeId?: string;
  dateRange?: DateRangeFilter;
  startDate?: string;  // YYYY-MM-DD
  endDate?: string;
  visibility?: VisibilityFilter;
  sortBy?: ActivitySortBy;
  search?: string;
  limit?: number;
  offset?: number;
}

export interface TrackPoint {
  lat: number;
  lon: number;
  ele: number | null;
  time: string | null;
}

// Stopped/Dig segment types
export interface StoppedSegment {
  id: string;
  activity_id: string;
  start_time: string;
  end_time: string;
  duration_seconds: number;
}

export interface DigSegment {
  id: string;
  activity_id: string;
  start_time: string;
  end_time: string;
  duration_seconds: number;
  created_at: string;
}

export interface CreateDigSegmentsRequest {
  stopped_segment_ids: string[];
}

export interface DigTimeSummary {
  total_dig_time_seconds: number;
  dig_segment_count: number;
  activity_duration_seconds: number | null;
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

// Sensor data types
export interface SensorData {
  activity_id: string;
  has_heart_rate: boolean;
  has_cadence: boolean;
  has_power: boolean;
  has_temperature: boolean;
  distances: number[];
  heart_rates?: (number | null)[];
  cadences?: (number | null)[];
  powers?: (number | null)[];
  temperatures?: (number | null)[];
}

export type SegmentVisibility = 'public' | 'private' | 'teams_only';

export interface Segment {
  id: string;
  creator_id: string;
  creator_name: string | null;
  name: string;
  description: string | null;
  activity_type_id: string;
  distance_meters: number;
  elevation_gain_meters: number | null;
  elevation_loss_meters: number | null;
  average_grade: number | null;
  max_grade: number | null;
  climb_category: number | null;
  visibility: SegmentVisibility;
  created_at: string;
}

export interface SegmentEffort {
  id: string;
  segment_id: string;
  activity_id: string;
  user_id: string;
  user_name: string | null;
  started_at: string;
  elapsed_time_seconds: number;
  moving_time_seconds: number | null;
  average_speed_mps: number | null;
  max_speed_mps: number | null;
  is_personal_record: boolean;
  created_at: string;
  start_fraction: number | null;
  end_fraction: number | null;
}

export interface CreateSegmentRequest {
  name: string;
  description?: string;
  /** Activity type UUID. Optional if source_activity_id is provided (inherits from the activity). */
  activity_type_id?: string;
  points: { lat: number; lon: number; ele?: number }[];
  visibility?: SegmentVisibility;
  /** If provided, the segment inherits its activity_type_id from this activity. */
  source_activity_id?: string;
  /** Team IDs to share segment with when visibility is 'teams_only'. */
  team_ids?: string[];
}

export type SegmentSortBy = 'created_at' | 'name' | 'distance' | 'elevation_gain';
export type SortOrder = 'asc' | 'desc';
export type ClimbCategoryFilter = 'hc' | 'cat1' | 'cat2' | 'cat3' | 'cat4' | 'flat';

export interface ListSegmentsOptions {
  activityTypeId?: string;
  search?: string;
  sortBy?: SegmentSortBy;
  sortOrder?: SortOrder;
  minDistanceMeters?: number;
  maxDistanceMeters?: number;
  climbCategory?: ClimbCategoryFilter;
  limit?: number;
}

export interface SegmentValidation {
  is_valid: boolean;
  errors: string[];
}

export interface PreviewSegmentResponse {
  distance_meters: number;
  elevation_gain_meters: number | null;
  elevation_loss_meters: number | null;
  average_grade: number | null;
  max_grade: number | null;
  climb_category: number | null;
  point_count: number;
  validation: SegmentValidation;
}

export interface ActivitySegmentEffort {
  effort_id: string;
  segment_id: string;
  elapsed_time_seconds: number;
  is_personal_record: boolean;
  started_at: string;
  segment_name: string;
  segment_distance: number;
  activity_type_id: string;
  rank: number;
  start_fraction: number | null;
  end_fraction: number | null;
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

export interface StarredSegmentEffort {
  segment_id: string;
  segment_name: string;
  activity_type_id: string;
  distance_meters: number;
  elevation_gain_meters: number | null;
  best_time_seconds: number | null;
  best_effort_rank: number | null;
  best_effort_date: string | null;
  user_effort_count: number;
  leader_time_seconds: number | null;
}

// Leaderboard types
export type LeaderboardScope = 'all_time' | 'year' | 'month' | 'week';
export type GenderFilter = 'all' | 'male' | 'female';
export type AgeGroup = 'all' | '18-24' | '25-29' | '30-34' | '35-39' | '40-49' | '50-59' | '60+';
export type WeightClass = 'all' | 'featherweight' | 'lightweight' | 'welterweight' | 'middleweight' | 'cruiserweight' | 'heavyweight';

export interface LeaderboardFilters {
  scope: LeaderboardScope;
  gender: GenderFilter;
  age_group: AgeGroup;
  weight_class: WeightClass;
  country: string | null;
  limit: number;
  offset: number;
}

export interface CountryStats {
  country: string;
  user_count: number;
}

export interface LeaderboardEntry {
  effort_id: string;
  elapsed_time_seconds: number;
  moving_time_seconds: number | null;
  average_speed_mps: number | null;
  started_at: string;
  is_personal_record: boolean;
  user_id: string;
  user_name: string;
  rank: number;
  gap_seconds: number | null;
}

export interface LeaderboardResponse {
  entries: LeaderboardEntry[];
  total_count: number;
  filters: LeaderboardFilters;
}

export interface LeaderboardPosition {
  user_rank: number | null;
  user_entry: LeaderboardEntry | null;
  entries_above: LeaderboardEntry[];
  entries_below: LeaderboardEntry[];
  total_count: number;
}

// Achievement types
export type AchievementType = 'kom' | 'qom' | 'course_record';

export interface Achievement {
  id: string;
  user_id: string;
  segment_id: string;
  effort_id: string | null;
  achievement_type: AchievementType;
  earned_at: string;
  lost_at: string | null;
  effort_count: number | null;
}

export interface AchievementWithSegment extends Achievement {
  segment_name: string;
  segment_distance_meters: number;
  segment_activity_type_id: string;
}

export interface AchievementHolder {
  user_id: string;
  user_name: string;
  achievement_type: AchievementType;
  earned_at: string;
  elapsed_time_seconds: number | null;
  effort_count: number | null;
}

export interface SegmentAchievements {
  segment_id: string;
  kom: AchievementHolder | null;
  qom: AchievementHolder | null;
}

// User demographics types
export interface UserWithDemographics extends User {
  gender: string | null;
  birth_year: number | null;
  weight_kg: number | null;
  country: string | null;
  region: string | null;
}

export interface UpdateDemographicsRequest {
  gender?: string | null;
  birth_year?: number | null;
  weight_kg?: number | null;
  country?: string | null;
  region?: string | null;
}

// Global leaderboard types
export interface GlobalLeaderboardFilters {
  scope?: LeaderboardScope;
  gender?: GenderFilter;
  ageGroup?: AgeGroup;
  weightClass?: WeightClass;
  country?: string;
  activityTypeId?: string;  // For crown leaderboard only
  limit?: number;
  offset?: number;
}

export interface CrownCountEntry {
  user_id: string;
  user_name: string;
  kom_count: number;
  qom_count: number;
  total_crowns: number;
  rank: number;
}

export interface DistanceLeaderEntry {
  user_id: string;
  user_name: string;
  total_distance_meters: number;
  activity_count: number;
  rank: number;
}

// Leaderboard type enum
export type LeaderboardType = 'crowns' | 'distance' | 'dig_time' | 'dig_percentage' | 'average_speed';

export interface DigTimeLeaderEntry {
  user_id: string;
  user_name: string;
  total_dig_time_seconds: number;
  dig_segment_count: number;
  rank: number;
}

export interface DigPercentageLeaderEntry {
  user_id: string;
  user_name: string;
  dig_percentage: number;
  total_dig_time_seconds: number;
  total_activity_duration_seconds: number;
  rank: number;
}

export interface AverageSpeedLeaderEntry {
  user_id: string;
  user_name: string;
  average_speed_mps: number;
  activity_count: number;
  rank: number;
}

// Social types (follows, notifications)
export interface UserProfile {
  id: string;
  email: string;
  name: string;
  created_at: string;
  follower_count: number;
  following_count: number;
  gender: string | null;
  birth_year: number | null;
  weight_kg: number | null;
  country: string | null;
  region: string | null;
}

export interface UserSummary {
  id: string;
  name: string;
  follower_count: number;
  following_count: number;
  followed_at: string;
}

export interface FollowListResponse {
  users: UserSummary[];
  total_count: number;
}

export interface FollowStatusResponse {
  is_following: boolean;
}

export type NotificationType = 'follow' | 'kudos' | 'comment' | 'crown_achieved' | 'crown_lost' | 'pr';

export interface Notification {
  id: string;
  user_id: string;
  notification_type: NotificationType;
  actor_id: string | null;
  actor_name: string | null;
  target_type: string | null;
  target_id: string | null;
  message: string | null;
  read_at: string | null;
  created_at: string;
}

export interface NotificationsResponse {
  notifications: Notification[];
  unread_count: number;
  total_count: number;
}

// Activity Feed types
export interface FeedActivity {
  id: string;
  user_id: string;
  name: string;
  activity_type_id: string;
  submitted_at: string;
  visibility: string;
  user_name: string;
  distance: number | null;
  duration: number | null;
  elevation_gain: number | null;
  kudos_count: number;
  comment_count: number;
  /** Team names this activity is shared with (for teams_only visibility) */
  team_names?: string[];
}

// Feed filter options
export interface FeedFilters {
  activityTypeId?: string;
  dateRange?: DateRangeFilter;
  startDate?: string;  // YYYY-MM-DD for custom range
  endDate?: string;
  limit?: number;
  offset?: number;
}

// Date range options for UI
export const DATE_RANGE_OPTIONS: { value: DateRangeFilter; label: string }[] = [
  { value: 'all', label: 'All Time' },
  { value: 'week', label: 'This Week' },
  { value: 'month', label: 'This Month' },
  { value: 'year', label: 'This Year' },
];

// Kudos types
export interface KudosGiver {
  user_id: string;
  user_name: string;
  created_at: string;
}

export interface KudosStatusResponse {
  has_given: boolean;
}

// Comment types
export interface Comment {
  id: string;
  user_id: string;
  activity_id: string;
  parent_id: string | null;
  content: string;
  created_at: string;
  updated_at: string | null;
  user_name: string;
}

// Stats types
export interface Stats {
  active_users: number;
  segments_created: number;
  activities_uploaded: number;
}

// Team types
export type TeamRole = 'owner' | 'admin' | 'member';
export type TeamVisibility = 'public' | 'private';
export type TeamJoinPolicy = 'open' | 'request' | 'invitation';

export interface Team {
  id: string;
  name: string;
  description: string | null;
  avatar_url: string | null;
  visibility: TeamVisibility;
  join_policy: TeamJoinPolicy;
  owner_id: string;
  member_count: number;
  activity_count: number;
  segment_count: number;
  featured_leaderboard: LeaderboardType | null;
  created_at: string;
  updated_at: string | null;
}

export interface TeamWithMembership {
  id: string;
  name: string;
  description: string | null;
  avatar_url: string | null;
  visibility: TeamVisibility;
  join_policy: TeamJoinPolicy;
  owner_id: string;
  member_count: number;
  activity_count: number;
  segment_count: number;
  featured_leaderboard: LeaderboardType | null;
  created_at: string;
  updated_at: string | null;
  user_role: TeamRole | null;
  is_member: boolean;
  owner_name: string;
}

export interface TeamSummary {
  id: string;
  name: string;
  description: string | null;
  avatar_url: string | null;
  member_count: number;
  activity_count: number;
  segment_count: number;
}

export interface TeamMember {
  user_id: string;
  user_name: string;
  role: TeamRole;
  joined_at: string;
  invited_by: string | null;
  invited_by_name: string | null;
}

export interface TeamMembership {
  team_id: string;
  user_id: string;
  role: TeamRole;
  invited_by: string | null;
  joined_at: string;
}

export interface TeamJoinRequest {
  id: string;
  team_id: string;
  user_id: string;
  user_name: string;
  message: string | null;
  status: string;
  created_at: string;
}

export interface TeamInvitation {
  id: string;
  team_id: string;
  email: string;
  invited_by: string;
  role: TeamRole;
  token: string;
  expires_at: string;
  accepted_at: string | null;
  created_at: string;
}

export interface TeamInvitationWithDetails {
  id: string;
  team_id: string;
  team_name: string;
  email: string;
  invited_by: string;
  invited_by_name: string;
  role: TeamRole;
  expires_at: string;
  created_at: string;
}

export interface CreateTeamRequest {
  name: string;
  description?: string;
  avatar_url?: string;
  visibility?: TeamVisibility;
  join_policy?: TeamJoinPolicy;
}

export interface UpdateTeamRequest {
  name?: string;
  description?: string;
  avatar_url?: string;
  visibility?: TeamVisibility;
  join_policy?: TeamJoinPolicy;
  featured_leaderboard?: LeaderboardType;
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

    const startTime = performance.now();
    const response = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers,
    });
    const duration = performance.now() - startTime;
    const requestId = response.headers.get('x-request-id');

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Request failed' }));
      // Log structured API error for monitoring
      console.error('[API_ERROR]', JSON.stringify({
        type: 'api_error',
        timestamp: new Date().toISOString(),
        request_id: requestId,
        method: options.method || 'GET',
        path,
        status: response.status,
        duration_ms: Math.round(duration),
        error: error.error || 'Unknown error',
      }));
      throw new Error(error.error || `Request failed with status ${response.status}`);
    }

    // Handle empty responses (204 No Content, or responses without JSON body)
    const contentType = response.headers.get('content-type');
    const contentLength = response.headers.get('content-length');
    if (response.status === 204 || contentLength === '0' || !contentType?.includes('application/json')) {
      return undefined as T;
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

  // Activity Type endpoints
  async listActivityTypes(): Promise<ActivityType[]> {
    return this.request<ActivityType[]>('/activity-types');
  }

  async getActivityType(id: string): Promise<ActivityType> {
    return this.request<ActivityType>(`/activity-types/${id}`);
  }

  async createActivityType(name: string): Promise<ActivityType> {
    return this.request<ActivityType>('/activity-types', {
      method: 'POST',
      body: JSON.stringify({ name }),
    });
  }

  async resolveActivityType(nameOrAlias: string): Promise<ResolveActivityTypeResponse> {
    const params = new URLSearchParams({ name: nameOrAlias });
    return this.request<ResolveActivityTypeResponse>(`/activity-types/resolve?${params.toString()}`);
  }

  // Activity endpoints
  async getUserActivities(userId: string, filters?: UserActivitiesFilters): Promise<Activity[]> {
    const params = new URLSearchParams();
    if (filters?.activityTypeId) params.set('activity_type_id', filters.activityTypeId);
    if (filters?.dateRange) params.set('date_range', filters.dateRange);
    if (filters?.startDate) params.set('start_date', filters.startDate);
    if (filters?.endDate) params.set('end_date', filters.endDate);
    if (filters?.visibility) params.set('visibility', filters.visibility);
    if (filters?.sortBy) params.set('sort_by', filters.sortBy);
    if (filters?.search) params.set('search', filters.search);
    if (filters?.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters?.offset !== undefined) params.set('offset', filters.offset.toString());
    const queryString = params.toString();
    return this.request<Activity[]>(`/users/${userId}/activities${queryString ? `?${queryString}` : ''}`);
  }

  async getActivity(id: string): Promise<Activity> {
    return this.request<Activity>(`/activities/${id}`);
  }

  async getActivityTrack(id: string): Promise<TrackData> {
    return this.request<TrackData>(`/activities/${id}/track`);
  }

  async getActivitySensorData(id: string): Promise<SensorData | null> {
    try {
      return await this.request<SensorData>(`/activities/${id}/sensor-data`);
    } catch {
      // Return null if no sensor data is available
      return null;
    }
  }

  async getActivitySegments(id: string): Promise<ActivitySegmentEffort[]> {
    return this.request<ActivitySegmentEffort[]>(`/activities/${id}/segments`);
  }

  async updateActivity(
    id: string,
    data: { name?: string; activity_type_id?: string; visibility?: ActivityVisibility }
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

  // Stopped/Dig segment endpoints
  async getStoppedSegments(activityId: string): Promise<StoppedSegment[]> {
    return this.request<StoppedSegment[]>(`/activities/${activityId}/stopped-segments`);
  }

  async getDigSegments(activityId: string): Promise<DigSegment[]> {
    return this.request<DigSegment[]>(`/activities/${activityId}/dig-segments`);
  }

  async createDigSegments(activityId: string, stoppedSegmentIds: string[]): Promise<DigSegment[]> {
    return this.request<DigSegment[]>(`/activities/${activityId}/dig-segments`, {
      method: 'POST',
      body: JSON.stringify({ stopped_segment_ids: stoppedSegmentIds }),
    });
  }

  async deleteDigSegment(activityId: string, segmentId: string): Promise<void> {
    await this.request<void>(`/activities/${activityId}/dig-segments/${segmentId}`, {
      method: 'DELETE',
    });
  }

  async getDigTime(activityId: string): Promise<DigTimeSummary> {
    return this.request<DigTimeSummary>(`/activities/${activityId}/dig-time`);
  }

  /**
   * Upload an activity.
   * @param file - The GPX file to upload
   * @param name - Activity name
   * @param activityTypeId - Activity type UUID
   * @param visibility - Visibility setting
   * @param options - Optional multi-sport settings and team sharing
   */
  async uploadActivity(
    file: File,
    name: string,
    activityTypeId: string,
    visibility: ActivityVisibility = 'public',
    options?: {
      teamIds?: string[];
      // Multi-sport support
      typeBoundaries?: string[];  // ISO8601 timestamps
      segmentTypes?: string[];    // Activity type UUIDs
    }
  ): Promise<Activity> {
    const token = this.getToken();
    const formData = new FormData();
    formData.append('file', file);

    const params = new URLSearchParams({
      activity_type_id: activityTypeId,
      name: name,
      visibility: visibility,
    });

    // Add team_ids as comma-separated list if provided
    if (options?.teamIds && options.teamIds.length > 0) {
      params.set('team_ids', options.teamIds.join(','));
    }

    // Multi-sport support
    if (options?.typeBoundaries && options.typeBoundaries.length > 0) {
      params.set('type_boundaries', options.typeBoundaries.join(','));
    }
    if (options?.segmentTypes && options.segmentTypes.length > 0) {
      params.set('segment_types', options.segmentTypes.join(','));
    }

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
  async listSegments(options?: ListSegmentsOptions): Promise<Segment[]> {
    const params = new URLSearchParams();
    if (options?.activityTypeId) {
      params.set('activity_type_id', options.activityTypeId);
    }
    if (options?.search) {
      params.set('search', options.search);
    }
    if (options?.sortBy) {
      params.set('sort_by', options.sortBy);
    }
    if (options?.sortOrder) {
      params.set('sort_order', options.sortOrder);
    }
    if (options?.minDistanceMeters !== undefined) {
      params.set('min_distance_meters', options.minDistanceMeters.toString());
    }
    if (options?.maxDistanceMeters !== undefined) {
      params.set('max_distance_meters', options.maxDistanceMeters.toString());
    }
    if (options?.climbCategory) {
      params.set('climb_category', options.climbCategory);
    }
    if (options?.limit !== undefined) {
      params.set('limit', options.limit.toString());
    }
    const queryString = params.toString();
    return this.request<Segment[]>(`/segments${queryString ? `?${queryString}` : ''}`);
  }

  async getSegment(id: string): Promise<Segment> {
    return this.request<Segment>(`/segments/${id}`);
  }

  async getSegmentLeaderboard(id: string): Promise<SegmentEffort[]> {
    return this.request<SegmentEffort[]>(`/segments/${id}/leaderboard`);
  }

  async getMySegmentEfforts(id: string): Promise<SegmentEffort[]> {
    return this.request<SegmentEffort[]>(`/segments/${id}/my-efforts`);
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

  async previewSegment(points: { lat: number; lon: number; ele?: number }[]): Promise<PreviewSegmentResponse> {
    return this.request<PreviewSegmentResponse>('/segments/preview', {
      method: 'POST',
      body: JSON.stringify({ points }),
    });
  }

  // Segment star endpoints
  async isSegmentStarred(id: string): Promise<boolean> {
    const result = await this.request<{ starred: boolean }>(`/segments/${id}/star`);
    return result.starred;
  }

  async starSegment(id: string): Promise<void> {
    await this.request<{ starred: boolean }>(`/segments/${id}/star`, {
      method: 'POST',
    });
  }

  async unstarSegment(id: string): Promise<void> {
    await this.request<{ starred: boolean }>(`/segments/${id}/star`, {
      method: 'DELETE',
    });
  }

  async getStarredSegments(): Promise<Segment[]> {
    return this.request<Segment[]>('/segments/starred');
  }

  async getStarredSegmentEfforts(): Promise<StarredSegmentEffort[]> {
    return this.request<StarredSegmentEffort[]>('/segments/starred/efforts');
  }

  async getNearbySegments(lat: number, lon: number, radiusMeters?: number, limit?: number): Promise<Segment[]> {
    const params = new URLSearchParams({
      lat: lat.toString(),
      lon: lon.toString(),
    });
    if (radiusMeters) params.set('radius_meters', radiusMeters.toString());
    if (limit) params.set('limit', limit.toString());
    return this.request<Segment[]>(`/segments/nearby?${params.toString()}`);
  }

  // Filtered leaderboard endpoints
  async getFilteredLeaderboard(
    segmentId: string,
    filters: Partial<LeaderboardFilters>
  ): Promise<LeaderboardResponse> {
    const params = new URLSearchParams();
    if (filters.scope) params.set('scope', filters.scope);
    if (filters.gender) params.set('gender', filters.gender);
    if (filters.age_group) params.set('age_group', filters.age_group);
    if (filters.weight_class) params.set('weight_class', filters.weight_class);
    if (filters.country) params.set('country', filters.country);
    if (filters.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters.offset !== undefined) params.set('offset', filters.offset.toString());
    const queryString = params.toString();
    const path = `/segments/${segmentId}/leaderboard/filtered${queryString ? `?${queryString}` : ''}`;
    return this.request<LeaderboardResponse>(path);
  }

  async getLeaderboardPosition(
    segmentId: string,
    filters: Partial<Pick<LeaderboardFilters, 'scope' | 'gender' | 'age_group' | 'weight_class' | 'country'>>
  ): Promise<LeaderboardPosition> {
    const params = new URLSearchParams();
    if (filters.scope) params.set('scope', filters.scope);
    if (filters.gender) params.set('gender', filters.gender);
    if (filters.age_group) params.set('age_group', filters.age_group);
    if (filters.weight_class) params.set('weight_class', filters.weight_class);
    if (filters.country) params.set('country', filters.country);
    const queryString = params.toString();
    const path = `/segments/${segmentId}/leaderboard/position${queryString ? `?${queryString}` : ''}`;
    return this.request<LeaderboardPosition>(path);
  }

  // Countries endpoint
  async getCountries(): Promise<CountryStats[]> {
    return this.request<CountryStats[]>('/leaderboards/countries');
  }

  // Demographics endpoints
  async getMyDemographics(): Promise<UserWithDemographics> {
    return this.request<UserWithDemographics>('/users/me/demographics');
  }

  async updateMyDemographics(data: UpdateDemographicsRequest): Promise<UserWithDemographics> {
    return this.request<UserWithDemographics>('/users/me/demographics', {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  // Achievement endpoints
  async getMyAchievements(includeLost?: boolean): Promise<AchievementWithSegment[]> {
    const params = includeLost !== undefined ? `?include_lost=${includeLost}` : '';
    return this.request<AchievementWithSegment[]>(`/users/me/achievements${params}`);
  }

  async getUserAchievements(userId: string, includeLost?: boolean): Promise<AchievementWithSegment[]> {
    const params = includeLost !== undefined ? `?include_lost=${includeLost}` : '';
    return this.request<AchievementWithSegment[]>(`/users/${userId}/achievements${params}`);
  }

  async getSegmentAchievements(segmentId: string): Promise<SegmentAchievements> {
    return this.request<SegmentAchievements>(`/segments/${segmentId}/achievements`);
  }

  // Global leaderboard endpoints
  async getCrownLeaderboard(filters?: GlobalLeaderboardFilters): Promise<CrownCountEntry[]> {
    const params = new URLSearchParams();
    if (filters?.scope) params.set('scope', filters.scope);
    if (filters?.gender) params.set('gender', filters.gender);
    if (filters?.ageGroup) params.set('age_group', filters.ageGroup);
    if (filters?.weightClass) params.set('weight_class', filters.weightClass);
    if (filters?.country) params.set('country', filters.country);
    if (filters?.activityTypeId) params.set('activity_type_id', filters.activityTypeId);
    if (filters?.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters?.offset !== undefined) params.set('offset', filters.offset.toString());
    const queryString = params.toString();
    const path = `/leaderboards/crowns${queryString ? `?${queryString}` : ''}`;
    return this.request<CrownCountEntry[]>(path);
  }

  async getDistanceLeaderboard(filters?: GlobalLeaderboardFilters): Promise<DistanceLeaderEntry[]> {
    const params = new URLSearchParams();
    if (filters?.scope) params.set('scope', filters.scope);
    if (filters?.gender) params.set('gender', filters.gender);
    if (filters?.ageGroup) params.set('age_group', filters.ageGroup);
    if (filters?.weightClass) params.set('weight_class', filters.weightClass);
    if (filters?.country) params.set('country', filters.country);
    if (filters?.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters?.offset !== undefined) params.set('offset', filters.offset.toString());
    const queryString = params.toString();
    const path = `/leaderboards/distance${queryString ? `?${queryString}` : ''}`;
    return this.request<DistanceLeaderEntry[]>(path);
  }

  async getDigTimeLeaderboard(filters?: GlobalLeaderboardFilters): Promise<DigTimeLeaderEntry[]> {
    const params = new URLSearchParams();
    if (filters?.gender) params.set('gender', filters.gender);
    if (filters?.ageGroup) params.set('age_group', filters.ageGroup);
    if (filters?.weightClass) params.set('weight_class', filters.weightClass);
    if (filters?.country) params.set('country', filters.country);
    if (filters?.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters?.offset !== undefined) params.set('offset', filters.offset.toString());
    const queryString = params.toString();
    const path = `/leaderboards/dig-time${queryString ? `?${queryString}` : ''}`;
    return this.request<DigTimeLeaderEntry[]>(path);
  }

  async getDigPercentageLeaderboard(filters?: GlobalLeaderboardFilters): Promise<DigPercentageLeaderEntry[]> {
    const params = new URLSearchParams();
    if (filters?.scope) params.set('scope', filters.scope);
    if (filters?.gender) params.set('gender', filters.gender);
    if (filters?.ageGroup) params.set('age_group', filters.ageGroup);
    if (filters?.weightClass) params.set('weight_class', filters.weightClass);
    if (filters?.country) params.set('country', filters.country);
    if (filters?.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters?.offset !== undefined) params.set('offset', filters.offset.toString());
    const queryString = params.toString();
    const path = `/leaderboards/dig-percentage${queryString ? `?${queryString}` : ''}`;
    return this.request<DigPercentageLeaderEntry[]>(path);
  }

  async getAverageSpeedLeaderboard(filters?: GlobalLeaderboardFilters): Promise<AverageSpeedLeaderEntry[]> {
    const params = new URLSearchParams();
    if (filters?.scope) params.set('scope', filters.scope);
    if (filters?.gender) params.set('gender', filters.gender);
    if (filters?.ageGroup) params.set('age_group', filters.ageGroup);
    if (filters?.weightClass) params.set('weight_class', filters.weightClass);
    if (filters?.country) params.set('country', filters.country);
    if (filters?.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters?.offset !== undefined) params.set('offset', filters.offset.toString());
    const queryString = params.toString();
    const path = `/leaderboards/average-speed${queryString ? `?${queryString}` : ''}`;
    return this.request<AverageSpeedLeaderEntry[]>(path);
  }

  // Team leaderboard endpoint
  async getTeamLeaderboard(
    teamId: string,
    leaderboardType: LeaderboardType,
    filters?: GlobalLeaderboardFilters
  ): Promise<CrownCountEntry[] | DistanceLeaderEntry[] | DigTimeLeaderEntry[] | DigPercentageLeaderEntry[] | AverageSpeedLeaderEntry[]> {
    const params = new URLSearchParams();
    if (filters?.scope) params.set('scope', filters.scope);
    if (filters?.gender) params.set('gender', filters.gender);
    if (filters?.ageGroup) params.set('age_group', filters.ageGroup);
    if (filters?.weightClass) params.set('weight_class', filters.weightClass);
    if (filters?.country) params.set('country', filters.country);
    if (filters?.activityTypeId) params.set('activity_type_id', filters.activityTypeId);
    if (filters?.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters?.offset !== undefined) params.set('offset', filters.offset.toString());
    const queryString = params.toString();
    const path = `/teams/${teamId}/leaderboard/${leaderboardType}${queryString ? `?${queryString}` : ''}`;
    return this.request(path);
  }

  // Social endpoints (follows)
  async getUserProfile(userId: string): Promise<UserProfile> {
    return this.request<UserProfile>(`/users/${userId}/profile`);
  }

  async followUser(userId: string): Promise<void> {
    await this.request<void>(`/users/${userId}/follow`, {
      method: 'POST',
    });
  }

  async unfollowUser(userId: string): Promise<void> {
    await this.request<void>(`/users/${userId}/follow`, {
      method: 'DELETE',
    });
  }

  async getFollowStatus(userId: string): Promise<boolean> {
    const result = await this.request<FollowStatusResponse>(`/users/${userId}/follow`);
    return result.is_following;
  }

  async getFollowers(userId: string, limit?: number, offset?: number): Promise<FollowListResponse> {
    const params = new URLSearchParams();
    if (limit !== undefined) params.set('limit', limit.toString());
    if (offset !== undefined) params.set('offset', offset.toString());
    const queryString = params.toString();
    return this.request<FollowListResponse>(`/users/${userId}/followers${queryString ? `?${queryString}` : ''}`);
  }

  async getFollowing(userId: string, limit?: number, offset?: number): Promise<FollowListResponse> {
    const params = new URLSearchParams();
    if (limit !== undefined) params.set('limit', limit.toString());
    if (offset !== undefined) params.set('offset', offset.toString());
    const queryString = params.toString();
    return this.request<FollowListResponse>(`/users/${userId}/following${queryString ? `?${queryString}` : ''}`);
  }

  // Notification endpoints
  async getNotifications(limit?: number, offset?: number): Promise<NotificationsResponse> {
    const params = new URLSearchParams();
    if (limit !== undefined) params.set('limit', limit.toString());
    if (offset !== undefined) params.set('offset', offset.toString());
    const queryString = params.toString();
    return this.request<NotificationsResponse>(`/notifications${queryString ? `?${queryString}` : ''}`);
  }

  async markNotificationRead(notificationId: string): Promise<void> {
    await this.request<void>(`/notifications/${notificationId}/read`, {
      method: 'POST',
    });
  }

  async markAllNotificationsRead(): Promise<{ marked_count: number }> {
    return this.request<{ marked_count: number }>('/notifications/read-all', {
      method: 'POST',
    });
  }

  // Activity feed endpoints
  async getFeed(filters?: FeedFilters): Promise<FeedActivity[]> {
    const params = new URLSearchParams();
    if (filters?.activityTypeId) params.set('activity_type_id', filters.activityTypeId);
    if (filters?.dateRange) params.set('date_range', filters.dateRange);
    if (filters?.startDate) params.set('start_date', filters.startDate);
    if (filters?.endDate) params.set('end_date', filters.endDate);
    if (filters?.limit !== undefined) params.set('limit', filters.limit.toString());
    if (filters?.offset !== undefined) params.set('offset', filters.offset.toString());
    const queryString = params.toString();
    return this.request<FeedActivity[]>(`/feed${queryString ? `?${queryString}` : ''}`);
  }

  // Kudos endpoints
  async giveKudos(activityId: string): Promise<void> {
    await this.request<void>(`/activities/${activityId}/kudos`, {
      method: 'POST',
    });
  }

  async removeKudos(activityId: string): Promise<void> {
    await this.request<void>(`/activities/${activityId}/kudos`, {
      method: 'DELETE',
    });
  }

  async getKudosStatus(activityId: string): Promise<boolean> {
    const result = await this.request<KudosStatusResponse>(`/activities/${activityId}/kudos`);
    return result.has_given;
  }

  async getKudosGivers(activityId: string): Promise<KudosGiver[]> {
    return this.request<KudosGiver[]>(`/activities/${activityId}/kudos/givers`);
  }

  // Comments endpoints
  async getComments(activityId: string): Promise<Comment[]> {
    return this.request<Comment[]>(`/activities/${activityId}/comments`);
  }

  async addComment(activityId: string, content: string, parentId?: string): Promise<Comment> {
    return this.request<Comment>(`/activities/${activityId}/comments`, {
      method: 'POST',
      body: JSON.stringify({ content, parent_id: parentId }),
    });
  }

  async deleteComment(commentId: string): Promise<void> {
    await this.request<void>(`/comments/${commentId}`, {
      method: 'DELETE',
    });
  }

  // Stats endpoint
  async getStats(): Promise<Stats> {
    return this.request<Stats>('/stats');
  }

  // ============================================================================
  // Team endpoints
  // ============================================================================

  async createTeam(data: CreateTeamRequest): Promise<Team> {
    return this.request<Team>('/teams', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getTeam(id: string): Promise<TeamWithMembership> {
    return this.request<TeamWithMembership>(`/teams/${id}`);
  }

  async listMyTeams(): Promise<TeamWithMembership[]> {
    return this.request<TeamWithMembership[]>('/teams');
  }

  async discoverTeams(limit?: number, offset?: number): Promise<TeamSummary[]> {
    const params = new URLSearchParams();
    if (limit !== undefined) params.set('limit', limit.toString());
    if (offset !== undefined) params.set('offset', offset.toString());
    const queryString = params.toString();
    return this.request<TeamSummary[]>(`/teams/discover${queryString ? `?${queryString}` : ''}`);
  }

  async updateTeam(id: string, data: UpdateTeamRequest): Promise<Team> {
    return this.request<Team>(`/teams/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteTeam(id: string): Promise<void> {
    await this.request<void>(`/teams/${id}`, {
      method: 'DELETE',
    });
  }

  // Team membership endpoints
  async listTeamMembers(teamId: string): Promise<TeamMember[]> {
    return this.request<TeamMember[]>(`/teams/${teamId}/members`);
  }

  async removeTeamMember(teamId: string, userId: string): Promise<void> {
    await this.request<void>(`/teams/${teamId}/members/${userId}`, {
      method: 'DELETE',
    });
  }

  async changeTeamMemberRole(teamId: string, userId: string, role: TeamRole): Promise<TeamMembership> {
    return this.request<TeamMembership>(`/teams/${teamId}/members/${userId}/role`, {
      method: 'PATCH',
      body: JSON.stringify({ role }),
    });
  }

  // Team join endpoints
  async joinTeam(teamId: string, message?: string): Promise<void> {
    await this.request<void>(`/teams/${teamId}/join`, {
      method: 'POST',
      body: JSON.stringify({ message }),
    });
  }

  async leaveTeam(teamId: string): Promise<void> {
    await this.request<void>(`/teams/${teamId}/leave`, {
      method: 'POST',
    });
  }

  async getJoinRequests(teamId: string): Promise<TeamJoinRequest[]> {
    return this.request<TeamJoinRequest[]>(`/teams/${teamId}/join-requests`);
  }

  async reviewJoinRequest(teamId: string, requestId: string, approved: boolean): Promise<void> {
    await this.request<void>(`/teams/${teamId}/join-requests/${requestId}`, {
      method: 'POST',
      body: JSON.stringify({ approved }),
    });
  }

  // Team invitation endpoints
  async inviteToTeam(teamId: string, email: string, role: TeamRole = 'member'): Promise<TeamInvitation> {
    return this.request<TeamInvitation>(`/teams/${teamId}/invitations`, {
      method: 'POST',
      body: JSON.stringify({ email, role }),
    });
  }

  async getTeamInvitations(teamId: string): Promise<TeamInvitation[]> {
    return this.request<TeamInvitation[]>(`/teams/${teamId}/invitations`);
  }

  async revokeInvitation(teamId: string, invitationId: string): Promise<void> {
    await this.request<void>(`/teams/${teamId}/invitations/${invitationId}`, {
      method: 'DELETE',
    });
  }

  async getInvitation(token: string): Promise<TeamInvitationWithDetails> {
    return this.request<TeamInvitationWithDetails>(`/invitations/${token}`);
  }

  async acceptInvitation(token: string): Promise<void> {
    await this.request<void>(`/invitations/${token}/accept`, {
      method: 'POST',
    });
  }

  // Activity-team sharing endpoints
  async getActivityTeams(activityId: string): Promise<TeamSummary[]> {
    return this.request<TeamSummary[]>(`/activities/${activityId}/teams`);
  }

  async shareActivityWithTeams(activityId: string, teamIds: string[]): Promise<void> {
    await this.request<void>(`/activities/${activityId}/teams`, {
      method: 'POST',
      body: JSON.stringify({ team_ids: teamIds }),
    });
  }

  async unshareActivityFromTeam(activityId: string, teamId: string): Promise<void> {
    await this.request<void>(`/activities/${activityId}/teams/${teamId}`, {
      method: 'DELETE',
    });
  }

  // Segment-team sharing endpoints
  async getSegmentTeams(segmentId: string): Promise<TeamSummary[]> {
    return this.request<TeamSummary[]>(`/segments/${segmentId}/teams`);
  }

  async shareSegmentWithTeams(segmentId: string, teamIds: string[]): Promise<void> {
    await this.request<void>(`/segments/${segmentId}/teams`, {
      method: 'POST',
      body: JSON.stringify({ team_ids: teamIds }),
    });
  }

  async unshareSegmentFromTeam(segmentId: string, teamId: string): Promise<void> {
    await this.request<void>(`/segments/${segmentId}/teams/${teamId}`, {
      method: 'DELETE',
    });
  }

  // Daily activities endpoint
  async getActivitiesByDate(date: string, mineOnly?: boolean): Promise<FeedActivity[]> {
    const params = new URLSearchParams({ date });
    if (mineOnly !== undefined) {
      params.set('mine_only', mineOnly.toString());
    }
    return this.request<FeedActivity[]>(`/activities/by-date?${params.toString()}`);
  }

  // Team content endpoints
  async getTeamActivities(teamId: string, limit?: number, offset?: number): Promise<FeedActivity[]> {
    const params = new URLSearchParams();
    if (limit !== undefined) params.set('limit', limit.toString());
    if (offset !== undefined) params.set('offset', offset.toString());
    const queryString = params.toString();
    return this.request<FeedActivity[]>(`/teams/${teamId}/activities${queryString ? `?${queryString}` : ''}`);
  }

  async getTeamSegments(teamId: string, limit?: number, offset?: number): Promise<Segment[]> {
    const params = new URLSearchParams();
    if (limit !== undefined) params.set('limit', limit.toString());
    if (offset !== undefined) params.set('offset', offset.toString());
    const queryString = params.toString();
    return this.request<Segment[]>(`/teams/${teamId}/segments${queryString ? `?${queryString}` : ''}`);
  }

  async getTeamActivitiesByDate(
    teamId: string,
    date: string,
    limit?: number,
    offset?: number
  ): Promise<FeedActivity[]> {
    const params = new URLSearchParams({ date });
    if (limit !== undefined) params.set('limit', limit.toString());
    if (offset !== undefined) params.set('offset', offset.toString());
    return this.request<FeedActivity[]>(`/teams/${teamId}/activities/daily?${params.toString()}`);
  }
}

export const api = new ApiClient();
