use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use time::serde::rfc3339;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}
impl User {
    pub fn new(email: String, name: String) -> Self {
        let id = Uuid::new_v4();
        let created_at = OffsetDateTime::now_utc();
        Self {
            id,
            email,
            name,
            created_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Public,
    Private,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
        }
    }
}

impl std::str::FromStr for Visibility {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "public" => Ok(Visibility::Public),
            "private" => Ok(Visibility::Private),
            _ => Err(format!("unknown visibility: {s}")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_type_id: Uuid,
    pub name: String,
    pub object_store_path: String,
    #[serde(with = "rfc3339")]
    pub submitted_at: OffsetDateTime,
    pub visibility: String,
    // Multi-sport support: boundaries mark segment transitions, types are parallel to segments
    pub type_boundaries: Option<Vec<OffsetDateTime>>,
    pub segment_types: Option<Vec<Uuid>>,
}

// ============================================================================
// Activity Type Models (table-based, replaces enum)
// ============================================================================

/// Built-in activity type UUIDs for compile-time constants
pub mod builtin_types {
    use uuid::Uuid;

    pub const WALK: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000001);
    pub const RUN: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000002);
    pub const HIKE: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000003);
    pub const ROAD: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000004);
    pub const MTB: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000005);
    pub const EMTB: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000006);
    pub const GRAVEL: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000007);
    pub const UNKNOWN: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000008);
}

/// Row from the activity_types table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityTypeRow {
    pub id: Uuid,
    pub name: String,
    pub is_builtin: bool,
    pub created_by: Option<Uuid>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Row from the activity_aliases table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityAliasRow {
    pub id: Uuid,
    pub alias: String,
    pub activity_type_id: Uuid,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Result of resolving an activity type by name or alias
#[derive(Debug, Clone)]
pub enum ResolvedActivityType {
    /// Direct name match or single alias match
    Exact(Uuid),
    /// Multiple alias matches - user must pick
    Ambiguous(Vec<Uuid>),
    /// No matching type found
    NotFound,
}

/// Request to create a custom activity type
#[derive(Debug, Deserialize)]
pub struct CreateActivityTypeRequest {
    pub name: String,
}

/// Request to create an activity alias
#[derive(Debug, Deserialize)]
pub struct CreateActivityAliasRequest {
    pub alias: String,
    pub activity_type_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub user_id: Uuid,
    pub activity_type_id: Uuid,
    /// Multi-sport: timestamps marking segment boundaries
    pub type_boundaries: Option<Vec<OffsetDateTime>>,
    /// Multi-sport: activity type IDs for each segment
    pub segment_types: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ScoresRow {
    pub user_id: Uuid,
    pub activity_id: Uuid,
    #[sqlx(flatten)]
    pub scores: Scores,
    pub created_at: OffsetDateTime,
}
#[derive(Debug, Clone, Default, sqlx::Type)]
pub struct Scores {
    pub distance: f64,
    pub duration: f64,
    pub elevation_gain: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Segment {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub activity_type_id: Uuid,
    pub distance_meters: f64,
    pub elevation_gain_meters: Option<f64>,
    pub elevation_loss_meters: Option<f64>,
    pub average_grade: Option<f64>,
    pub max_grade: Option<f64>,
    pub climb_category: Option<i32>,
    pub visibility: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SegmentEffort {
    pub id: Uuid,
    pub segment_id: Uuid,
    pub activity_id: Uuid,
    pub user_id: Uuid,
    #[serde(with = "rfc3339")]
    pub started_at: OffsetDateTime,
    pub elapsed_time_seconds: f64,
    pub moving_time_seconds: Option<f64>,
    pub average_speed_mps: Option<f64>,
    pub max_speed_mps: Option<f64>,
    pub is_personal_record: bool,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    pub start_fraction: Option<f64>,
    pub end_fraction: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentWithStats {
    #[serde(flatten)]
    pub segment: Segment,
    pub effort_count: i64,
    pub athlete_count: i64,
    pub creator_name: String,
}

/// Segment effort with segment details, for displaying on activity detail page.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ActivitySegmentEffort {
    pub effort_id: Uuid,
    pub segment_id: Uuid,
    pub elapsed_time_seconds: f64,
    pub is_personal_record: bool,
    #[serde(with = "rfc3339")]
    pub started_at: OffsetDateTime,
    pub segment_name: String,
    pub segment_distance: f64,
    pub activity_type_id: Uuid,
    pub rank: i64,
    pub start_fraction: Option<f64>,
    pub end_fraction: Option<f64>,
}

/// Starred segment with the user's effort stats, for the starred segments dashboard.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct StarredSegmentEffort {
    // Segment basic info
    pub segment_id: Uuid,
    pub segment_name: String,
    pub activity_type_id: Uuid,
    pub distance_meters: f64,
    pub elevation_gain_meters: Option<f64>,
    // User's best effort
    pub best_time_seconds: Option<f64>,
    pub best_effort_rank: Option<i64>,
    #[serde(with = "rfc3339::option")]
    pub best_effort_date: Option<OffsetDateTime>,
    // User's effort count on this segment
    pub user_effort_count: i64,
    // Segment leader time for comparison
    pub leader_time_seconds: Option<f64>,
}

// ============================================================================
// Leaderboard Models
// ============================================================================

/// User gender for demographic filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "gender", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Other,
    #[default]
    PreferNotToSay,
}

impl std::str::FromStr for Gender {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            "other" => Ok(Gender::Other),
            "prefer_not_to_say" => Ok(Gender::PreferNotToSay),
            _ => Err(format!("unknown gender: {s}")),
        }
    }
}

