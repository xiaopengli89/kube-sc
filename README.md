# kube-sc

A simple template generator for kubernetes local storage class and persistent volume.

## Installation

```bash
$ cargo install --git https://github.com/xiaopengli89/kube-sc
```

## Usage

Describe your local storage class and persistent volume in a yaml file, such as `app.sc.yaml`:

```yaml
storageClassName: app
nodes:
- host: node0
  pvs:
  - name: "0"
    path: /data/0
    capacity: 10Gi
  - name: "1"
    path: /data/1
    capacity: 20Gi
```

Generate the template file and create the resources by `kubectl`:

```bash
$ kube-sc -f app.sc.yaml | kubectl apply -f -
storageclass.storage.k8s.io/app created
persistentvolume/app-0 created
persistentvolume/app-1 created

$ kubectl get sc,pv
NAME                              PROVISIONER                    RECLAIMPOLICY   VOLUMEBINDINGMODE      ALLOWVOLUMEEXPANSION   AGE
storageclass.storage.k8s.io/app   kubernetes.io/no-provisioner   Delete          WaitForFirstConsumer   false                  24s

NAME                     CAPACITY   ACCESS MODES   RECLAIM POLICY   STATUS      CLAIM   STORAGECLASS   REASON   AGE
persistentvolume/app-0   10Gi       RWO            Retain           Available           app                     24s
persistentvolume/app-1   20Gi       RWO            Retain           Available           app                     24s
```

Or you can just print the template file:

```bash
kube-sc -f app.sc.yaml
```

The output:

```
---
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: app
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: WaitForFirstConsumer
---
apiVersion: v1
kind: PersistentVolume
metadata:
  name: app-0
spec:
  accessModes:
    - ReadWriteOnce
  capacity:
    storage: 10Gi
  local:
    path: /data/0
  nodeAffinity:
    required:
      nodeSelectorTerms:
        - matchExpressions:
            - key: kubernetes.io/hostname
              operator: In
              values:
                - node0
  persistentVolumeReclaimPolicy: Retain
  storageClassName: app
  volumeMode: Filesystem
---
apiVersion: v1
kind: PersistentVolume
metadata:
  name: app-1
spec:
  accessModes:
    - ReadWriteOnce
  capacity:
    storage: 20Gi
  local:
    path: /data/1
  nodeAffinity:
    required:
      nodeSelectorTerms:
        - matchExpressions:
            - key: kubernetes.io/hostname
              operator: In
              values:
                - node0
  persistentVolumeReclaimPolicy: Retain
  storageClassName: app
  volumeMode: Filesystem
```

Alternatively, to save the template as a file:

```bash
$ kube-sc -f app.sc.yaml > output.yaml
```

## License

MIT Licensed.
