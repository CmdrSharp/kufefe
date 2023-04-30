# Kufefe - Ephemeral Kubeconfig Generator

Kufefe lets you generate kubeconfig files that are;

* Ephemeral - because giving out the cluster kubeconfig is dangerous as it never expires
* Scoped - allowing you to limit access to only what is required

It runs inside your cluster and scans for new cluster-scoped `Request`s. Related resources (the `ServiceAccount`, `Secret` and `ClusterRoleBinding`) are automatically deleted through their `ownerReference` to the Request. For resources that are namespaced, they will always be created in the deployment namespace.

Kufefe currently supports Kubernetes version 1.25 and up.

## Installation

Installation is done via Helm. Only a few values need to be set. Kufefe does not expose a service (and does not need to) - it runs as a single pod and consumes very little in terms of resources.

```
helm repo add kufefe https://cmdrsharp.github.io/kufefe

helm install kufefe kufefe/kufefe \
  --set kufefe.clusterUrl="https://my-cluster-api-url:443/"
```

Make sure to point the `clusterUrl` at the API address of the Kubernetes cluster. `expireMinutes` defaults to 60, but can be set to any number of minutes.

## Request CRD

The Request CRD a simple cluster-scoped resource where you specify which `ClusterRole` you want the request to be tied to. A request will look something like this:

```yaml
apiVersion: "kufefe.io/v1"
kind: Request
metadata:
  name: i-need-a-kubeconfig
spec:
  role: my-cluster-role # See note below on roles!
```

Shortly upon creating this `Request`, you should see that it is marked `Ready`.

```
❯ kubectl get req
NAME                  READY   AGE
i-need-a-kubeconfig   true    2s
```

You can now get the kubeconfig:
```
❯ kubectl get req i-need-a-kubeconfig -o=jsonpath='{.status.kubeconfig}'
```

### Privilege Escalation & Role Aggregation

Kufefe's own RBAC is set up using [aggregated cluster roles](https://kubernetes.io/docs/reference/access-authn-authz/rbac/#aggregated-clusterroles) with the label `rbac.authorization.k8s.io/aggregate-kufefe: "true"`.

In order for Kufefe to be allowed to create Service Accounts that bind to other roles, there are two prerequisites:

1. The role must be annotated with `kufefe.io/role: "true"`. If it is not, Kufefe will not create an SA bound to that role.
2. Kufefe itself must be allowed to bind to the role it is trying to create an SA for. This a [Kubernetes mechanism](https://kubernetes.io/docs/reference/access-authn-authz/rbac/#privilege-escalation-prevention-and-bootstrapping) to prevent privilege escalation.

For scenario two, let's say you have a `ClusterRole` called `debug`. In order for Kufefe to be allowed to create SA's bound to this role, you would create a `ClusterRole` like this:

```yaml
---
kind: ClusterRole
apiVersion: rbac.authorization.k8s.io/v1
metadata:
  name: kufefe:rbac:bind:debug
  labels:
    rbac.authorization.k8s.io/aggregate-kufefe: "true"
rules:
- apiGroups: ["rbac.authorization.k8s.io"]
  resources: ["clusterroles"]
  verbs: ["bind"]
  resourceNames: ["debug"] # Leave out to allow binding to ANY ClusterRole. Not recommended.
```

With this role created, Kufefe will now be allowed to create the ServiceAccount tied to the `debug` role.

