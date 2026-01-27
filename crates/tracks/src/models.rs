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
    pub activity_type: ActivityType,
    pub name: String,
    pub object_store_path: String,
    #[serde(with = "rfc3339")]
    pub submitted_at: OffsetDateTime,
    pub visibility: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "activity_type", rename_all = "snake_case")]
pub enum ActivityType {
    Walking,
    Running,
    Hiking,
    RoadCycling,
    MountainBiking,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub user_id: Uuid,
    pub activity_type: ActivityType,
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
    pub activity_type: ActivityType,
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
    pub activity_type: ActivityType,
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
    pub activity_type: ActivityType,
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
    pub segment_activity_type: ActivityType,
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
    pub activity_type: String,
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
