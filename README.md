# HCBS-Manager

**HCBS-Manager** is a helper application to manage *cgroups* and *processes* for hierarchical scheduling on real-time systems, using the upcoming [HCBS](https://github.com/Yurand2000/HCBS-patch) patchset.

It provides a [FUSE](https://github.com/libfuse/libfuse/)-based filesystem interface to manage cgroups for hierarchical real-time scheduling and to control running processes' scheduling policy and assigned cgroups.

It is not (yet) a general purpose application as its only purpose is to manage interactions with the hierarchical scheduling framework. As an example, applications may ask the manager to be migrated into a *managed* cgroup and then their scheduling policy set to SCHED_FIFO, but it is disallowed to do this in the opposite order. The manager only changes the scheduling policy of a process inside of a *managed* cgroup.
*Managed* cgroups are those created specifically by the manager application on user request. While on the same machine there may exist other cgroups that work using the HCBS framework, they cannot be interacted with if they are not directly managed by the **HCBS-Manager**.

## 🚀 Quick Start

### Setup

Build the code
```bash
> cargo build --release
```

Start the manager software (needs root/sudo)

```bash
> sudo ./target/release/hcbs-manager
```

For help (doesn't need sudo)

```bash
./target/release/hcbs-manager --help
```

### Interface

The manager software will setup the machine to run real-time workloads and expose a file based interface to manage cgroups and processes. Standard processes can communicate with the manager by reading and writing to the exposed files. The main mount point for the filesystem is `/mnt/hcbs-manager/`.
The folder `cgroup` contains three files used to manage the cgroups:
- `cgroup/create`, which accepts a string of format `<cgroup name> <runtime us> <period us>`.
- `cgroup/update`, which accepts a string of format `<cgroup name> <runtime us> <period us>`.
- `cgroup/delete`, which accepts a string of format `<cgroup name>`.

The folder `proc` contains a sub-directory for each alive process in the system, the directories are named using the process identifiers. As an example, if the system has a process of PID 128, the filesystem will contain the directory `proc/128`. Each *PID* directory contains two files:
- `proc/<PID>/cgroup`, which accepts a cgroup name, and assigns the process with PID `<PID>` to the input cgroup.
- `proc/<PID>/sched_policy`, which accepts `SCHED_OTHER`, `SCHED_FIFO(<prio>)` or `SCHED_RR(<prio>)`, and sets the given scheduling policy to the process `<PID>`.

Note that cgroup migration is allowed only to groups created using the manager's interface. Additionally, it is currently enforced that only `SCHED_OTHER` processes can migrate. The scheduling policies `SCHED_FIFO/SCHED_RR` can only be set to processes that are assigned to *managed* cgroups.

### Example

Suppose the manager is running. Let's create a cgroup of name `my_cgroup` which requires a runtime of 10ms every 100ms:

```bash
echo "my_cgroup 10000 100000" > /mnt/hcbs-manager/cgroup/create
```

Let's now have a process of PID 1276 running, let's assign it to `my_cgroup` and change its scheduling policy to `SCHED_FIFO` with priority level 50.

```bash
echo "my_cgroup" > /mnt/hcbs-manager/proc/1276/cgroup
echo "SCHED_FIFO(50)" > /mnt/hcbs-manager/proc/1276/sched_policy
```

After some time we want to migrate the process back to the root control group.

```bash
# necessary to switch back to SCHED_OTHER
echo "SCHED_OTHER" > /mnt/hcbs-manager/proc/1276/sched_policy
echo "." > /mnt/hcbs-manager/proc/1276/cgroup
```

We can finally delete the cgroup to free resources.

```bash
echo "my_cgroup" > /mnt/hcbs-manager/cgroup/delete
```

## 🛠️ Future Work

- [ ] User/Group ID based cgroup creation/deletion
- [ ] HCBS multi-runtime support
- [ ] Documentation
- [ ] [📦 crates.io](https://crates.io) release

## 📄 License

This project is licensed under the GNU General Public License v3 - see the [LICENSE](LICENSE) file for details.

## 👤 Author

This software was developed by:
- **Yuri Andriaccio** [yurand2000@gmail.com](mailto:yurand2000@gmail.com), [GitHub](https://github.com/Yurand2000).

---

**HCBS-Manager**