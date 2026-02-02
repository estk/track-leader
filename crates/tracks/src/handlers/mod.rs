//! HTTP request handlers for the tracks API.
//!
//! This module re-exports handlers from focused submodules organized by domain.

// Utility submodules
pub mod pagination;

// Handler modules
pub mod achievements;
pub mod activities;
pub mod activity_types;
pub mod demographics;
pub mod leaderboards;
pub mod segments;
pub mod social;
pub mod stats;
pub mod teams;
pub mod users;

// Re-export handlers from submodules (including utoipa __path types for OpenAPI)
pub use achievements::{
    __path_get_my_achievements, __path_get_segment_achievements, __path_get_user_achievements,
    GetAchievementsQuery, get_my_achievements, get_segment_achievements, get_user_achievements,
};
pub use activities::{
    __path_create_dig_parts, __path_delete_activity, __path_delete_dig_part,
    __path_download_gpx_file, __path_get_activities_by_date, __path_get_activity,
    __path_get_activity_segments, __path_get_activity_sensor_data, __path_get_activity_track,
    __path_get_dig_parts, __path_get_dig_time, __path_get_stopped_segments,
    __path_get_user_activities, __path_new_activity, __path_reprocess_dig_parts,
    __path_update_activity, ActivitiesByDateQuery, ReprocessDigPartsResult, TrackBounds, TrackData,
    TrackPoint, UpdateActivityRequest, UploadQuery, UserActivitiesQuery, create_dig_parts,
    delete_activity, delete_dig_part, download_gpx_file, get_activities_by_date, get_activity,
    get_activity_segments, get_activity_sensor_data, get_activity_track, get_dig_parts,
    get_dig_time, get_stopped_segments, get_user_activities, new_activity, reprocess_dig_parts,
    update_activity,
};
pub use activities::{
    __path_create_dig_parts, __path_delete_activity, __path_delete_dig_part,
    __path_download_gpx_file, __path_get_activities_by_date, __path_get_activity,
    __path_get_activity_segments, __path_get_activity_sensor_data, __path_get_activity_track,
    __path_get_dig_segments, __path_get_dig_time, __path_get_stopped_segments,
    __path_get_user_activities, __path_new_activity, __path_preview_activity, __path_update_activity,
    ActivitiesByDateQuery, PreviewActivityResponse, PreviewSportSegment, PreviewTrackPoint,
    TrackBounds, TrackData, TrackPoint, UpdateActivityRequest, UploadQuery, UserActivitiesQuery,
    create_dig_segments, delete_activity, delete_dig_segment, download_gpx_file,
    get_activities_by_date, get_activity, get_activity_segments, get_activity_sensor_data,
    get_activity_track, get_dig_segments, get_dig_time, get_stopped_segments, get_user_activities,
    new_activity, preview_activity, update_activity,
};
pub use activity_types::{
    __path_create_activity_type, __path_get_activity_type, __path_list_activity_types,
    __path_resolve_activity_type, ResolveTypeQuery, ResolveTypeResponse, create_activity_type,
    get_activity_type, list_activity_types, resolve_activity_type,
};
pub use demographics::{
    __path_get_my_demographics, __path_update_my_demographics, get_my_demographics,
    update_my_demographics,
};
pub use leaderboards::{
    __path_get_average_speed_leaderboard, __path_get_countries, __path_get_crown_leaderboard,
    __path_get_dig_percentage_leaderboard, __path_get_dig_time_leaderboard,
    __path_get_distance_leaderboard, GlobalLeaderboardQuery, get_average_speed_leaderboard,
    get_countries, get_crown_leaderboard, get_dig_percentage_leaderboard, get_dig_time_leaderboard,
    get_distance_leaderboard,
};
pub use segments::{
    __path_create_segment, __path_get_filtered_leaderboard, __path_get_leaderboard_position,
    __path_get_my_segment_efforts, __path_get_nearby_segments, __path_get_segment,
    __path_get_segment_leaderboard, __path_get_segment_track, __path_get_starred_segment_efforts,
    __path_get_starred_segments, __path_is_segment_starred, __path_list_segments,
    __path_preview_segment, __path_reprocess_segment, __path_star_segment, __path_unstar_segment,
    ClimbCategoryFilter, CreateSegmentRequest, ListSegmentsQuery, NearbySegmentsQuery,
    PreviewSegmentRequest, PreviewSegmentResponse, ReprocessResult, SegmentPoint, SegmentSortBy,
    SegmentTrackData, SegmentTrackPoint, SegmentValidation, SortOrder, StarResponse,
    create_segment, get_filtered_leaderboard, get_leaderboard_position, get_my_segment_efforts,
    get_nearby_segments, get_segment, get_segment_leaderboard, get_segment_track,
    get_starred_segment_efforts, get_starred_segments, is_segment_starred, list_segments,
    preview_segment, reprocess_segment, star_segment, unstar_segment,
};
pub use social::{
    __path_add_comment, __path_delete_comment, __path_follow_user, __path_get_comments,
    __path_get_feed, __path_get_follow_status, __path_get_followers, __path_get_following,
    __path_get_kudos_givers, __path_get_kudos_status, __path_get_notifications,
    __path_get_user_profile, __path_give_kudos, __path_mark_all_notifications_read,
    __path_mark_notification_read, __path_remove_kudos, __path_unfollow_user, AddCommentRequest,
    FeedQuery, FollowListQuery, FollowListResponse, FollowStatusResponse, KudosResponse,
    KudosStatusResponse, NotificationsQuery, add_comment, delete_comment, follow_user,
    get_comments, get_feed, get_follow_status, get_followers, get_following, get_kudos_givers,
    get_kudos_status, get_notifications, get_user_profile, give_kudos, mark_all_notifications_read,
    mark_notification_read, remove_kudos, unfollow_user,
};
pub use stats::{__path_get_stats, __path_health_check, get_stats, health_check};
pub use teams::{
    __path_accept_invitation, __path_change_member_role, __path_create_team, __path_delete_team,
    __path_discover_teams, __path_get_activity_teams, __path_get_invitation,
    __path_get_join_requests, __path_get_segment_teams, __path_get_team,
    __path_get_team_activities, __path_get_team_activities_by_date, __path_get_team_invitations,
    __path_get_team_leaderboard, __path_get_team_segments, __path_invite_to_team, __path_join_team,
    __path_leave_team, __path_list_my_teams, __path_list_team_members, __path_remove_team_member,
    __path_review_join_request, __path_revoke_invitation, __path_share_activity_with_teams,
    __path_share_segment_with_teams, __path_unshare_activity_from_team,
    __path_unshare_segment_from_team, __path_update_team, DiscoverTeamsQuery,
    ReviewJoinRequestRequest, TeamActivitiesByDateQuery, TeamContentQuery, TeamLeaderboardQuery,
    TeamLeaderboardResponse, accept_invitation, change_member_role, create_team, delete_team,
    discover_teams, get_activity_teams, get_invitation, get_join_requests, get_segment_teams,
    get_team, get_team_activities, get_team_activities_by_date, get_team_invitations,
    get_team_leaderboard, get_team_segments, invite_to_team, join_team, leave_team, list_my_teams,
    list_team_members, remove_team_member, review_join_request, revoke_invitation,
    share_activity_with_teams, share_segment_with_teams, unshare_activity_from_team,
    unshare_segment_from_team, update_team,
};
pub use users::{__path_all_users, __path_new_user, NewUserQuery, all_users, new_user};
