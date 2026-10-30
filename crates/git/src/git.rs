pub mod blame;
pub mod commit;
mod hosting_provider;
mod remote;
pub mod repository;
pub mod stash;
pub mod status;

pub use crate::hosting_provider::*;
pub use crate::remote::*;
use anyhow::Result;
use gpui::{Action, actions};
pub use repository::RemoteCommandOutput;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Write as _};
use std::str::FromStr;

pub const DOT_GIT: &str = ".git";
pub const GITIGNORE: &str = ".gitignore";
pub const FSMONITOR_DAEMON: &str = "fsmonitor--daemon";
pub const LFS_DIR: &str = "lfs";
pub const OBJECTS_DIR: &str = "objects";
pub const REFS_DIR: &str = "refs";
pub const REFTABLE_DIR: &str = "reftable";
pub const HOOKS_DIR: &str = "hooks";
pub const LOGS_DIR: &str = "logs";
pub const LOGS_REF_STASH: &str = "logs/refs/stash";
pub const REBASE_MERGE_DIR: &str = "rebase-merge";
pub const REBASE_APPLY_DIR: &str = "rebase-apply";
pub const SEQUENCER_DIR: &str = "sequencer";
pub const COMMIT_MESSAGE: &str = "COMMIT_EDITMSG";
pub const FETCH_HEAD: &str = "FETCH_HEAD";
pub const ORIG_HEAD: &str = "ORIG_HEAD";
pub const BISECT_LOG: &str = "BISECT_LOG";
pub const GC_PID: &str = "gc.pid";
pub const INFO_DIR: &str = "info";
pub const REPO_EXCLUDE: &str = "info/exclude";

actions!(
    git,
    [
        // per-hunk
        /// 切换光标处区块或状态条目的暂存状态
        ToggleStaged,
        /// 暂存锚点条目与光标之间的状态条目
        StageRange,
        /// 暂存当前区块并移动到下一个
        StageAndNext,
        /// 取消暂存当前区块并移至下一个
        UnstageAndNext,
        /// 将选定区块恢复到原始状态
        #[action(deprecated_aliases = ["editor::RevertSelectedHunks"])]
        Restore,
        /// 将选定区块恢复到原始状态，
        /// 并移至下一个区块
        RestoreAndNext,
        // per-file
        /// 显示当前文件的 Git 追溯信息
        #[action(deprecated_aliases = ["editor::ToggleGitBlame"])]
        Blame,
        /// 显示所选文件、文件夹或项目的 Git 历史
        FileHistory,
        /// 打开所选文件在其 Git 托管提供商上的永久链接
        OpenFilePermalink,
        /// 复制所选文件在其 Git 托管提供商上的永久链接
        CopyFilePermalink,
        /// 在编辑器中打开所选文件，不显示差异视图
        ViewFile,
        /// 暂存当前文件
        StageFile,
        /// 取消暂存当前文件
        UnstageFile,
        // per-section
        /// 暂存所选条目所在分组中的所有条目
        StageSection,
        /// 取消暂存所选条目所在分组中的所有条目
        UnstageSection,
        // repo-wide
        /// 暂存仓库中的所有更改
        StageAll,
        /// 取消暂存仓库中的所有更改
        UnstageAll,
        /// 贮藏仓库中的所有更改，包括未跟踪的文件
        StashAll,
        /// 贮藏仓库中已跟踪的更改，保留未跟踪的文件
        StashTracked,
        /// 贮藏仓库中已暂存的更改，保留未暂存的更改
        StashStaged,
        /// 弹出最近的贮藏
        StashPop,
        /// 应用最近的贮藏
        StashApply,
        /// 将所有已跟踪文件恢复到上次提交的状态
        RestoreTrackedFiles,
        /// 将所有未跟踪文件移至回收站
        TrashUntrackedFiles,
        /// 撤销上次提交，保留工作目录中的更改
        Uncommit,
        /// 将提交推送到远程仓库
        Push,
        /// 将提交推送到特定的远程分支
        PushTo,
        /// 强制推送提交至远程仓库
        ForcePush,
        /// 从远程仓库拉取更改
        Pull,
        /// 从远程仓库拉取更改并进行变基
        PullRebase,
        /// 从远程仓库抓取更改
        Fetch,
        /// 从指定远程抓取更改
        FetchFrom,
        /// 使用已暂存的更改创建新提交
        Commit,
        /// 使用 `git commit --no-verify` 执行下一次提交
        SkipHooks,
        /// 使用已暂存的更改修正上次提交
        Amend,
        /// 启用 --signoff 选项
        Signoff,
        /// 取消当前 Git 操作
        Cancel,
        /// 展开提交消息编辑器
        ExpandCommitEditor,
        /// 切换提交消息编辑器是否填满
        /// Git 面板内的垂直空间
        ToggleFillCommitEditor,
        /// 切换是否在 Git 面板中显示提交消息编辑器
        ToggleCommitEditor,
        /// 使用 AI 生成提交消息
        GenerateCommitMessage,
        /// 初始化新的 Git 仓库
        Init,
        /// 在编辑器中打开所有已修改的文件
        OpenModifiedFiles,
        /// 在独立差异视图中打开当前文件
        OpenFileDiff,
        /// 克隆仓库
        Clone,
        ViewCommit,
        /// 将文件添加到 .gitignore
        AddToGitignore,
        /// 将文件添加到仓库的 .git/info/exclude
        AddToGitInfoExclude,
        /// 将当前分支名称复制到剪贴板
        CopyBranchName,
        /// 在 HEAD 创建标签
        CreateTagAtHead,
    ]
);

