//! This module contains all actions supported by [`Editor`].
use super::*;
use gpui::{Action, actions};
use project::project_settings::GoToDiagnosticSeverityFilter;
use schemars::JsonSchema;
use util::serde::default_true;

/// 选中当前选区的下一个匹配位置
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct SelectNext {
    #[serde(default)]
    pub replace_newest: bool,
}

/// 选中当前选区的上一个匹配位置
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct SelectPrevious {
    #[serde(default)]
    pub replace_newest: bool,
}

/// 将光标移到当前行的开头
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct MoveToBeginningOfLine {
    #[serde(default = "default_true")]
    pub stop_at_soft_wraps: bool,
    #[serde(default)]
    pub stop_at_indent: bool,
}

/// 从光标选择到当前行的开头
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct SelectToBeginningOfLine {
    #[serde(default)]
    pub(super) stop_at_soft_wraps: bool,
    #[serde(default)]
    pub stop_at_indent: bool,
}

/// 从光标删除到当前行行首
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct DeleteToBeginningOfLine {
    #[serde(default)]
    pub(super) stop_at_indent: bool,
}

/// 将光标向上移动一页
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct MovePageUp {
    #[serde(default)]
    pub(super) center_cursor: bool,
}

/// 将光标向下移动一页
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct MovePageDown {
    #[serde(default)]
    pub(super) center_cursor: bool,
}

/// 将光标移到当前行的末尾
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct MoveToEndOfLine {
    #[serde(default = "default_true")]
    pub stop_at_soft_wraps: bool,
}

/// 从光标选择到当前行的末尾
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct SelectToEndOfLine {
    #[serde(default)]
    pub(super) stop_at_soft_wraps: bool,
}

/// 切换光标处可用代码操作的显示
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ToggleCodeActions {
    // Source from which the action was deployed.
    #[serde(default)]
    #[serde(skip)]
    pub deployed_from: Option<CodeActionSource>,
    // Run first available task if there is only one.
    #[serde(default)]
    #[serde(skip)]
    pub quick_launch: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub enum CodeActionSource {
    Indicator(DisplayRow),
    RunMenu(DisplayRow),
    QuickActionBar,
}

/// 确认并接受当前选中的补全建议
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ConfirmCompletion {
    #[serde(default)]
    pub item_ix: Option<usize>,
}

/// 将多个补全建议合并为单个补全
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ComposeCompletion {
    #[serde(default)]
    pub item_ix: Option<usize>,
}

/// 确认并应用当前选中的代码操作
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ConfirmCodeAction {
    #[serde(default)]
    pub item_ix: Option<usize>,
}

/// 切换所选行的注释标记
#[derive(PartialEq, Clone, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ToggleComments {
    #[serde(default)]
    pub advance_downwards: bool,
    #[serde(default)]
    pub ignore_indent: bool,
    /// Whether to add comment markers to blank lines inside a multi-line
    /// selection. A line of only whitespace counts as blank. Defaults to true.
    #[serde(default = "default_true")]
    pub comment_empty_lines: bool,
}

// `Default` is written out rather than derived because `comment_empty_lines`
// defaults to true.
impl Default for ToggleComments {
    fn default() -> Self {
        Self {
            advance_downwards: false,
            ignore_indent: false,
            comment_empty_lines: true,
        }
    }
}

/// 切换所选文本的块注释标记
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ToggleBlockComments;

/// 将光标向上移动指定行数
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct MoveUpByLines {
    #[serde(default)]
    pub(super) lines: u32,
}

/// 将光标向下移动指定行数
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct MoveDownByLines {
    #[serde(default)]
    pub(super) lines: u32,
}

/// 将选区向上扩展指定的行数
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct SelectUpByLines {
    #[serde(default)]
    pub(super) lines: u32,
}

/// 将选区向下扩展指定的行数
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct SelectDownByLines {
    #[serde(default)]
    pub(super) lines: u32,
}

/// 展开所有包含选区的摘录
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ExpandExcerpts {
    #[serde(default)]
    pub(super) lines: u32,
}