/// Time scope for leaderboard filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardScope {
    #[default]
    AllTime,
    Year,
    Month,
    Week,
}

impl std::str::FromStr for LeaderboardScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all_time" => Ok(LeaderboardScope::AllTime),
            "year" => Ok(LeaderboardScope::Year),
            "month" => Ok(LeaderboardScope::Month),
            "week" => Ok(LeaderboardScope::Week),
            _ => Err(format!("unknown scope: {s}")),
        }
    }
}

/// Age group for demographic filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgeGroup {
    #[default]
    All,
    #[serde(rename = "18-24")]
    Age18To24,
    #[serde(rename = "25-34")]
    Age25To34,
    #[serde(rename = "35-44")]
    Age35To44,
    #[serde(rename = "45-54")]
    Age45To54,
    #[serde(rename = "55-64")]
    Age55To64,
    #[serde(rename = "65+")]
    Age65Plus,
}

impl AgeGroup {
    /// Returns the age range (min, max) for this group. Max is None for 65+.
    pub fn age_range(&self) -> Option<(i32, Option<i32>)> {
        match self {
            AgeGroup::All => None,
            AgeGroup::Age18To24 => Some((18, Some(24))),
            AgeGroup::Age25To34 => Some((25, Some(34))),
            AgeGroup::Age35To44 => Some((35, Some(44))),
            AgeGroup::Age45To54 => Some((45, Some(54))),
            AgeGroup::Age55To64 => Some((55, Some(64))),
            AgeGroup::Age65Plus => Some((65, None)),
        }
    }
}

impl std::str::FromStr for AgeGroup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(AgeGroup::All),
            "18-24" => Ok(AgeGroup::Age18To24),
            "25-34" => Ok(AgeGroup::Age25To34),
            "35-44" => Ok(AgeGroup::Age35To44),
            "45-54" => Ok(AgeGroup::Age45To54),
            "55-64" => Ok(AgeGroup::Age55To64),
            "65+" | "65_plus" => Ok(AgeGroup::Age65Plus),
            _ => Err(format!("unknown age group: {s}")),
        }
    }
}

/// Gender filter for leaderboards (includes "all" option)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GenderFilter {
    #[default]
    All,
    Male,
    Female,
}

impl std::str::FromStr for GenderFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(GenderFilter::All),
            "male" => Ok(GenderFilter::Male),
            "female" => Ok(GenderFilter::Female),
            _ => Err(format!("unknown gender filter: {s}")),
        }
    }
}

/// Query parameters for filtered leaderboard requests
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LeaderboardFilters {
    #[serde(default)]
    pub scope: LeaderboardScope,
    #[serde(default)]
    pub gender: GenderFilter,
    #[serde(default)]
    pub age_group: AgeGroup,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// A single entry in the leaderboard with user info and ranking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaderboardEntry {
    // Effort data
    pub effort_id: Uuid,
    pub elapsed_time_seconds: f64,
    pub moving_time_seconds: Option<f64>,
    pub average_speed_mps: Option<f64>,
    #[serde(with = "rfc3339")]
    pub started_at: OffsetDateTime,
    pub is_personal_record: bool,

    // User data
    pub user_id: Uuid,
    pub user_name: String,

    // Ranking data
    pub rank: i64,
    pub gap_seconds: Option<f64>,
}

