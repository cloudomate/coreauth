export interface CreateGroupRequest {
  name: string;
  description?: string;
  external_id?: string;
}

export interface Group {
  id: string;
  organization_id?: string;
  name: string;
  description?: string;
  external_id?: string;
  member_count?: number;
  created_at?: string;
  updated_at?: string;
}

export interface UpdateGroupRequest {
  name?: string;
  description?: string;
}

export interface AddGroupMemberRequest {
  user_id: string;
  role?: string;
}

export interface GroupMember {
  id: string;
  user_id: string;
  group_id: string;
  role?: string;
  email?: string;
  full_name?: string;
  joined_at?: string;
}

export interface UpdateGroupMemberRequest {
  role: string;
}

export interface GroupRole {
  role_id: string;
  group_id: string;
  created_at?: string;
}

export interface CreateInvitationRequest {
  email: string;
  role_id?: string;
  metadata?: Record<string, any>;
  expires_in_days?: number;
}

export interface InvitationResponse {
  id: string;
  tenant_id: string;
  email: string;
  role_id?: string;
  status: string;
  invited_by?: string;
  expires_at?: string;
  created_at?: string;
}

export interface AcceptInvitationRequest {
  token: string;
  password: string;
  full_name: string;
  metadata?: Record<string, any>;
}
