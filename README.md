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
	--set kufefe.expireMinutes=60
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

For safety reasons, you can't bind to just any role, however. Kufefe will only create SA's for roles that have the annotation `kufefe.io/role: "true"`. This is to prevent binding to very privileged roles on accident.

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