/// Response for paginated leaderboard
#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub total_count: i64,
    pub filters: LeaderboardFiltersResponse,
}

/// Echoed filters in leaderboard response
#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardFiltersResponse {
    pub scope: LeaderboardScope,
    pub gender: GenderFilter,
    pub age_group: AgeGroup,
    pub limit: i64,
    pub offset: i64,
}

/// User's position in the leaderboard with surrounding entries
#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardPosition {
    pub user_rank: i64,
    pub user_entry: LeaderboardEntry,
    pub entries_above: Vec<LeaderboardEntry>,
    pub entries_below: Vec<LeaderboardEntry>,
    pub total_count: i64,
}

// ============================================================================
// Achievement Models
// ============================================================================

/// Type of achievement/crown
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "achievement_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AchievementType {
    Kom,
    Qom,
    LocalLegend,
    CourseRecord,
}

impl std::fmt::Display for AchievementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AchievementType::Kom => write!(f, "KOM"),
            AchievementType::Qom => write!(f, "QOM"),
            AchievementType::LocalLegend => write!(f, "Local Legend"),
            AchievementType::CourseRecord => write!(f, "Course Record"),
        }
    }
}

/// An achievement/crown earned by a user
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Achievement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub segment_id: Uuid,
    pub effort_id: Option<Uuid>,
    pub achievement_type: AchievementType,
    #[serde(with = "rfc3339")]
    pub earned_at: OffsetDateTime,
    #[serde(with = "rfc3339::option")]
    pub lost_at: Option<OffsetDateTime>,
    pub effort_count: Option<i32>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Achievement with segment details for display
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AchievementWithSegment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub segment_id: Uuid,
    pub effort_id: Option<Uuid>,
    pub achievement_type: AchievementType,
    #[serde(with = "rfc3339")]
    pub earned_at: OffsetDateTime,
    #[serde(with = "rfc3339::option")]
    pub lost_at: Option<OffsetDateTime>,
    pub effort_count: Option<i32>,
    // Segment details
    pub segment_name: String,
    pub segment_distance_meters: f64,
    pub segment_activity_type_id: Uuid,
}

/// Current achievement holders for a segment
#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentAchievements {
    pub segment_id: Uuid,
    pub kom: Option<AchievementHolder>,
    pub qom: Option<AchievementHolder>,
    pub local_legend: Option<AchievementHolder>,
}

/// Holder of an achievement with their details
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AchievementHolder {
    pub user_id: Uuid,
    pub user_name: String,
    pub achievement_type: AchievementType,
    #[serde(with = "rfc3339")]
    pub earned_at: OffsetDateTime,
    pub elapsed_time_seconds: Option<f64>,
    pub effort_count: Option<i32>,
}

// ============================================================================
// User Demographics Models
// ============================================================================

/// User profile with demographics
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserWithDemographics {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    pub gender: Option<Gender>,
    pub birth_year: Option<i32>,
    pub weight_kg: Option<f64>,
    pub country: Option<String>,
    pub region: Option<String>,
}

/// Request to update user demographics
#[derive(Debug, Deserialize)]
pub struct UpdateDemographicsRequest {
    pub gender: Option<Gender>,
    pub birth_year: Option<i32>,
    pub weight_kg: Option<f64>,
    pub country: Option<String>,
    pub region: Option<String>,
}

// ============================================================================
// Global Leaderboard Models
// ============================================================================

/// Entry in global crown count leaderboard
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CrownCountEntry {
    pub user_id: Uuid,
    pub user_name: String,
    pub kom_count: i64,
    pub qom_count: i64,
    pub local_legend_count: i64,
    pub total_crowns: i64,
    pub rank: i64,
}

/// Entry in global distance leaderboard
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DistanceLeaderEntry {
    pub user_id: Uuid,
    pub user_name: String,
    pub total_distance_meters: f64,
    pub activity_count: i64,
    pub rank: i64,
}

// ============================================================================
// Social Models (Follows, Notifications)
// ============================================================================

