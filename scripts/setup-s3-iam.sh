#!/bin/bash
# Set up IAM role + policy for EC2 → S3 access (Track Leader uploads)
#
# Idempotent: safe to re-run. Skips resources that already exist.
#
# What it does:
#   1. Creates an S3 access policy scoped to the uploads bucket/prefix
#   2. If the EC2 instance already has a role (e.g. for SSM), attaches the policy to it
#   3. If no role exists, creates one with an instance profile and associates it
#
# Prerequisites:
#   - AWS CLI configured with permissions to manage IAM + EC2
#   - Instance ID of your EC2 host
#
# Usage:
#   ./scripts/setup-s3-iam.sh <instance-id>
#   ./scripts/setup-s3-iam.sh i-0abc123def456

set -euo pipefail

# ── Configuration ────────────────────────────────────────────────────────────
BUCKET="tracks.rs-663959447334-us-west-1-an"
PREFIX="uploads"
POLICY_NAME="track-leader-s3-access"
ROLE_NAME="track-leader-ec2"
PROFILE_NAME="track-leader-ec2"
REGION="us-west-1"
# ─────────────────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC}  $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

if [ $# -lt 1 ]; then
    log_error "Usage: $0 <ec2-instance-id>"
    exit 1
fi

INSTANCE_ID="$1"

# Validate instance exists
log_info "Verifying instance ${INSTANCE_ID}..."
if ! aws ec2 describe-instances --instance-ids "$INSTANCE_ID" --region "$REGION" > /dev/null 2>&1; then
    log_error "Instance ${INSTANCE_ID} not found in ${REGION}"
    exit 1
fi

ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
POLICY_ARN="arn:aws:iam::${ACCOUNT_ID}:policy/${POLICY_NAME}"

# ── Step 1: Create S3 policy ────────────────────────────────────────────────
log_info "Step 1: S3 access policy..."

POLICY_DOC=$(cat <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowUploadObjects",
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject"
      ],
      "Resource": "arn:aws:s3:::${BUCKET}/${PREFIX}/*"
    },
    {
      "Sid": "AllowListUploads",
      "Effect": "Allow",
      "Action": "s3:ListBucket",
      "Resource": "arn:aws:s3:::${BUCKET}",
      "Condition": {
        "StringLike": { "s3:prefix": "${PREFIX}/*" }
      }
    }
  ]
}
EOF
)

if aws iam get-policy --policy-arn "$POLICY_ARN" > /dev/null 2>&1; then
    log_info "Policy ${POLICY_NAME} already exists, updating..."
    # Create a new policy version and set it as default
    VERSIONS=$(aws iam list-policy-versions --policy-arn "$POLICY_ARN" --query 'Versions[?!IsDefaultVersion].VersionId' --output text)
    # IAM allows max 5 versions — delete the oldest non-default if at the limit
    VERSION_COUNT=$(aws iam list-policy-versions --policy-arn "$POLICY_ARN" --query 'length(Versions)' --output text)
    if [ "$VERSION_COUNT" -ge 5 ]; then
        OLDEST=$(echo "$VERSIONS" | tr '\t' '\n' | tail -1)
        aws iam delete-policy-version --policy-arn "$POLICY_ARN" --version-id "$OLDEST"
    fi
    aws iam create-policy-version --policy-arn "$POLICY_ARN" \
        --policy-document "$POLICY_DOC" --set-as-default > /dev/null
else
    aws iam create-policy --policy-name "$POLICY_NAME" \
        --policy-document "$POLICY_DOC" \
        --description "Track Leader: S3 access for activity file uploads" > /dev/null
    log_info "Created policy ${POLICY_NAME}"
fi

# ── Step 2: Check if instance already has a role ────────────────────────────
log_info "Step 2: Instance role..."

