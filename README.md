# kube-sc

A persistent volume generator for kubernetes local storage class

## Installation

```bash
$ cargo install --git https://github.com/xiaopengli89/kube-sc
```

## Usage

Describe your local storage class and persistent volume in a yaml file, such as `app.sc.yaml`:

```yaml
storageClassName: app
jobNamespace: kube-sc
jobName: kube-sc-helper
jobImage: busybox:1.31.1
nodes:
  - host: k3d-k3s-default-server
    rootPath: /pv-data/0
  - selector: node.kubernetes.io/instance-type=k3s
    rootPath: /pv-data/1
count: 3
capacity: 20Gi
```

Create 3 pvs on nodes

```bash
$ kube-sc -f app.sc.yaml
StorageClass [app] applied
Namespace [kube-sc] applied
PersistentVolume [app-n6fo32jh] created
PersistentVolume [app-w50wkvhu] created
PersistentVolume [app-gakmqv2f] created

$ kubectl get sc,pv
NAME                                               PROVISIONER                    RECLAIMPOLICY   VOLUMEBINDINGMODE      ALLOWVOLUMEEXPANSION   AGE
storageclass.storage.k8s.io/local-path (default)   rancher.io/local-path          Delete          WaitForFirstConsumer   false                  32h
storageclass.storage.k8s.io/app                    kubernetes.io/no-provisioner   Delete          WaitForFirstConsumer   false                  26m

NAME                            CAPACITY   ACCESS MODES   RECLAIM POLICY   STATUS      CLAIM   STORAGECLASS   REASON   AGE
persistentvolume/app-n6fo32jh   20Gi       RWO            Retain           Available           app                     61s
persistentvolume/app-w50wkvhu   20Gi       RWO            Retain           Available           app                     61s
persistentvolume/app-gakmqv2f   20Gi       RWO            Retain           Available           app                     61s

$ /pv-data # ls -l *
0:
total 8
drwxrwxrwx 2 0 0 4096 Jun 12 15:53 app-n6fo32jh
drwxrwxrwx 2 0 0 4096 Jun 12 15:53 app-w50wkvhu

1:
total 4
drwxrwxrwx 2 0 0 4096 Jun 12 15:53 app-gakmqv2f
```

## License

MIT Licensed.