/// 展开当前位置上方的摘录
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ExpandExcerptsUp {
    #[serde(default)]
    pub(super) lines: u32,
}

/// 展开当前位置下方的摘录
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ExpandExcerptsDown {
    #[serde(default)]
    pub(super) lines: u32,
}

/// 处理编辑器中的文本输入
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
pub struct HandleInput(pub String);

/// 从光标删除到下一个单词的末尾
/// 如果遇到长度 >= 2 的空白序列，则在下一个单词末尾之前停止
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct DeleteToNextWordEnd {
    #[serde(default)]
    pub ignore_newlines: bool,
    // Whether to stop before the end of the next word, if language-defined bracket is encountered.
    #[serde(default)]
    pub ignore_brackets: bool,
}

/// 从光标删除到上一个单词的开头
/// 如果遇到长度 >= 2 的空白序列，则在前一个单词开头之前停止
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct DeleteToPreviousWordStart {
    #[serde(default)]
    pub ignore_newlines: bool,
    // Whether to stop before the start of the previous word, if language-defined bracket is encountered.
    #[serde(default)]
    pub ignore_brackets: bool,
}

/// 从光标删除到下一个子词的末尾
/// 如果遇到长度 >= 2 的空白序列，则在下一个子词末尾之前停止
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct DeleteToNextSubwordEnd {
    #[serde(default)]
    pub ignore_newlines: bool,
    // Whether to stop before the start of the previous word, if language-defined bracket is encountered.
    #[serde(default)]
    pub ignore_brackets: bool,
}

/// 从光标删除到上一个子词的开头
/// 如果遇到长度 >= 2 的空白序列，则在前一个子词开头之前停止
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct DeleteToPreviousSubwordStart {
    #[serde(default)]
    pub ignore_newlines: bool,
    // Whether to stop before the start of the previous word, if language-defined bracket is encountered.
    #[serde(default)]
    pub ignore_brackets: bool,
}

/// 从光标剪切到行尾
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct CutToEndOfLine {
    #[serde(default)]
    pub stop_at_newlines: bool,
}

/// 折叠指定缩进级别的所有代码块
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
pub struct FoldAtLevel(pub u32);

/// 从当前光标位置运行最近的可用任务
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct SpawnNearestTask {
    #[serde(default)]
    pub reveal: task::RevealStrategy,
}

#[derive(Clone, PartialEq, Action)]
#[action(no_json, no_register)]
pub struct DiffClipboardWithSelectionData {
    pub clipboard_text: String,
    pub editor: Entity<Editor>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Default)]
pub enum UuidVersion {
    #[default]
    V4,
    V7,
}

/// 将选区拆分为单独的行
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct SplitSelectionIntoLines {
    /// Keep the text selected after splitting instead of collapsing to cursors.
    #[serde(default)]
    pub keep_selections: bool,
}

/// 如果诊断尚未激活，则展开光标下的诊断（如果有）。
/// 否则，转到文件中的下一个诊断。
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct GoToDiagnostic {
    #[serde(default)]
    pub severity: GoToDiagnosticSeverityFilter,
}

/// 如果诊断尚未激活，则展开光标下的诊断（如果有）。
/// 否则，转到文件中的上一个诊断。
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct GoToPreviousDiagnostic {
    #[serde(default)]
    pub severity: GoToDiagnosticSeverityFilter,
}

/// 在当前选区上方添加光标
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct AddSelectionAbove {
    #[serde(default = "default_true")]
    pub skip_soft_wrap: bool,
}

/// 在当前选区下方添加光标
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct AddSelectionBelow {
    #[serde(default = "default_true")]
    pub skip_soft_wrap: bool,
}

/// 在光标处插入代码片段
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct InsertSnippet {
    /// Language name if using a named snippet, or `None` for a global snippet
    ///
    /// This is typically lowercase and matches the filename containing the snippet, without the `.json` extension.
    pub language: Option<String>,
    /// Name if using a named snippet
    pub name: Option<String>,

    /// Snippet body, if not using a named snippet
    // todo(andrew): use `ListOrDirect` or similar for multiline snippet body
    pub snippet: Option<String>,
}

