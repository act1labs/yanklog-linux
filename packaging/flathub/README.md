# Flathub Manifest Reference

YankLog is published on Flathub from
[`flathub/com.yanklog.YankLog`](https://github.com/flathub/com.yanklog.YankLog). That
repository is the source of truth. The two files here are reference copies of it:

```text
com.yanklog.YankLog.yml
generated-sources.json
```

The manifest builds from an immutable public Git tag and its exact commit, never a
moving branch such as `master`. Flathub's external data checker updates `tag` and
`commit` in the Flathub repository for each new release.

Regenerate `generated-sources.json` after every `Cargo.lock` change, copy it here, and
push it onto the checker's pull request in the Flathub repository. See
[Releasing To Flathub](../../PACKAGING_LINUX.md#releasing-to-flathub).
