package coreauth

// CreateGroupRequest represents a request to create a group.
type CreateGroupRequest struct {
	Name        string  `json:"name"`
	Description *string `json:"description,omitempty"`
	ExternalID  *string `json:"external_id,omitempty"`
}

// Group represents a group within an organization.
type Group struct {
	ID             string  `json:"id"`
	OrganizationID *string `json:"organization_id,omitempty"`
	Name           string  `json:"name"`
	Description    *string `json:"description,omitempty"`
	ExternalID     *string `json:"external_id,omitempty"`
	MemberCount    *int    `json:"member_count,omitempty"`
	CreatedAt      *string `json:"created_at,omitempty"`
	UpdatedAt      *string `json:"updated_at,omitempty"`
}

// UpdateGroupRequest represents a request to update a group.
type UpdateGroupRequest struct {
	Name        *string `json:"name,omitempty"`
	Description *string `json:"description,omitempty"`
}

// AddGroupMemberRequest represents a request to add a member to a group.
type AddGroupMemberRequest struct {
	UserID string  `json:"user_id"`
	Role   *string `json:"role,omitempty"`
}

// GroupMember represents a member of a group.
type GroupMember struct {
	ID       string  `json:"id"`
	UserID   string  `json:"user_id"`
	GroupID  string  `json:"group_id"`
	Role     *string `json:"role,omitempty"`
	Email    *string `json:"email,omitempty"`
	FullName *string `json:"full_name,omitempty"`
	JoinedAt *string `json:"joined_at,omitempty"`
}

// UpdateGroupMemberRequest represents a request to update a group member's role.
type UpdateGroupMemberRequest struct {
	Role string `json:"role"`
}

// GroupRole represents a role assigned to a group.
type GroupRole struct {
	RoleID    string  `json:"role_id"`
	GroupID   string  `json:"group_id"`
	CreatedAt *string `json:"created_at,omitempty"`
}

// AssignGroupRoleRequest represents a request to assign a role to a group.
type AssignGroupRoleRequest struct {
	RoleID string `json:"role_id"`
}

// CreateInvitationRequest represents a request to create a user invitation.
type CreateInvitationRequest struct {
	Email         string         `json:"email"`
	RoleID        *string        `json:"role_id,omitempty"`
	Metadata      map[string]any `json:"metadata,omitempty"`
	ExpiresInDays *int           `json:"expires_in_days,omitempty"`
}

// InvitationResponse represents an invitation.
type InvitationResponse struct {
	ID        string  `json:"id"`
	TenantID  string  `json:"tenant_id"`
	Email     string  `json:"email"`
	RoleID    *string `json:"role_id,omitempty"`
	Status    string  `json:"status"`
	InvitedBy *string `json:"invited_by,omitempty"`
	ExpiresAt *string `json:"expires_at,omitempty"`
	CreatedAt *string `json:"created_at,omitempty"`
}

// AcceptInvitationRequest represents a request to accept an invitation.
type AcceptInvitationRequest struct {
	Token    string         `json:"token"`
	Password string         `json:"password"`
	FullName string         `json:"full_name"`
	Metadata map[string]any `json:"metadata,omitempty"`
}
