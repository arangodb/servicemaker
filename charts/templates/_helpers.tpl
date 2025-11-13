{{/* vim: set filetype=mustache: */}}

{{/*
Expand the name of the chart.
*/}}
{{- define "template.name" -}}
{{- lower (printf "%s" .Chart.Name | trunc 63 | trimSuffix "-") -}}
{{- end -}}

{{/*
Expand the name of the release.
*/}}
{{- define "template.releaseName" -}}
{{- lower (printf "%s" .Release.Name | trunc 63 | trimSuffix "-") -}}
{{- end -}}

{{/*
Get the checksum of deployment spec
*/}}
{{- define "release.id" -}}
{{- printf "%s-%s" .Release.Namespace .Release.Name  | sha256sum | trunc 7 -}}
{{- end -}}