/// A follow relationship between two users
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Follow {
    pub follower_id: Uuid,
    pub following_id: Uuid,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// User profile with follow counts for display
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    pub follower_count: i32,
    pub following_count: i32,
    // Demographics
    pub gender: Option<Gender>,
    pub birth_year: Option<i32>,
    pub weight_kg: Option<f64>,
    pub country: Option<String>,
    pub region: Option<String>,
}

/// Summary of a user for follower/following lists
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserSummary {
    pub id: Uuid,
    pub name: String,
    pub follower_count: i32,
    pub following_count: i32,
    #[serde(with = "rfc3339")]
    pub followed_at: OffsetDateTime,
}

/// Type of notification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    Follow,
    Kudos,
    Comment,
    CrownAchieved,
    CrownLost,
    PersonalRecord,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationType::Follow => "follow",
            NotificationType::Kudos => "kudos",
            NotificationType::Comment => "comment",
            NotificationType::CrownAchieved => "crown_achieved",
            NotificationType::CrownLost => "crown_lost",
            NotificationType::PersonalRecord => "pr",
        }
    }
}

impl std::str::FromStr for NotificationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "follow" => Ok(NotificationType::Follow),
            "kudos" => Ok(NotificationType::Kudos),
            "comment" => Ok(NotificationType::Comment),
            "crown_achieved" => Ok(NotificationType::CrownAchieved),
            "crown_lost" => Ok(NotificationType::CrownLost),
            "pr" => Ok(NotificationType::PersonalRecord),
            _ => Err(format!("unknown notification type: {s}")),
        }
    }
}

/// A notification for a user
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub actor_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub message: Option<String>,
    #[serde(with = "rfc3339::option")]
    pub read_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Notification with actor details for display
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotificationWithActor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub message: Option<String>,
    #[serde(with = "rfc3339::option")]
    pub read_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Response for notifications list with unread count
#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationsResponse {
    pub notifications: Vec<NotificationWithActor>,
    pub unread_count: i64,
    pub total_count: i64,
}

// ============================================================================
// Activity Feed Models
// ============================================================================

/// Activity with user and stats for the feed
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FeedActivity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub activity_type_id: Uuid,
    #[serde(with = "rfc3339")]
    pub submitted_at: OffsetDateTime,
    pub visibility: String,
    pub user_name: String,
    pub distance: Option<f64>,
    pub duration: Option<f64>,
    pub elevation_gain: Option<f64>,
    pub kudos_count: i32,
    pub comment_count: i32,
}

// ============================================================================
// Kudos and Comments Models
// ============================================================================

/// User who gave kudos
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct KudosGiver {
    pub user_id: Uuid,
    pub user_name: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// A comment on an activity
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339::option")]
    pub updated_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
}

/// Comment with user info for display
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CommentWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339::option")]
    pub updated_at: Option<OffsetDateTime>,
    pub user_name: String,
}

// ============================================================================
// Stats Models
// ============================================================================

/// Platform statistics for the homepage
#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub active_users: i64,
    pub segments_created: i64,
    pub activities_uploaded: i64,
}

// ============================================================================
// Track Point Data for Storage
// ============================================================================

/// Track point with all 4 dimensions for storage in LineStringZM geometry
#[derive(Debug, Clone)]
pub struct TrackPointData {
    pub lat: f64,
    pub lon: f64,
    pub elevation: Option<f64>,
    pub timestamp: Option<OffsetDateTime>,
}

// ============================================================================
// Team Models
// ============================================================================

/// Team role enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "team_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TeamRole {
    Owner,
    Admin,
    #[default]
    Member,
}

impl TeamRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamRole::Owner => "owner",
            TeamRole::Admin => "admin",
            TeamRole::Member => "member",
        }
    }

    /// Returns true if this role can manage members (invite, remove, change roles)
    pub fn can_manage_members(&self) -> bool {
        matches!(self, TeamRole::Owner | TeamRole::Admin)
    }

    /// Returns true if this role can modify team settings
    pub fn can_modify_team(&self) -> bool {
        matches!(self, TeamRole::Owner | TeamRole::Admin)
    }

    /// Returns true if this role can delete the team
    pub fn can_delete_team(&self) -> bool {
        matches!(self, TeamRole::Owner)
    }
}

impl std::str::FromStr for TeamRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(TeamRole::Owner),
            "admin" => Ok(TeamRole::Admin),
            "member" => Ok(TeamRole::Member),
            _ => Err(format!("unknown team role: {s}")),
        }
    }
}

