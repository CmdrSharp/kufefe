{{- define "kufefe.rbac.label" -}}
rbac.authorization.k8s.io/aggregate-kufefe: "true"
{{- end -}}

{{- define "kufefe.rbac.roleName" -}}
{{ .Release.Name }}:rbac
{{- end -}}
