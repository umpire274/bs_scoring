# Git Release Instructions for v0.2.0

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

**Alternative (manual commit):**
```bash
git commit -m "feat: Add SQLite database and menu-driven interface (v0.2.0)"
```

### 4. Create Annotated Tag
```bash
git tag -a v0.2.0 -F TAG_MESSAGE.txt
```

**Alternative (inline tag):**
```bash
git tag -a v0.2.0 -m "Baseball Scorer v0.2.0 - Database & Menu System"
```

### 5. Push Changes and Tags
```bash
git push origin main
git push origin v0.2.0
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
git show v0.2.0
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
git tag -a v0.2.0 -F TAG_MESSAGE.txt && \
git push origin main --tags
```

---

## GitHub Release (Optional)

After pushing the tag, create a GitHub Release:

1. Go to: https://github.com/YOUR_USERNAME/baseball_scorer/releases/new
2. Select tag: `v0.2.0`
3. Release title: `v0.2.0 - Database & Menu System`
4. Description: Copy content from `TAG_MESSAGE.txt`
5. Attach assets (optional):
   - Pre-compiled binaries
   - CHANGELOG.md
   - SCORING_GUIDE.md
6. Click "Publish release"

---

## Rollback (if needed)

### Delete local tag
```bash
git tag -d v0.2.0
```

### Delete remote tag
```bash
git push origin :refs/tags/v0.2.0
```

### Revert commit
```bash
git revert HEAD
```

---

## Version Bumping for Next Release

For v0.3.0:
1. Update `Cargo.toml`: `version = "0.3.0"`
2. Add new section in `CHANGELOG.md`
3. Create new `COMMIT_MESSAGE.txt`
4. Create new `TAG_MESSAGE.txt`
5. Repeat release process

---

**Current Version:** 0.2.0  
**Previous Version:** 0.1.0  
**Next Planned:** 0.3.0 (Live scoring interface)
