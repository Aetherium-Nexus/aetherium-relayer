{{/*
Expand the name of the chart.
*/}}
{{- define "agent-common.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars - 11 because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "agent-common.fullname" -}}
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
Create chart name and version as used by the chart label.
*/}}
{{- define "agent-common.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "agent-common.labels" -}}
helm.sh/chart: {{ include "agent-common.chart" . }}
app.kubernetes.io/component: agent-common
Aetherium/deployment: {{ .Values.Aetherium.runEnv | quote }}
Aetherium/context: {{ .Values.Aetherium.context | quote }}
{{ include "agent-common.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "agent-common.selectorLabels" -}}
app.kubernetes.io/name: {{ include "agent-common.name" . | trunc 63 | trimSuffix "-"}}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "agent-common.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "agent-common.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
The name of the ClusterSecretStore/SecretStore
*/}}
{{- define "agent-common.secret-store.name" -}}
{{- default "external-secrets-gcp-cluster-secret-store" .Values.externalSecrets.storeName }}
{{- end }}

{{/*
Recursively converts a config object into environment variables than can
be parsed by rust. For example, a config of { foo: { bar: { baz: 420 }, booGo: 421 } } will
be: AET_FOO_BAR_BAZ=420 and AET_FOO_BOOGO=421
Env vars can be formatted in FOO="BAR" format if .format is "dot_env",
FOO: "BAR" format if .format is "config_map", or otherwise
they will be formatted as spec YAML-friendly environment variables
*/}}
{{- define "agent-common.config-env-vars" -}}
{{- range $key_or_idx, $value := .config }}
{{- $key_name := printf "%s%v" (default "" $.key_name_prefix) $key_or_idx }}
{{- if or (typeIs "map[string]interface {}" $value) (typeIs "[]interface {}" $value) }}
{{- include "agent-common.config-env-vars" (dict "config" $value "format" $.format "key_name_prefix" (printf "%s_" $key_name)) }}
{{- else }}
{{- include "agent-common.config-env-var" (dict "key" $key_name "value" $value "format" $.format ) }}
{{- end }}
{{- end }}
{{- end }}

{{- define "agent-common.config-env-var" }}
{{- if (eq .format "dot_env") }}
AET_{{ .key | upper }}={{ .value | quote }}
{{- else if (eq .format "config_map") }}
AET_{{ .key | upper }}: {{ .value | quote }}
{{- else }}
- name: AET_{{ .key | upper }}
  value: {{ .value | quote }}
{{- end }}
{{- end }}