actions!(
    debugger,
    [
        /// 运行程序至当前光标位置
        RunToCursor,
        /// 在调试器上下文中对所选文本求值
        EvaluateSelectedText
    ]
);

actions!(
    go_to_line,
    [
        /// 切换跳转到行对话框
        #[action(name = "Toggle")]
        ToggleGoToLine
    ]
);

actions!(
    markdown,
    [
        /// 切换选中行（或当前行）的块引用（`> `）前缀。
        /// 此操作仅适用于 Markdown 文件。
        ToggleBlockQuote,
    ]
);

actions!(
    editor,
    [
        /// 接受完整的编辑预测
        AcceptEditPrediction,
        /// 将内嵌提示的编辑（若有提供）应用到编辑器文本
        AcceptInlayHint,
        /// 接受部分编辑预测
        #[action(deprecated_aliases = ["editor::AcceptPartialCopilotSuggestion"])]
        AcceptNextWordEditPrediction,
        AcceptNextLineEditPrediction,
        /// 应用编辑器中的所有差异区块
        ApplyAllDiffHunks,
        /// 应用当前位置的差异区块
        ApplyDiffHunk,
        /// 删除光标前的字符
        Backspace,
        /// 显示当前行的 Git 追溯信息
        BlameHover,
        /// 取消当前操作
        Cancel,
        /// 取消正在运行的 flycheck 操作
        CancelFlycheck,
        /// 取消待处理的语言服务器工作
        CancelLanguageServerWork,
        /// 清除 flycheck 结果
        ClearFlycheck,
        /// 确认重命名操作
        ConfirmRename,
        /// 通过在光标处插入来确认补全
        ConfirmCompletionInsert,
        /// 通过替换现有文本来确认补全
        ConfirmCompletionReplace,
        /// 跳转到上下文菜单的第一项
        ContextMenuFirst,
        /// 跳转到上下文菜单的最后一项
        ContextMenuLast,
        /// 跳转到上下文菜单的下一项
        ContextMenuNext,
        /// 跳转到上下文菜单的上一项
        ContextMenuPrevious,
        /// 将缩进从制表符转换为空格
        ConvertIndentationToSpaces,
        /// 将缩进从空格转换为制表符
        ConvertIndentationToTabs,
        /// 将所选文本转换为 kebab-case
        ConvertToKebabCase,
        /// 将所选文本转换为 lowerCamelCase
        ConvertToLowerCamelCase,
        /// 将所选文本转换为全小写
        ConvertToLowerCase,
        /// 切换选中文本的大小写
        ConvertToOppositeCase,
        /// 将所选文本转换为句首字母大写
        ConvertToSentenceCase,
        /// 将所选文本转换为 snake_case
        ConvertToSnakeCase,
        /// 将所选文本转换为标题大小写
        ConvertToTitleCase,
        /// 将所选文本转换为 UpperCamelCase
        ConvertToUpperCamelCase,
        /// 将所选文本转换为全大写
        ConvertToUpperCase,
        /// 对选中文本应用 ROT13 密码
        ConvertToRot13,
        /// 对选中文本应用 ROT47 密码
        ConvertToRot47,
        /// 对选中的文本或光标下的单词进行 Base64 编码
        ConvertToBase64,
        /// 对选中的文本或光标下的单词进行 Base64 解码
        ConvertFromBase64,
        /// 将所选文本复制到剪贴板
        Copy,
        /// 将所选文本去除首尾空白后复制到剪贴板
        CopyAndTrim,
        /// 将当前文件位置复制到剪贴板
        CopyFileLocation,
        /// 将高亮文本以 JSON 格式复制
        CopyHighlightJson,
        /// 将当前文件名复制到剪贴板
        CopyFileName,
        /// 将不含文件扩展名的文件名复制到剪贴板
        CopyFileNameWithoutExtension,
        /// 复制当前行或选区的永久链接
        CopyPermalinkToLine,
        /// 将选中文本剪切到剪贴板
        Cut,
        /// 删除光标后的字符
        Delete,
        /// 删除当前行
        DeleteLine,
        /// 从光标删除到行尾
        DeleteToEndOfLine,
        /// 将剪贴板中的文本与当前选区进行差异比较
        DiffClipboardWithSelection,
        /// 显示所有活动光标的名称
        DisplayCursorNames,
        /// 在当前行下方复制当前行
        DuplicateLineDown,
        /// 在当前行上方复制当前行
        DuplicateLineUp,
        /// 复制当前选区
        DuplicateSelection,
        /// 展开编辑器中的所有差异区块
        #[action(deprecated_aliases = ["editor::ExpandAllHunkDiffs"])]
        ExpandAllDiffHunks,
        /// 折叠编辑器中所有差异区块
        CollapseAllDiffHunks,
        /// 切换编辑器中的所有差异区块。如果有任何区块已展开，
        /// 则折叠所有区块，否则展开所有区块
        ToggleAllDiffHunks,
        /// 在光标位置递归展开宏
        ExpandMacroRecursively,
        /// 查找搜索中的下一个匹配项
        FindNextMatch,
        /// 查找搜索中的上一个匹配项
        FindPreviousMatch,
        /// 折叠当前代码块
        Fold,
        /// 折叠编辑器中所有可折叠区域
        FoldAll,
        /// 折叠编辑器中缩进级别 1 的所有代码块
        #[action(name = "FoldAtLevel_1")]
        FoldAtLevel1,
        /// 折叠编辑器中缩进级别 2 的所有代码块
        #[action(name = "FoldAtLevel_2")]
        FoldAtLevel2,
        /// 折叠编辑器中缩进级别 3 的所有代码块
        #[action(name = "FoldAtLevel_3")]
        FoldAtLevel3,
        /// 折叠编辑器中缩进级别 4 的所有代码块
        #[action(name = "FoldAtLevel_4")]
        FoldAtLevel4,
        /// 折叠编辑器中缩进级别 5 的所有代码块
        #[action(name = "FoldAtLevel_5")]
        FoldAtLevel5,
        /// 折叠编辑器中缩进级别 6 的所有代码块
        #[action(name = "FoldAtLevel_6")]
        FoldAtLevel6,
        /// 折叠编辑器中缩进级别 7 的所有代码块
        #[action(name = "FoldAtLevel_7")]
        FoldAtLevel7,
        /// 折叠编辑器中缩进级别 8 的所有代码块
        #[action(name = "FoldAtLevel_8")]
        FoldAtLevel8,
        /// 折叠编辑器中缩进级别 9 的所有代码块
        #[action(name = "FoldAtLevel_9")]
        FoldAtLevel9,
        /// 折叠编辑器中所有函数体
        FoldFunctionBodies,
        /// 折叠当前代码块及其所有子块
        FoldRecursive,
        /// 折叠选中的范围
        FoldSelectedRanges,
        /// 将焦点切换回上次活动的缓冲区
        ToggleFocus,
        /// 切换当前位置的折叠状态
        ToggleFold,
        /// 切换当前位置的递归折叠
        ToggleFoldRecursive,
        /// 切换缓冲区中的所有折叠或多缓冲区中的所有摘录
        ToggleFoldAll,
        /// 格式化整个文档
        Format,
        /// 仅格式化所选文本
        ///
        /// 此操作仅在活动格式化程序支持范围格式化时可用。
        /// 使用语言服务器时，会为每个
        /// 选区发送 LSP 范围格式化请求；当所选缓冲区配置的语言服务器
        /// 未声明范围格式化支持时，该操作会被隐藏。使用 Prettier 时，Prettier 自带的范围
        /// 格式化用于对所有选区的涵盖范围进行格式化，产生的
        /// 所选范围之外的编辑将被丢弃。外部命令格式化程序
        /// 不支持范围格式化，将被跳过。
        FormatSelections,
        /// 在分割窗格中跳转到声明
        GoToDeclarationSplit,
        /// 在分割窗格中跳转到定义
        GoToDefinitionSplit,
        /// 跳转到下一个差异区块
        GoToHunk,
        /// 跳转到上一个差异区块
        GoToPreviousHunk,
        /// 在分割窗格中跳转到实现
        GoToImplementationSplit,
        /// 跳转到文件中的下一个书签
        GoToNextBookmark,
        /// 跳转到文件中的下一处更改
        GoToNextChange,
        /// 跳转到当前文件的父模块
        GoToParentModule,
        /// 跳转到文件中的上一个书签
        GoToPreviousBookmark,
        /// 跳转到文件中的上一处更改
        GoToPreviousChange,
        /// 跳转到下一个符号
        GoToNextSymbol,
        /// 跳转到上一个符号
        GoToPreviousSymbol,
        /// 跳转到光标下符号的下一个引用
        GoToNextReference,
        /// 跳转到光标下符号的上一个引用
        GoToPreviousReference,
        /// 在分割窗格中跳转到类型定义
        GoToTypeDefinitionSplit,
        /// 跳转到下一个文档高亮
        GoToNextDocumentHighlight,
        /// 跳转到上一个文档高亮
        GoToPreviousDocumentHighlight,
        /// 向下滚动半页
        HalfPageDown,
        /// 向上滚动半页
        HalfPageUp,
        /// 显示光标处符号的悬停信息
        Hover,
        /// 展开选中的调试器悬停条目
        DebuggerHoverExpandSelected,
        /// 折叠选中的调试器悬停条目
        DebuggerHoverCollapseSelected,
        /// 选中下一个调试器悬停条目
        DebuggerHoverSelectNext,
        /// 选中上一个调试器悬停条目
        DebuggerHoverSelectPrevious,
        /// 增加所选行的缩进
        Indent,
        /// 在光标位置插入 UUID v4
        InsertUuidV4,
        /// 在光标位置插入 UUID v7
        InsertUuidV7,
        /// 将当前行与下一行合并
        JoinLines,
        /// 剪切到 kill ring（Emacs 风格）
        KillRingCut,
        /// 从 kill ring 粘贴（Emacs 风格）
        KillRingYank,
        /// 将光标向下移动一行
        LineDown,
        /// 将光标向上移动一行
        LineUp,
        /// 将光标向左移动
        MoveLeft,
        /// 将当前行向下移动
        MoveLineDown,
        /// 将当前行向上移动
        MoveLineUp,
        /// 将光标向右移动
        MoveRight,
        /// 将光标移到文档开头
        MoveToBeginning,
        /// 将光标移到外层括号处
        MoveToEnclosingBracket,
        /// 选择最近一对包围定界符内的内容
        /// （方括号、花括号、圆括号或引号），不包括
        /// 定界符。
        /// 重复此操作会将选区扩展到下一对包围
        /// 定界符。
        SelectInsideDelimiters,
        /// 选择最近的一对包围定界符（方括号、花括号、
        /// 圆括号或引号）以及它们之间的内容。
        /// 重复此操作会将选区扩展到下一对包围
        /// 定界符。
        SelectAroundDelimiters,
        /// 将光标移到文档末尾
        MoveToEnd,
        /// 将光标移到段落末尾
        MoveToEndOfParagraph,
        /// 将光标移到下一个注释段落的开头
        MoveToNextCommentParagraph,
        /// 将光标移到下一个子词的末尾
        MoveToNextSubwordEnd,
        /// 将光标移到下一个单词的末尾
        MoveToNextWordEnd,
        /// 将光标移到上一个注释段落的开头
        MoveToPreviousCommentParagraph,
        /// 将光标移到上一个子词的开头
        MoveToPreviousSubwordStart,
        /// 将光标移到上一个单词的开头
        MoveToPreviousWordStart,
        /// 将光标移到段落开头
        MoveToStartOfParagraph,
        /// 将光标移到当前摘录的开头
        MoveToStartOfExcerpt,
        /// 将光标移到下一个摘录的开头
        MoveToStartOfNextExcerpt,
        /// 将光标移到当前摘录的末尾
        MoveToEndOfExcerpt,
        /// 将光标移到上一个摘录的末尾
        MoveToEndOfPreviousExcerpt,
        /// 将光标移到下一个更大语法节点的开头
        MoveToStartOfLargerSyntaxNode,
        /// 将光标移到下一个更大语法节点的末尾
        MoveToEndOfLargerSyntaxNode,
        /// 插入新行并将光标移至该行
        Newline,
        /// 在当前行上方插入新行
        NewlineAbove,
        /// 在当前行下方插入新行
        NewlineBelow,
        /// 跳转到下一个编辑预测
        NextEditPrediction,
        /// 滚动到下一屏
        NextScreen,
        /// 跳转到下一个代码片段制表位（如果存在）
        NextSnippetTabstop,
        /// 打开项目中所有书签的视图
        ViewBookmarks,
        /// 在光标位置打开上下文菜单
        OpenContextMenu,
        /// 打开当前文件的摘录
        OpenExcerpts,
        /// 在分割窗格中打开摘录
        OpenExcerptsSplit,
        /// 打开建议更改编辑器
        OpenProposedChangesEditor,
        /// 打开光标处符号的文档
        OpenDocs,
        /// 打开指向当前行或选区的永久链接
        OpenPermalinkToLine,
        /// 打开编辑器中所选文件名对应的文件
        #[action(deprecated_aliases = ["editor::OpenFile"])]
        OpenSelectedFilename,
        /// 在多缓冲区中打开所有选区
        OpenSelectionsInMultibuffer,
        /// 打开光标位置处的 URL
        OpenUrl,
        /// 整理导入语句
        OrganizeImports,
        /// 减少所选行的缩进
        Outdent,
        /// 根据上下文自动调整缩进
        AutoIndent,
        /// 向下滚动一页
        PageDown,
        /// 向上滚动一页
        PageUp,
        /// 从剪贴板粘贴
        Paste,
        /// 跳转到上一个编辑预测
        PreviousEditPrediction,
        /// 跳转到上一个代码片段制表位（如果存在）
        PreviousSnippetTabstop,
        /// 重做上次撤销的编辑
        Redo,
        /// 重做上次的选区更改
        RedoSelection,
        /// 重命名光标处的符号
        Rename,
        /// 重启当前文件的语言服务器
        RestartLanguageServer,
        /// 反转选定行的顺序
        ReverseLines,
        /// 从磁盘重新加载文件
        ReloadFile,
        /// 将文本重新换行以适应首选行长度
        Rewrap,
        /// 向后轮换选区或行
        RotateSelectionsBackward,
        /// 向前轮换选区或行
        RotateSelectionsForward,
        /// 运行 flycheck 诊断
        RunFlycheck,
        /// 滚动视口，使光标位于底部
        ScrollCursorBottom,
        /// 滚动视口，使光标位于中央
        ScrollCursorCenter,
        /// 依次滚动视口，使光标位于中央、顶部和底部
        ScrollCursorCenterTopBottom,
        /// 滚动视口，使光标位于顶部
        ScrollCursorTop,
        /// 选择编辑器中的所有文本
        SelectAll,
        /// 选择当前选区的所有匹配项
        SelectAllMatches,
        /// 选中至当前摘录开头
        SelectToStartOfExcerpt,
        /// 选中至下一个摘录开头
        SelectToStartOfNextExcerpt,
        /// 选中至当前摘录末尾
        SelectToEndOfExcerpt,
        /// 选中至上一个摘录的末尾
        SelectToEndOfPreviousExcerpt,
        /// 向下扩展选区
        SelectDown,
        /// 选中包含当前位置的符号
        SelectEnclosingSymbol,
        /// 选中至下一个更大语法节点的开头
        SelectToStartOfLargerSyntaxNode,
        /// 选中至下一个更大语法节点的末尾
        SelectToEndOfLargerSyntaxNode,
        /// 选中下一个更大的语法节点
        SelectLargerSyntaxNode,
        /// 选中下一个同级语法节点
        SelectNextSyntaxNode,
        /// 选中上一个同级语法节点
        SelectPreviousSyntaxNode,
        /// 向左扩展选区
        SelectLeft,
        /// 选中当前行
        SelectLine,
        /// 向下扩展选区一页
        SelectPageDown,
        /// 向上扩展选区一页
        SelectPageUp,
        /// 向右扩展选区
        SelectRight,
        /// 选中下一个更小的语法节点
        SelectSmallerSyntaxNode,
        /// 选中至文档开头
        SelectToBeginning,
        /// 选中至文档末尾
        SelectToEnd,
        /// 选中至段落末尾
        SelectToEndOfParagraph,
        /// 选中至下一个子词的末尾
        SelectToNextSubwordEnd,
        /// 选中至下一个单词的末尾
        SelectToNextWordEnd,
        /// 选中至上一个子词的开头
        SelectToPreviousSubwordStart,
        /// 选中至上一个单词的开头
        SelectToPreviousWordStart,
        /// 选中至段落开头
        SelectToStartOfParagraph,
        /// 向上扩展选区
        SelectUp,
        /// 在光标位置显示代码补全建议
        ShowCompletions,
        /// 显示系统字符面板
        ShowCharacterPalette,
        /// 在光标处显示编辑预测
        ShowEditPrediction,
        /// 显示当前函数的签名帮助
        ShowSignatureHelp,
        /// 显示单词补全
        ShowWordCompletions,
        /// 随机打乱所选行
        ShuffleLines,
        /// 跳转到签名帮助弹窗中的下一个签名
        SignatureHelpNext,
        /// 跳转到签名帮助弹窗中的上一个签名
        SignatureHelpPrevious,
        /// 按长度排序所选行
        SortLinesByLength,
        /// 不区分大小写地排序所选行
        SortLinesCaseInsensitive,
        /// 区分大小写地排序所选行
        SortLinesCaseSensitive,
        /// 停止当前文件的语言服务器
        StopLanguageServer,
        /// 在源文件和头文件之间切换
        SwitchSourceHeader,
        /// 插入制表符或缩进
        Tab,
        /// 移除制表符或减少缩进
        Backtab,
        /// 切换当前行的书签
        ToggleBookmark,
        /// 在当前行切换书签，添加时提示输入标签
        ToggleBookmarkWithLabel,
        /// 编辑当前行书签的标签
        EditBookmark,
        /// 切换当前行的断点
        ToggleBreakpoint,
        /// 切换选中文本的大小写
        ToggleCase,
        /// 禁用当前行的断点
        DisableBreakpoint,
        /// 启用当前行的断点
        EnableBreakpoint,
        /// 编辑断点的日志消息
        EditLogBreakpoint,
        /// 切换自动签名帮助
        ToggleAutoSignatureHelp,
        /// 切换内联 Git 追溯显示
        ToggleGitBlameInline,
        /// 打开光标处追溯信息对应的 Git 提交
        OpenGitBlameCommit,
        /// 打开光标处追溯条目对应修订版本的文件追溯
        BlameRevision,
        /// 打开光标处追溯条目上一个修订版本的文件追溯
        BlamePreviousRevision,
        /// 切换诊断面板
        ToggleDiagnostics,
        /// 切换缩进参考线显示
        ToggleIndentGuides,
        /// 切换内嵌提示显示
        ToggleInlayHints,
        /// 切换 Code Lens 显示
        ToggleCodeLens,
        /// 切换语义高亮显示
        ToggleSemanticHighlights,
        /// 切换内联值显示
        ToggleInlineValues,
        /// 切换内联诊断显示
        ToggleInlineDiagnostics,
        /// 切换编辑预测功能
        ToggleEditPrediction,
        /// 切换行号显示
        ToggleLineNumbers,
        /// 切换小地图显示
        ToggleMinimap,
        /// 交换当前选区的起点和终点
        SwapSelectionEnds,
        /// 在当前位置设置标记
        SetMark,
        /// 切换相对行号显示
        ToggleRelativeLineNumbers,
        /// 切换所选区块的差异显示
        #[action(deprecated_aliases = ["editor::ToggleHunkDiff"])]
        ToggleSelectedDiffHunks,
        /// 将差异审阅评论保存到本地（供后续批量提交）
        SubmitDiffReviewComment,
        /// 切换叠加层中评论区的展开状态
        ToggleReviewCommentsExpanded,
        /// 将所有已存储的审阅评论发送至智能体面板
        SendReviewToAgent,
        /// 切换选区菜单
        ToggleSelectionMenu,
        /// 切换自动换行模式
        ToggleSoftWrap,
        /// 切换标签栏显示
        ToggleTabBar,
        /// 转置光标处的字符
        Transpose,
        /// 撤销上次编辑
        Undo,
        /// 撤销上一次选区更改
        UndoSelection,
        /// 展开所有折叠区域
        UnfoldAll,
        /// 展开光标处的行
        UnfoldLines,
        /// 在光标处递归展开
        UnfoldRecursive,
        /// 移除重复行（不区分大小写）
        UniqueLinesCaseInsensitive,
        /// 移除重复行（区分大小写）
        UniqueLinesCaseSensitive,
        /// 移除环绕语法节点（例如括号或闭包）。
        /// 作用范围为当前选区。
        UnwrapSyntaxNode,
        /// 将选区包裹在语言指定的标签中
        WrapSelectionsInTag,
        /// 将选区包裹在展开后的 Emmet 缩写中
        WrapWithAbbreviation,
        /// 将不同行的选区对齐到同一列
        AlignSelections,
        /// 将当前位置保存到导航历史
        SaveLocation,
        /// 切换面包屑显示
        ToggleBreadcrumb,
    ]
);

