//! Team management handlers.

use axum::{
    Extension,
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    database::Database,
    errors::AppError,
    models::{
        ChangeMemberRoleRequest, CreateTeamRequest, InviteToTeamRequest, JoinTeamRequest, Segment,
        ShareWithTeamsRequest, Team, TeamInvitationWithDetails, TeamMember, TeamRole, TeamSummary,
        TeamWithMembership, UpdateTeamRequest,
    },
};

use super::pagination::default_limit;

// ============================================================================
// ============================================================================

#[utoipa::path(
    post,
    path = "/teams",
    tag = "teams",
    request_body = CreateTeamRequest,
    responses(
        (status = 201, description = "Team created", body = Team),
        (status = 401, description = "Unauthorized")
    )
)]
/// Create a new team.
pub async fn create_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<Team>), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Team name cannot be empty".to_string(),
        ));
    }

    let team = db
        .create_team(
            &req.name,
            req.description.as_deref(),
            req.avatar_url.as_deref(),
            req.visibility,
            req.join_policy,
            claims.sub,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(team)))
}

#[utoipa::path(
    get,
    path = "/teams/{id}",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    responses(
        (status = 200, description = "Team details with membership context", body = TeamWithMembership),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Team not found")
    )
)]
/// Get a team by ID (with membership context for the current user).
pub async fn get_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamWithMembership>, AppError> {
    let team = db
        .get_team_with_membership(id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    // Private teams are only visible to members
    if team.team.visibility == crate::models::TeamVisibility::Private && !team.is_member {
        return Err(AppError::NotFound);
    }

    Ok(Json(team))
}

#[utoipa::path(
    get,
    path = "/teams",
    tag = "teams",
    responses(
        (status = 200, description = "List of teams user is a member of", body = Vec<TeamWithMembership>),
        (status = 401, description = "Unauthorized")
    )
)]
/// List teams the authenticated user is a member of.
pub async fn list_my_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<TeamWithMembership>>, AppError> {
    let teams = db.list_user_teams(claims.sub).await?;
    Ok(Json(teams))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DiscoverTeamsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[utoipa::path(
    get,
    path = "/teams/discover",
    tag = "teams",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of teams to return"),
        ("offset" = Option<i64>, Query, description = "Number of teams to skip")
    ),
    responses(
        (status = 200, description = "List of discoverable teams", body = Vec<TeamSummary>)
    )
)]
/// List discoverable teams.
pub async fn discover_teams(
    Extension(db): Extension<Database>,
    Query(query): Query<DiscoverTeamsQuery>,
) -> Result<Json<Vec<TeamSummary>>, AppError> {
    let teams = db
        .list_discoverable_teams(query.limit, query.offset)
        .await?;
    Ok(Json(teams))
}

#[utoipa::path(
    put,
    path = "/teams/{id}",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    request_body = UpdateTeamRequest,
    responses(
        (status = 200, description = "Team updated", body = Team),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Team not found")
    )
)]
/// Update a team's settings.
pub async fn update_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTeamRequest>,
) -> Result<Json<Team>, AppError> {
    // Check membership and role
    let membership = db
        .get_team_membership(id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_modify_team() {
        return Err(AppError::Forbidden);
    }

    let team = db
        .update_team(
            id,
            req.name.as_deref(),
            req.description.as_deref(),
            req.avatar_url.as_deref(),
            req.visibility,
            req.join_policy,
            req.featured_leaderboard,
        )
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(team))
}

