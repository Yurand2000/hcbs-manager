# HCBS-Manager

**HCBS-Manager** is a helper application to manage *cgroups* and *processes* for hierarchical scheduling on real-time systems, using the upcoming [HCBS](https://github.com/Yurand2000/HCBS-patch) patchset.

It provides a [FUSE](https://github.com/libfuse/libfuse/)-based filesystem interface to manage cgroups for hierarchical real-time scheduling and to control running processes' scheduling policy and assigned cgroups.

It is not (yet) a general purpose application as its only purpose is to manage interactions with the hierarchical scheduling framework. As an example, applications may ask the manager to be migrated into a *managed* cgroup and then their scheduling policy set to SCHED_FIFO, but it is disallowed to do this in the opposite order. The manager only changes the scheduling policy of a process inside of a *managed* cgroup.
*Managed* cgroups are those created specifically by the manager application on user request. While on the same machine there may exist other cgroups that work using the HCBS framework, they cannot be interacted with if they are not directly managed by the **HCBS-Manager**.

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