/// 跳转到光标处符号的定义
#[derive(PartialEq, Clone, Default, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct GoToDefinition {
    /// Where to show the definitions. Falls back to the `lsp_results_location`
    /// setting when omitted. A single result is always opened directly.
    #[serde(default)]
    pub open_results_in: Option<OpenResultsIn>,
}

/// 跳转到光标处符号的声明
#[derive(PartialEq, Clone, Default, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct GoToDeclaration {
    /// Where to show the declarations. Falls back to the `lsp_results_location`
    /// setting when omitted. A single result is always opened directly.
    #[serde(default)]
    pub open_results_in: Option<OpenResultsIn>,
}

/// 跳转到光标处符号的实现
#[derive(PartialEq, Clone, Default, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct GoToImplementation {
    /// Where to show the implementations. Falls back to the `lsp_results_location`
    /// setting when omitted. A single result is always opened directly.
    #[serde(default)]
    pub open_results_in: Option<OpenResultsIn>,
}

/// 跳转到光标处符号的类型定义
#[derive(PartialEq, Clone, Default, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct GoToTypeDefinition {
    /// Where to show the definitions. Falls back to the `lsp_results_location`
    /// setting when omitted. A single result is always opened directly.
    #[serde(default)]
    pub open_results_in: Option<OpenResultsIn>,
}

/// 查找光标处符号的所有引用
#[derive(PartialEq, Clone, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct FindAllReferences {
    #[serde(default = "default_true")]
    pub always_open_multibuffer: bool,
    /// Where to show the references. Falls back to the `lsp_results_location`
    /// setting when omitted. A single result is always opened directly.
    #[serde(default)]
    pub open_results_in: Option<OpenResultsIn>,
}

impl Default for FindAllReferences {
    fn default() -> Self {
        Self {
            always_open_multibuffer: true,
            open_results_in: None,
        }
    }
}

/// 内联编辑已存储的审阅评论
#[derive(PartialEq, Clone, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct EditReviewComment {
    pub id: usize,
}

/// 删除已存储的审阅评论
#[derive(PartialEq, Clone, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct DeleteReviewComment {
    pub id: usize,
}

/// 确认审阅评论的内联编辑
#[derive(PartialEq, Clone, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct ConfirmEditReviewComment {
    pub id: usize,
}

/// 取消对审阅评论的内联编辑
#[derive(PartialEq, Clone, Deserialize, JsonSchema, Action)]
#[action(namespace = editor)]
#[serde(deny_unknown_fields)]
pub struct CancelEditReviewComment {
    pub id: usize,
}