/// Team visibility (whether team is discoverable)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "team_visibility", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TeamVisibility {
    /// Team is discoverable in team listings
    Public,
    /// Team is only visible to members
    #[default]
    Private,
}

impl TeamVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamVisibility::Public => "public",
            TeamVisibility::Private => "private",
        }
    }
}

impl std::str::FromStr for TeamVisibility {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "public" => Ok(TeamVisibility::Public),
            "private" => Ok(TeamVisibility::Private),
            _ => Err(format!("unknown team visibility: {s}")),
        }
    }
}

/// Team join policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "team_join_policy", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TeamJoinPolicy {
    /// Anyone can join without approval
    Open,
    /// Users can request to join, requires admin approval
    Request,
    /// Users can only join via invitation
    #[default]
    Invitation,
}

impl TeamJoinPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamJoinPolicy::Open => "open",
            TeamJoinPolicy::Request => "request",
            TeamJoinPolicy::Invitation => "invitation",
        }
    }
}

impl std::str::FromStr for TeamJoinPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(TeamJoinPolicy::Open),
            "request" => Ok(TeamJoinPolicy::Request),
            "invitation" => Ok(TeamJoinPolicy::Invitation),
            _ => Err(format!("unknown team join policy: {s}")),
        }
    }
}

/// A team for sharing activities and segments
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub visibility: TeamVisibility,
    pub join_policy: TeamJoinPolicy,
    pub owner_id: Uuid,
    pub member_count: i32,
    pub activity_count: i32,
    pub segment_count: i32,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339::option")]
    pub updated_at: Option<OffsetDateTime>,
}

/// Team with additional context for the current user
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamWithMembership {
    #[serde(flatten)]
    pub team: Team,
    pub user_role: Option<TeamRole>,
    pub is_member: bool,
    pub owner_name: String,
}

/// A membership in a team
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TeamMembership {
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: TeamRole,
    pub invited_by: Option<Uuid>,
    #[serde(with = "rfc3339")]
    pub joined_at: OffsetDateTime,
}

/// Team member with user details
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TeamMember {
    pub user_id: Uuid,
    pub user_name: String,
    pub role: TeamRole,
    #[serde(with = "rfc3339")]
    pub joined_at: OffsetDateTime,
    pub invited_by: Option<Uuid>,
    pub invited_by_name: Option<String>,
}

/// Request to join a team
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TeamJoinRequest {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub message: Option<String>,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    #[serde(with = "rfc3339::option")]
    pub reviewed_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Join request with user details for admin review
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TeamJoinRequestWithUser {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub message: Option<String>,
    pub status: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// An invitation to join a team
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TeamInvitation {
    pub id: Uuid,
    pub team_id: Uuid,
    pub email: String,
    pub invited_by: Uuid,
    pub role: TeamRole,
    pub token: String,
    #[serde(with = "rfc3339")]
    pub expires_at: OffsetDateTime,
    #[serde(with = "rfc3339::option")]
    pub accepted_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Invitation with additional context
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TeamInvitationWithDetails {
    pub id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub email: String,
    pub invited_by: Uuid,
    pub invited_by_name: String,
    pub role: TeamRole,
    #[serde(with = "rfc3339")]
    pub expires_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Request to create a team
#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub visibility: TeamVisibility,
    #[serde(default)]
    pub join_policy: TeamJoinPolicy,
}

/// Request to update a team
#[derive(Debug, Deserialize)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub visibility: Option<TeamVisibility>,
    pub join_policy: Option<TeamJoinPolicy>,
}

/// Request to invite a user to a team
#[derive(Debug, Deserialize)]
pub struct InviteToTeamRequest {
    pub email: String,
    #[serde(default)]
    pub role: TeamRole,
}

/// Request to change a member's role
#[derive(Debug, Deserialize)]
pub struct ChangeMemberRoleRequest {
    pub role: TeamRole,
}

/// Request to join a team
#[derive(Debug, Deserialize)]
pub struct JoinTeamRequest {
    pub message: Option<String>,
}

/// Request to share activity/segment with teams
#[derive(Debug, Deserialize)]
pub struct ShareWithTeamsRequest {
    pub team_ids: Vec<Uuid>,
}

/// Team summary for listings
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TeamSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub member_count: i32,
    pub activity_count: i32,
    pub segment_count: i32,
}