/// 重命名 Git 分支
#[derive(Clone, Debug, Default, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = git)]
#[serde(deny_unknown_fields)]
pub struct RenameBranch {
    /// The branch to rename.
    ///
    /// Default: the current branch.
    #[serde(default)]
    pub branch: Option<String>,
}

/// 将文件恢复到上次提交的状态，丢弃本地更改
#[derive(Clone, Debug, Default, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = git, deprecated_aliases = ["editor::RevertFile"])]
#[serde(deny_unknown_fields)]
pub struct RestoreFile {
    #[serde(default)]
    pub skip_prompt: bool,
}

/// The length of a Git short SHA.
pub const SHORT_SHA_LENGTH: usize = 7;

const SHA1_BYTE_LENGTH: usize = 20;
const SHA256_BYTE_LENGTH: usize = 32;
const SHA1_HEX_LENGTH: usize = SHA1_BYTE_LENGTH * 2;
const SHA256_HEX_LENGTH: usize = SHA256_BYTE_LENGTH * 2;
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Oid {
    bytes: [u8; SHA256_BYTE_LENGTH],
    format: OidFormat,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum OidFormat {
    Sha1,
    Sha256,
}

impl OidFormat {
    fn byte_len(self) -> usize {
        match self {
            Self::Sha1 => SHA1_BYTE_LENGTH,
            Self::Sha256 => SHA256_BYTE_LENGTH,
        }
    }

    fn hex_len(self) -> usize {
        match self {
            Self::Sha1 => SHA1_HEX_LENGTH,
            Self::Sha256 => SHA256_HEX_LENGTH,
        }
    }
}

impl Oid {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let format = match bytes.len() {
            SHA1_BYTE_LENGTH => OidFormat::Sha1,
            SHA256_BYTE_LENGTH => OidFormat::Sha256,
            len => {
                anyhow::bail!(
                    "invalid git oid byte length: expected {SHA1_BYTE_LENGTH} for SHA-1 or {SHA256_BYTE_LENGTH} for SHA-256, got {len}"
                );
            }
        };

        let mut oid_bytes = [0u8; SHA256_BYTE_LENGTH];
        oid_bytes[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            bytes: oid_bytes,
            format,
        })
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        let mut bytes = [0u8; SHA256_BYTE_LENGTH];
        rng.fill(&mut bytes[..SHA1_BYTE_LENGTH]);
        Self {
            bytes,
            format: OidFormat::Sha1,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.format.byte_len()]
    }

    pub fn is_zero(&self) -> bool {
        self.as_bytes().iter().all(|byte| *byte == 0)
    }

    /// Returns this [`Oid`] as a short SHA.
    pub fn display_short(&self) -> String {
        self.hex_string(SHORT_SHA_LENGTH)
    }

    fn hex_string(&self, len: usize) -> String {
        let mut string = String::with_capacity(len);
        for index in 0..len {
            string.push(self.hex_digit(index));
        }
        string
    }

    fn write_hex(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for index in 0..self.format.hex_len() {
            f.write_char(self.hex_digit(index))?;
        }
        Ok(())
    }

    #[inline(always)]
    fn hex_digit(&self, index: usize) -> char {
        debug_assert!(index < self.format.hex_len());
        let byte = self.as_bytes()[index / 2];
        let nibble = if index & 1 == 0 {
            byte >> 4
        } else {
            byte & 0x0f
        };
        char::from(HEX_DIGITS[nibble as usize])
    }
}