#[utoipa::path(
    delete,
    path = "/teams/{id}",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    responses(
        (status = 204, description = "Team deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Team not found")
    )
)]
/// Delete a team.
pub async fn delete_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check membership and role
    let membership = db
        .get_team_membership(id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_delete_team() {
        return Err(AppError::Forbidden);
    }

    if db.delete_team(id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

// ============================================================================
// ============================================================================

#[utoipa::path(
    get,
    path = "/teams/{id}/members",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    responses(
        (status = 200, description = "List of team members", body = Vec<TeamMember>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Team not found")
    )
)]
/// List members of a team.
pub async fn list_team_members(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<TeamMember>>, AppError> {
    // Check if user is a member (only members can see member list)
    let membership = db.get_team_membership(team_id, claims.sub).await?;
    if membership.is_none() {
        // Check if team exists and is public
        let team = db.get_team(team_id).await?.ok_or(AppError::NotFound)?;
        if team.visibility == crate::models::TeamVisibility::Private {
            return Err(AppError::NotFound);
        }
    }

    let members = db.list_team_members(team_id).await?;
    Ok(Json(members))
}

#[utoipa::path(
    delete,
    path = "/teams/{team_id}/members/{user_id}",
    tag = "teams",
    params(
        ("team_id" = Uuid, Path, description = "Team ID"),
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Team or member not found")
    )
)]
/// Remove a member from a team (admin/owner) or leave team (self).
pub async fn remove_team_member(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let my_membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let is_self = claims.sub == user_id;

    // Self-removal (leaving) is always allowed for non-owners
    if is_self {
        if my_membership.role == TeamRole::Owner {
            return Err(AppError::InvalidInput(
                "Owner cannot leave the team. Transfer ownership first or delete the team."
                    .to_string(),
            ));
        }
    } else {
        // Removing someone else requires admin/owner role
        if !my_membership.role.can_manage_members() {
            return Err(AppError::Forbidden);
        }

        // Check target membership
        let target_membership = db
            .get_team_membership(team_id, user_id)
            .await?
            .ok_or(AppError::NotFound)?;

        // Can't remove someone with equal or higher role
        if target_membership.role == TeamRole::Owner {
            return Err(AppError::InvalidInput(
                "Cannot remove the team owner".to_string(),
            ));
        }
        if target_membership.role == TeamRole::Admin && my_membership.role == TeamRole::Admin {
            return Err(AppError::InvalidInput(
                "Admins cannot remove other admins".to_string(),
            ));
        }
    }

    if db.remove_team_member(team_id, user_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

#[utoipa::path(
    put,
    path = "/teams/{team_id}/members/{user_id}/role",
    tag = "teams",
    params(
        ("team_id" = Uuid, Path, description = "Team ID"),
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = ChangeMemberRoleRequest,
    responses(
        (status = 200, description = "Role changed", body = crate::models::TeamMembership),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Team or member not found")
    )
)]
/// Change a member's role.
pub async fn change_member_role(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ChangeMemberRoleRequest>,
) -> Result<Json<crate::models::TeamMembership>, AppError> {
    let my_membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !my_membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    // Cannot promote to owner via role change
    if req.role == TeamRole::Owner {
        return Err(AppError::InvalidInput(
            "Use transfer ownership instead".to_string(),
        ));
    }

    // Can't demote another owner
    let target_membership = db
        .get_team_membership(team_id, user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if target_membership.role == TeamRole::Owner && claims.sub != user_id {
        return Err(AppError::InvalidInput(
            "Cannot change the owner's role".to_string(),
        ));
    }

    let membership = db
        .change_team_member_role(team_id, user_id, req.role)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(membership))
}

// ============================================================================
// ============================================================================

#[utoipa::path(
    post,
    path = "/teams/{id}/join",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    request_body = JoinTeamRequest,
    responses(
        (status = 201, description = "Joined team directly (open policy)"),
        (status = 202, description = "Join request submitted (request policy)"),
        (status = 400, description = "Already a member or invitation-only team"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Team not found")
    )
)]
/// Request to join a team (for request-based teams) or join directly (for open teams).
pub async fn join_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<JoinTeamRequest>,
) -> Result<StatusCode, AppError> {
    let team = db.get_team(team_id).await?.ok_or(AppError::NotFound)?;

    // Check if already a member
    if db.get_team_membership(team_id, claims.sub).await?.is_some() {
        return Err(AppError::InvalidInput(
            "Already a member of this team".to_string(),
        ));
    }

    match team.join_policy {
        crate::models::TeamJoinPolicy::Open => {
            // Directly add as member
            db.add_team_member(team_id, claims.sub, TeamRole::Member, None)
                .await?;
            Ok(StatusCode::CREATED)
        }
        crate::models::TeamJoinPolicy::Request => {
            // Create a join request
            db.create_team_join_request(team_id, claims.sub, req.message.as_deref())
                .await?;
            Ok(StatusCode::ACCEPTED)
        }
        crate::models::TeamJoinPolicy::Invitation => Err(AppError::InvalidInput(
            "This team is invitation-only".to_string(),
        )),
    }
}