EXISTING_PROFILE=$(aws ec2 describe-iam-instance-profile-associations \
    --filters "Name=instance-id,Values=${INSTANCE_ID}" "Name=state,Values=associated" \
    --region "$REGION" \
    --query 'IamInstanceProfileAssociations[0].IamInstanceProfile.Arn' \
    --output text 2>/dev/null || echo "None")

if [ "$EXISTING_PROFILE" != "None" ] && [ -n "$EXISTING_PROFILE" ]; then
    # Instance already has a role — extract its name and attach our policy
    EXISTING_PROFILE_NAME=$(echo "$EXISTING_PROFILE" | sed 's|.*/||')
    EXISTING_ROLE=$(aws iam get-instance-profile \
        --instance-profile-name "$EXISTING_PROFILE_NAME" \
        --query 'InstanceProfile.Roles[0].RoleName' --output text)

    log_info "Instance already has role: ${EXISTING_ROLE}"
    log_info "Attaching S3 policy to existing role..."

    if aws iam list-attached-role-policies --role-name "$EXISTING_ROLE" \
        --query "AttachedPolicies[?PolicyArn=='${POLICY_ARN}'].PolicyName" \
        --output text | grep -q "$POLICY_NAME"; then
        log_info "Policy already attached to ${EXISTING_ROLE}"
    else
        aws iam attach-role-policy --role-name "$EXISTING_ROLE" --policy-arn "$POLICY_ARN"
        log_info "Attached ${POLICY_NAME} to ${EXISTING_ROLE}"
    fi
else
    # No role yet — create role + instance profile + associate
    log_info "No existing role found. Creating ${ROLE_NAME}..."

    TRUST_POLICY=$(cat <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": { "Service": "ec2.amazonaws.com" },
      "Action": "sts:AssumeRole"
    }
  ]
}
EOF
)

    if ! aws iam get-role --role-name "$ROLE_NAME" > /dev/null 2>&1; then
        aws iam create-role --role-name "$ROLE_NAME" \
            --assume-role-policy-document "$TRUST_POLICY" \
            --description "Track Leader EC2 backend role" > /dev/null
        log_info "Created role ${ROLE_NAME}"
    else
        log_info "Role ${ROLE_NAME} already exists"
    fi

    # Attach S3 policy
    aws iam attach-role-policy --role-name "$ROLE_NAME" --policy-arn "$POLICY_ARN"
    log_info "Attached ${POLICY_NAME} to ${ROLE_NAME}"

    # Create instance profile
    if ! aws iam get-instance-profile --instance-profile-name "$PROFILE_NAME" > /dev/null 2>&1; then
        aws iam create-instance-profile --instance-profile-name "$PROFILE_NAME" > /dev/null
        aws iam add-role-to-instance-profile \
            --instance-profile-name "$PROFILE_NAME" \
            --role-name "$ROLE_NAME"
        log_info "Created instance profile ${PROFILE_NAME}"
        # IAM is eventually consistent — wait for the profile to propagate
        log_info "Waiting for IAM propagation..."
        sleep 10
    else
        log_info "Instance profile ${PROFILE_NAME} already exists"
    fi

    # Associate with instance
    log_info "Associating instance profile with ${INSTANCE_ID}..."
    aws ec2 associate-iam-instance-profile \
        --instance-id "$INSTANCE_ID" \
        --iam-instance-profile Name="$PROFILE_NAME" \
        --region "$REGION"
    log_info "Associated ${PROFILE_NAME} with ${INSTANCE_ID}"
fi

# ── Done ────────────────────────────────────────────────────────────────────
echo ""
log_info "IAM setup complete."
echo ""
echo "  Policy:   ${POLICY_ARN}"
echo "  Bucket:   s3://${BUCKET}/${PREFIX}/"
echo "  Instance: ${INSTANCE_ID}"
echo ""
echo "The backend container will pick up the role automatically via"
echo "the instance metadata service — no access keys needed."
echo ""
echo "You can remove AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY"
echo "from .env.production if they are set."