impl TryFrom<&str> for Oid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Oid::from_str(value)
    }
}

impl FromStr for Oid {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let format = match s.len() {
            1..=SHA1_HEX_LENGTH => OidFormat::Sha1,
            SHA256_HEX_LENGTH => OidFormat::Sha256,
            len => {
                anyhow::bail!(
                    "invalid git oid hex length: expected 1..={SHA1_HEX_LENGTH} for SHA-1 or {SHA256_HEX_LENGTH} for SHA-256, got {len}"
                );
            }
        };

        let mut bytes = [0u8; SHA256_BYTE_LENGTH];
        for (index, byte) in s.bytes().enumerate() {
            let digit = decode_hex_digit(byte)
                .ok_or_else(|| anyhow::anyhow!("invalid hex digit at byte {index} for git oid"))?;
            if index % 2 == 0 {
                bytes[index / 2] = digit << 4;
            } else {
                bytes[index / 2] |= digit;
            }
        }

        Ok(Self { bytes, format })
    }
}

fn decode_hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

impl fmt::Debug for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_hex(f)
    }
}

impl Serialize for Oid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.hex_string(self.format.hex_len()))
    }
}

impl<'de> Deserialize<'de> for Oid {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Oid>().map_err(serde::de::Error::custom)
    }
}

impl From<Oid> for u32 {
    fn from(oid: Oid) -> Self {
        let mut u32_bytes = [0u8; 4];
        u32_bytes.copy_from_slice(&oid.as_bytes()[..4]);
        u32::from_ne_bytes(u32_bytes)
    }
}

impl From<Oid> for usize {
    fn from(oid: Oid) -> Self {
        let mut u64_bytes = [0u8; 8];
        u64_bytes.copy_from_slice(&oid.as_bytes()[..8]);
        u64::from_ne_bytes(u64_bytes) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_short_sha1_oid_by_padding_trailing_nibbles() {
        let oid = "abc1234".parse::<Oid>().expect("failed to parse oid");

        assert_eq!(oid.as_bytes().len(), SHA1_BYTE_LENGTH);
        assert_eq!(oid.display_short(), "abc1234");
        assert_eq!(oid.to_string(), "abc1234000000000000000000000000000000000");
    }

    #[test]
    fn parses_full_sha256_oid() {
        let sha = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let oid = sha.parse::<Oid>().expect("failed to parse oid");

        assert_eq!(oid.as_bytes().len(), SHA256_BYTE_LENGTH);
        assert_eq!(oid.to_string(), sha);
    }

    #[test]
    fn rejects_invalid_oid_lengths() {
        assert!("".parse::<Oid>().is_err());
        assert!("a".repeat(SHA1_HEX_LENGTH + 1).parse::<Oid>().is_err());
        assert!("a".repeat(SHA256_HEX_LENGTH - 1).parse::<Oid>().is_err());
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum RunHook {
    PreCommit,
}

impl RunHook {
    pub fn as_str(&self) -> &str {
        match self {
            Self::PreCommit => "pre-commit",
        }
    }

    pub fn to_proto(&self) -> i32 {
        *self as i32
    }

    pub fn from_proto(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::PreCommit),
            _ => None,
        }
    }
}