#[utoipa::path(
    post,
    path = "/teams/{id}/leave",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    responses(
        (status = 204, description = "Left team"),
        (status = 400, description = "Owner cannot leave team"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Team not found or not a member")
    )
)]
/// Leave a team.
pub async fn leave_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if membership.role == TeamRole::Owner {
        return Err(AppError::InvalidInput(
            "Owner cannot leave the team. Transfer ownership first or delete the team.".to_string(),
        ));
    }

    if db.remove_team_member(team_id, claims.sub).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

#[utoipa::path(
    get,
    path = "/teams/{id}/join-requests",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    responses(
        (status = 200, description = "List of pending join requests", body = Vec<crate::models::TeamJoinRequestWithUser>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Team not found")
    )
)]
/// Get pending join requests for a team (admin only).
pub async fn get_join_requests(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::TeamJoinRequestWithUser>>, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    let requests = db.get_pending_join_requests(team_id).await?;
    Ok(Json(requests))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewJoinRequestRequest {
    pub approved: bool,
}

#[utoipa::path(
    put,
    path = "/teams/{team_id}/join-requests/{request_id}",
    tag = "teams",
    params(
        ("team_id" = Uuid, Path, description = "Team ID"),
        ("request_id" = Uuid, Path, description = "Join request ID")
    ),
    request_body = ReviewJoinRequestRequest,
    responses(
        (status = 204, description = "Request reviewed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Team or request not found")
    )
)]
/// Approve or reject a join request.
pub async fn review_join_request(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((team_id, request_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ReviewJoinRequestRequest>,
) -> Result<StatusCode, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    // Get the join request
    let join_request = db
        .get_join_request(request_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if join_request.team_id != team_id {
        return Err(AppError::NotFound);
    }

    let result = db
        .review_join_request(request_id, claims.sub, req.approved)
        .await?;

    if let Some(reviewed_request) = result {
        if req.approved {
            // Add the user as a member
            db.add_team_member(
                team_id,
                reviewed_request.user_id,
                TeamRole::Member,
                Some(claims.sub),
            )
            .await?;

            // Notify the user they were accepted
            db.create_notification(
                reviewed_request.user_id,
                "team_join_approved",
                Some(claims.sub),
                Some("team"),
                Some(team_id),
                None,
            )
            .await?;
        } else {
            // Notify the user they were rejected
            db.create_notification(
                reviewed_request.user_id,
                "team_join_rejected",
                Some(claims.sub),
                Some("team"),
                Some(team_id),
                None,
            )
            .await?;
        }
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound)
    }
}

// ============================================================================
// ============================================================================

#[utoipa::path(
    post,
    path = "/teams/{id}/invitations",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    request_body = InviteToTeamRequest,
    responses(
        (status = 201, description = "Invitation created", body = crate::models::TeamInvitation),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Team not found")
    )
)]
/// Create an invitation to join a team.
pub async fn invite_to_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<InviteToTeamRequest>,
) -> Result<(StatusCode, Json<crate::models::TeamInvitation>), AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    // Only owners can invite as admin
    if req.role == TeamRole::Admin && membership.role != TeamRole::Owner {
        return Err(AppError::InvalidInput(
            "Only owners can invite admins".to_string(),
        ));
    }

    // Can't invite as owner
    if req.role == TeamRole::Owner {
        return Err(AppError::InvalidInput("Cannot invite as owner".to_string()));
    }

    // Generate a secure token
    let token = Uuid::new_v4().to_string();

    // Set expiry to 7 days from now
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(7);

    let invitation = db
        .create_team_invitation(
            team_id, &req.email, claims.sub, req.role, &token, expires_at,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(invitation)))
}

#[utoipa::path(
    get,
    path = "/teams/{id}/invitations",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID")
    ),
    responses(
        (status = 200, description = "List of pending invitations", body = Vec<crate::models::TeamInvitation>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Team not found")
    )
)]
/// Get pending invitations for a team (admin only).
pub async fn get_team_invitations(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<crate::models::TeamInvitation>>, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    let invitations = db.get_pending_invitations(team_id).await?;
    Ok(Json(invitations))
}

