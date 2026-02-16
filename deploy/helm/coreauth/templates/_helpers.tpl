{{/*
Expand the name of the chart.
*/}}
{{- define "coreauth.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "coreauth.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version for chart label.
*/}}
{{- define "coreauth.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels.
*/}}
{{- define "coreauth.labels" -}}
helm.sh/chart: {{ include "coreauth.chart" . }}
{{ include "coreauth.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels.
*/}}
{{- define "coreauth.selectorLabels" -}}
app.kubernetes.io/name: {{ include "coreauth.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Core API fully qualified name.
*/}}
{{- define "coreauth.core.fullname" -}}
{{- printf "%s-core" (include "coreauth.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Dashboard fully qualified name.
*/}}
{{- define "coreauth.dashboard.fullname" -}}
{{- printf "%s-dashboard" (include "coreauth.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Service account name.
*/}}
{{- define "coreauth.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "coreauth.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Database URL — built from Bitnami subchart or external config.
*/}}
{{- define "coreauth.databaseUrl" -}}
{{- if .Values.postgresql.enabled -}}
postgresql://{{ .Values.postgresql.auth.username }}:{{ .Values.postgresql.auth.password }}@{{ include "coreauth.fullname" . }}-postgresql:5432/{{ .Values.postgresql.auth.database }}
{{- else -}}
postgresql://{{ .Values.externalDatabase.user }}:$(DB_PASSWORD)@{{ .Values.externalDatabase.host }}:{{ .Values.externalDatabase.port }}/{{ .Values.externalDatabase.database }}
{{- end -}}
{{- end }}

{{/*
PostgreSQL host for migration entrypoint.
*/}}
{{- define "coreauth.postgresHost" -}}
{{- if .Values.postgresql.enabled -}}
{{ include "coreauth.fullname" . }}-postgresql
{{- else -}}
{{ .Values.externalDatabase.host }}
{{- end -}}
{{- end }}

{{/*
PostgreSQL password — resolves from subchart or external.
*/}}
{{- define "coreauth.postgresPassword" -}}
{{- if .Values.postgresql.enabled -}}
{{ .Values.postgresql.auth.password }}
{{- else -}}
{{ .Values.externalDatabase.password }}
{{- end -}}
{{- end }}

{{/*
Redis URL — built from Bitnami subchart or external config.
*/}}
{{- define "coreauth.redisUrl" -}}
{{- if .Values.redis.enabled -}}
redis://{{ include "coreauth.fullname" . }}-redis-master:6379
{{- else -}}
{{ .Values.externalRedis.url }}
{{- end -}}
{{- end }}

{{/*
Secret name for chart-managed secrets.
*/}}
{{- define "coreauth.secretName" -}}
{{- include "coreauth.fullname" . }}
{{- end }}

{{/*
Resolve JWT secret name — existingSecret or chart-managed.
*/}}
{{- define "coreauth.jwtSecretName" -}}
{{- if .Values.config.jwt.existingSecret -}}
{{ .Values.config.jwt.existingSecret }}
{{- else -}}
{{ include "coreauth.secretName" . }}
{{- end -}}
{{- end }}

{{/*
Resolve DB secret name — existingSecret or chart-managed.
*/}}
{{- define "coreauth.dbSecretName" -}}
{{- if .Values.externalDatabase.existingSecret -}}
{{ .Values.externalDatabase.existingSecret }}
{{- else -}}
{{ include "coreauth.secretName" . }}
{{- end -}}
{{- end }}

{{/*
Resolve Redis secret name — existingSecret or chart-managed.
*/}}
{{- define "coreauth.redisSecretName" -}}
{{- if .Values.externalRedis.existingSecret -}}
{{ .Values.externalRedis.existingSecret }}
{{- else -}}
{{ include "coreauth.secretName" . }}
{{- end -}}
{{- end }}

{{/*
Core image reference.
*/}}
{{- define "coreauth.core.image" -}}
{{- printf "%s:%s" .Values.core.image.repository (default .Chart.AppVersion .Values.core.image.tag) }}
{{- end }}

{{/*
Dashboard image reference.
*/}}
{{- define "coreauth.dashboard.image" -}}
{{- printf "%s:%s" .Values.dashboard.image.repository (default .Chart.AppVersion .Values.dashboard.image.tag) }}
{{- end }}
