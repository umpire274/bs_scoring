# Git Release Instructions

This document describes the process to cut a new release of `bs_scoring`.
Example commands use the current working version, **v0.11.0-alpha2**.

For releases cut at a different version, simply substitute `v0.11.0-alpha2`
everywhere.

## Step-by-Step Release Process

### 1. Review Changes

```bash
git status
git diff
```

### 2. Stage All Changes

```bash
git add .
```

### 3. Commit with Message

```bash
git commit -F COMMIT_MESSAGE.txt
```

**Alternative (inline commit):**

```bash
git commit -m "feat: Grammar refactor for command parser (v0.11.0-alpha2)"
```

### 4. Create Annotated Tag

```bash
git tag -a v0.11.0-alpha2 -F TAG_MESSAGE.txt
```

**Alternative (inline tag):**

```bash
git tag -a v0.11.0-alpha2 -m "Baseball Scorer v0.11.0-alpha2 — Grammar refactor"
```

### 5. Push Changes and Tags

```bash
git push origin main
git push origin v0.11.0-alpha2
```

**Push all tags:**

```bash
git push origin --tags
```

---

## Verification

### View Commit Log

```bash
git log --oneline -5
```

### View Tag Details

```bash
git show v0.11.0-alpha2
```

### List All Tags

```bash
git tag -l
```

---

## Quick Release (One-liner)

```bash
git add . && \
git commit -F COMMIT_MESSAGE.txt && \
git tag -a v0.11.0-alpha2 -F TAG_MESSAGE.txt && \
git push origin main --tags
```

---

## GitHub Release (Optional)

After pushing the tag, create a GitHub Release:

1. Go to: https://github.com/umpire274/bs_scoring/releases/new
2. Select tag: `v0.11.0-alpha2`
3. Release title: `v0.11.0-alpha2 — Grammar refactor for command parser`
4. Description: Copy content from `TAG_MESSAGE.txt`
5. For alpha / beta / RC tags, flag **"Set as a pre-release"**
6. Attach assets (optional):
    - Pre-compiled binaries
    - CHANGELOG.md
    - SCORING_GUIDE.md
7. Click "Publish release"

---

## Rollback (if needed)

### Delete local tag

```bash
git tag -d v0.11.0-alpha2
```

### Delete remote tag

```bash
git push origin :refs/tags/v0.11.0-alpha2
```

### Revert commit

```bash
git revert HEAD
```

---

## Version Bumping for Next Release

For the next release (e.g. `v0.11.0-alpha3` or `v0.11.0` final):

1. Update `Cargo.toml`: `version = "<new-version>"`
2. Update `Cargo.lock`: `version = "<new-version>"` under the
   `name = "bs_scoring"` entry.
3. Add a new section at the top of `CHANGELOG.md` following the
   [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.
4. Update the version number in the headers of `README.md`,
   `STRUCTURE.md`, and `SCORING_GUIDE.md` if they reference a specific
   release.
5. Create a new `COMMIT_MESSAGE.txt` and `TAG_MESSAGE.txt` for the
   release, reusing the structure of the previous ones.
6. Repeat the release process above.

---

**Current Version:** 0.11.0-alpha2
**Previous Version:** 0.11.0-alpha1