#[utoipa::path(
    delete,
    path = "/teams/{team_id}/invitations/{invitation_id}",
    tag = "teams",
    params(
        ("team_id" = Uuid, Path, description = "Team ID"),
        ("invitation_id" = Uuid, Path, description = "Invitation ID")
    ),
    responses(
        (status = 204, description = "Invitation revoked"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Invitation not found")
    )
)]
/// Revoke an invitation.
pub async fn revoke_invitation(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((team_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let membership = db
        .get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    if !membership.role.can_manage_members() {
        return Err(AppError::Forbidden);
    }

    if db.revoke_invitation(invitation_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

#[utoipa::path(
    get,
    path = "/invitations/{id}",
    tag = "teams",
    params(
        ("id" = String, Path, description = "Invitation token")
    ),
    responses(
        (status = 200, description = "Invitation details", body = crate::models::TeamInvitationWithDetails),
        (status = 400, description = "Invitation has expired"),
        (status = 404, description = "Invitation not found")
    )
)]
/// Get invitation details by token.
pub async fn get_invitation(
    Extension(db): Extension<Database>,
    Path(token): Path<String>,
) -> Result<Json<TeamInvitationWithDetails>, AppError> {
    let invitation = db
        .get_invitation_by_token(&token)
        .await?
        .ok_or(AppError::NotFound)?;

    // Check if expired
    if invitation.expires_at < time::OffsetDateTime::now_utc() {
        return Err(AppError::InvalidInput("Invitation has expired".to_string()));
    }

    Ok(Json(invitation))
}

#[utoipa::path(
    post,
    path = "/invitations/{id}/accept",
    tag = "teams",
    params(
        ("id" = String, Path, description = "Invitation token")
    ),
    responses(
        (status = 200, description = "Invitation accepted"),
        (status = 400, description = "Invalid invitation or already a member"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invitation not found")
    )
)]
/// Accept an invitation.
pub async fn accept_invitation(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(token): Path<String>,
) -> Result<StatusCode, AppError> {
    // Get invitation details
    let invitation_details = db
        .get_invitation_by_token(&token)
        .await?
        .ok_or(AppError::NotFound)?;

    // Mark invitation as accepted
    let invitation = db
        .accept_invitation(&token)
        .await?
        .ok_or(AppError::InvalidInput(
            "Invitation is invalid or has expired".to_string(),
        ))?;

    // Check if already a member
    if db
        .get_team_membership(invitation.team_id, claims.sub)
        .await?
        .is_some()
    {
        return Err(AppError::InvalidInput(
            "Already a member of this team".to_string(),
        ));
    }

    // Add as member with the invited role
    db.add_team_member(
        invitation.team_id,
        claims.sub,
        invitation.role,
        Some(invitation.invited_by),
    )
    .await?;

    // Notify the inviter
    db.create_notification(
        invitation.invited_by,
        "team_invite_accepted",
        Some(claims.sub),
        Some("team"),
        Some(invitation.team_id),
        Some(&invitation_details.team_name),
    )
    .await?;

    Ok(StatusCode::CREATED)
}

// ============================================================================
// ============================================================================

#[utoipa::path(
    get,
    path = "/activities/{id}/teams",
    tag = "teams",
    params(("id" = Uuid, Path, description = "Activity ID")),
    responses(
        (status = 200, description = "Teams the activity is shared with", body = Vec<TeamSummary>),
        (status = 404, description = "Activity not found"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Get teams an activity is shared with.
pub async fn get_activity_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<Vec<TeamSummary>>, AppError> {
    // Verify activity exists and user has access
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Only owner or team members can see sharing
    let is_owner = activity.user_id == claims.sub;
    let has_team_access = db
        .user_has_activity_team_access(claims.sub, activity_id)
        .await?;

    if !is_owner && !has_team_access && activity.visibility != "public" {
        return Err(AppError::NotFound);
    }

    let teams = db.get_activity_teams(activity_id).await?;
    Ok(Json(teams))
}

#[utoipa::path(
    post,
    path = "/activities/{id}/teams",
    tag = "teams",
    params(("id" = Uuid, Path, description = "Activity ID")),
    request_body = crate::models::ShareWithTeamsRequest,
    responses(
        (status = 200, description = "Activity shared with teams"),
        (status = 404, description = "Activity not found"),
        (status = 403, description = "Forbidden - not the owner"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Share an activity with teams.
pub async fn share_activity_with_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(activity_id): Path<Uuid>,
    Json(req): Json<ShareWithTeamsRequest>,
) -> Result<StatusCode, AppError> {
    // Verify activity exists and user is owner
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if activity.user_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    // Verify user is a member of all target teams
    for team_id in &req.team_ids {
        if db
            .get_team_membership(*team_id, claims.sub)
            .await?
            .is_none()
        {
            return Err(AppError::InvalidInput(format!(
                "You are not a member of team {team_id}"
            )));
        }
    }

    db.share_activity_with_teams(activity_id, &req.team_ids, claims.sub)
        .await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/activities/{activity_id}/teams/{team_id}",
    tag = "teams",
    params(
        ("activity_id" = Uuid, Path, description = "Activity ID"),
        ("team_id" = Uuid, Path, description = "Team ID")
    ),
    responses(
        (status = 204, description = "Activity unshared from team"),
        (status = 404, description = "Activity or sharing not found"),
        (status = 403, description = "Forbidden - not the owner"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Unshare an activity from a team.
pub async fn unshare_activity_from_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((activity_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    // Verify activity exists and user is owner
    let activity = db
        .get_activity(activity_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if activity.user_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    if db.unshare_activity_from_team(activity_id, team_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

// ============================================================================
// ============================================================================

#[utoipa::path(
    get,
    path = "/segments/{id}/teams",
    tag = "teams",
    params(("id" = Uuid, Path, description = "Segment ID")),
    responses(
        (status = 200, description = "Teams the segment is shared with", body = Vec<crate::models::TeamSummary>),
        (status = 404, description = "Segment not found"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Get teams a segment is shared with.
pub async fn get_segment_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(segment_id): Path<Uuid>,
) -> Result<Json<Vec<TeamSummary>>, AppError> {
    // Verify segment exists
    let segment = db
        .get_segment(segment_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Only creator or team members can see sharing
    let is_creator = segment.creator_id == claims.sub;
    let has_team_access = db
        .user_has_segment_team_access(claims.sub, segment_id)
        .await?;

    if !is_creator && !has_team_access && segment.visibility != "public" {
        return Err(AppError::NotFound);
    }

    let teams = db.get_segment_teams(segment_id).await?;
    Ok(Json(teams))
}

#[utoipa::path(
    post,
    path = "/segments/{id}/teams",
    tag = "teams",
    params(("id" = Uuid, Path, description = "Segment ID")),
    request_body = crate::models::ShareWithTeamsRequest,
    responses(
        (status = 200, description = "Segment shared with teams"),
        (status = 404, description = "Segment not found"),
        (status = 403, description = "Forbidden - not the creator"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Share a segment with teams.
pub async fn share_segment_with_teams(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(segment_id): Path<Uuid>,
    Json(req): Json<ShareWithTeamsRequest>,
) -> Result<StatusCode, AppError> {
    // Verify segment exists and user is creator
    let segment = db
        .get_segment(segment_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if segment.creator_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    // Verify user is a member of all target teams
    for team_id in &req.team_ids {
        if db
            .get_team_membership(*team_id, claims.sub)
            .await?
            .is_none()
        {
            return Err(AppError::InvalidInput(format!(
                "You are not a member of team {team_id}"
            )));
        }
    }

    db.share_segment_with_teams(segment_id, &req.team_ids)
        .await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/segments/{segment_id}/teams/{team_id}",
    tag = "teams",
    params(
        ("segment_id" = Uuid, Path, description = "Segment ID"),
        ("team_id" = Uuid, Path, description = "Team ID")
    ),
    responses(
        (status = 204, description = "Segment unshared from team"),
        (status = 404, description = "Segment or sharing not found"),
        (status = 403, description = "Forbidden - not the creator"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Unshare a segment from a team.
pub async fn unshare_segment_from_team(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((segment_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    // Verify segment exists and user is creator
    let segment = db
        .get_segment(segment_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if segment.creator_id != claims.sub {
        return Err(AppError::Forbidden);
    }

    if db.unshare_segment_from_team(segment_id, team_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

// ============================================================================
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct TeamContentQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct TeamActivitiesByDateQuery {
    /// Date to filter activities by (YYYY-MM-DD).
    pub date: time::Date,
    /// Maximum number of results.
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of results to skip.
    #[serde(default)]
    pub offset: i64,
}

#[utoipa::path(
    get,
    path = "/teams/{id}/activities",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results"),
        ("offset" = Option<i64>, Query, description = "Number of results to skip")
    ),
    responses(
        (status = 200, description = "Activities shared with the team", body = Vec<crate::models::FeedActivity>),
        (status = 404, description = "Team not found or not a member"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Get activities shared with a team.
pub async fn get_team_activities(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Query(query): Query<TeamContentQuery>,
) -> Result<Json<Vec<crate::models::FeedActivity>>, AppError> {
    // Verify user is a member
    db.get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let activities = db
        .get_team_activities(team_id, query.limit, query.offset)
        .await?;
    Ok(Json(activities))
}

#[utoipa::path(
    get,
    path = "/teams/{id}/segments",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results"),
        ("offset" = Option<i64>, Query, description = "Number of results to skip")
    ),
    responses(
        (status = 200, description = "Segments shared with the team", body = Vec<crate::models::Segment>),
        (status = 404, description = "Team not found or not a member"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Get segments shared with a team.
pub async fn get_team_segments(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Query(query): Query<TeamContentQuery>,
) -> Result<Json<Vec<Segment>>, AppError> {
    // Verify user is a member
    db.get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let segments = db
        .get_team_segments(team_id, query.limit, query.offset)
        .await?;
    Ok(Json(segments))
}

#[utoipa::path(
    get,
    path = "/teams/{id}/activities/daily",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID"),
        TeamActivitiesByDateQuery
    ),
    responses(
        (status = 200, description = "Team activities for the specified date", body = Vec<crate::models::FeedActivity>),
        (status = 404, description = "Team not found or not a member"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Get activities shared with a team for a specific date.
pub async fn get_team_activities_by_date(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Query(query): Query<TeamActivitiesByDateQuery>,
) -> Result<Json<Vec<crate::models::FeedActivity>>, AppError> {
    // Verify user is a member
    db.get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let activities = db
        .get_team_activities_by_date(team_id, query.date, query.limit, query.offset)
        .await?;
    Ok(Json(activities))
}

// ============================================================================
// Team Leaderboard Handlers
// ============================================================================

/// Query parameters for team leaderboards.
#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct TeamLeaderboardQuery {
    /// Maximum number of entries to return
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of entries to skip
    #[serde(default)]
    pub offset: i64,
    /// Time scope for filtering (default: all_time)
    #[serde(default)]
    pub scope: crate::models::LeaderboardScope,
    /// Gender filter
    #[serde(default)]
    pub gender: crate::models::GenderFilter,
    /// Age group filter
    #[serde(default)]
    pub age_group: crate::models::AgeGroup,
    /// Weight class filter
    #[serde(default)]
    pub weight_class: crate::models::WeightClass,
    /// Country filter (ISO country code)
    pub country: Option<String>,
    /// Filter crowns by activity type (only for crown leaderboard)
    pub activity_type_id: Option<Uuid>,
}

/// Team leaderboard response enum to handle different leaderboard types.
#[derive(Debug, serde::Serialize, ToSchema)]
#[serde(untagged)]
pub enum TeamLeaderboardResponse {
    Crowns(Vec<crate::models::CrownCountEntry>),
    Distance(Vec<crate::models::DistanceLeaderEntry>),
    DigTime(Vec<crate::models::DigTimeLeaderEntry>),
    DigPercentage(Vec<crate::models::DigPercentageLeaderEntry>),
    AverageSpeed(Vec<crate::models::AverageSpeedLeaderEntry>),
}

#[utoipa::path(
    get,
    path = "/teams/{id}/leaderboard/{leaderboard_type}",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID"),
        ("leaderboard_type" = String, Path, description = "Leaderboard type: crowns, distance, dig_time, dig_percentage, average_speed"),
        TeamLeaderboardQuery
    ),
    responses(
        (status = 200, description = "Team leaderboard", body = TeamLeaderboardResponse),
        (status = 400, description = "Invalid leaderboard type"),
        (status = 404, description = "Team not found or not a member"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Get a team-scoped leaderboard.
pub async fn get_team_leaderboard(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path((team_id, leaderboard_type)): Path<(Uuid, String)>,
    Query(query): Query<TeamLeaderboardQuery>,
) -> Result<Json<TeamLeaderboardResponse>, AppError> {
    // Verify user is a member
    db.get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let leaderboard_type: crate::models::LeaderboardType = leaderboard_type
        .parse()
        .map_err(|_| AppError::InvalidInput("Invalid leaderboard type".to_string()))?;

    let response = match leaderboard_type {
        crate::models::LeaderboardType::Crowns => {
            let entries = db
                .get_crown_leaderboard_filtered(
                    query.limit,
                    query.offset,
                    query.scope,
                    query.gender,
                    query.age_group,
                    query.weight_class,
                    query.country.as_deref(),
                    query.activity_type_id,
                    Some(team_id),
                )
                .await?;
            TeamLeaderboardResponse::Crowns(entries)
        }
        crate::models::LeaderboardType::Distance => {
            let entries = db
                .get_distance_leaderboard_filtered(
                    query.limit,
                    query.offset,
                    query.scope,
                    query.gender,
                    query.age_group,
                    query.weight_class,
                    query.country.as_deref(),
                    Some(team_id),
                )
                .await?;
            TeamLeaderboardResponse::Distance(entries)
        }
        crate::models::LeaderboardType::DigTime => {
            let entries = db
                .get_dig_time_leaderboard_filtered(
                    query.limit,
                    query.offset,
                    query.gender,
                    query.age_group,
                    query.weight_class,
                    query.country.as_deref(),
                    Some(team_id),
                )
                .await?;
            TeamLeaderboardResponse::DigTime(entries)
        }
        crate::models::LeaderboardType::DigPercentage => {
            let entries = db
                .get_dig_percentage_leaderboard_filtered(
                    query.limit,
                    query.offset,
                    query.scope,
                    query.gender,
                    query.age_group,
                    query.weight_class,
                    query.country.as_deref(),
                    Some(team_id),
                )
                .await?;
            TeamLeaderboardResponse::DigPercentage(entries)
        }
        crate::models::LeaderboardType::AverageSpeed => {
            let entries = db
                .get_average_speed_leaderboard_filtered(
                    query.limit,
                    query.offset,
                    query.scope,
                    query.gender,
                    query.age_group,
                    query.weight_class,
                    query.country.as_deref(),
                    Some(team_id),
                )
                .await?;
            TeamLeaderboardResponse::AverageSpeed(entries)
        }
    };

    Ok(Json(response))
}

// ============================================================================
// Dig Heatmap Handlers
// ============================================================================

fn default_heatmap_limit() -> i64 {
    10000
}

/// Query parameters for dig heatmap requests.
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct DigHeatmapQuery {
    /// Time range filter: 7, 30, 90 days, or "all" for all time
    #[serde(default)]
    pub days: Option<String>,
    /// Maximum number of points to return
    #[serde(default = "default_heatmap_limit")]
    pub limit: i64,
}

impl DigHeatmapQuery {
    /// Convert days parameter to OffsetDateTime threshold
    fn since(&self) -> Option<time::OffsetDateTime> {
        let days_str = self.days.as_deref().unwrap_or("all");
        let days: Option<i64> = match days_str {
            "7" => Some(7),
            "30" => Some(30),
            "90" => Some(90),
            "all" | "" => None,
            s => s.parse().ok(),
        };

        days.map(|d| time::OffsetDateTime::now_utc() - time::Duration::days(d))
    }
}

#[utoipa::path(
    get,
    path = "/teams/{id}/dig-heatmap",
    tag = "teams",
    params(
        ("id" = Uuid, Path, description = "Team ID"),
        DigHeatmapQuery
    ),
    responses(
        (status = 200, description = "Dig heatmap data for the team", body = crate::models::DigHeatmapResponse),
        (status = 404, description = "Team not found or not a member"),
        (status = 401, description = "Unauthorized")
    )
)]
/// Get dig heatmap data for a team's activities.
/// Returns geographic points where trail maintenance occurred, aggregated by location.
pub async fn get_team_dig_heatmap(
    Extension(db): Extension<Database>,
    AuthUser(claims): AuthUser,
    Path(team_id): Path<Uuid>,
    Query(query): Query<DigHeatmapQuery>,
) -> Result<Json<crate::models::DigHeatmapResponse>, AppError> {
    // Verify user is a member
    db.get_team_membership(team_id, claims.sub)
        .await?
        .ok_or(AppError::NotFound)?;

    let response = db
        .get_dig_heatmap_data(Some(team_id), query.since(), query.limit)
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/dig-heatmap",
    tag = "stats",
    params(DigHeatmapQuery),
    responses(
        (status = 200, description = "Global dig heatmap data", body = crate::models::DigHeatmapResponse)
    )
)]
/// Get global dig heatmap data for all public activities.
/// Returns geographic points where trail maintenance occurred, aggregated by location.
pub async fn get_global_dig_heatmap(
    Extension(db): Extension<Database>,
    Query(query): Query<DigHeatmapQuery>,
) -> Result<Json<crate::models::DigHeatmapResponse>, AppError> {
    let response = db
        .get_dig_heatmap_data(None, query.since(), query.limit)
        .await?;
    Ok(Json(response))
}
