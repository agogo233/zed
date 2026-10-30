use gpui::{App, actions};
use workspace::Workspace;

pub mod markdown_preview_settings;
pub mod markdown_preview_view;

pub use zed_actions::preview::markdown::{OpenPreview, OpenPreviewToTheSide};

use crate::markdown_preview_view::MarkdownPreviewView;

actions!(
    markdown,
    [
        /// 在 Markdown 预览中向上滚动一页
        #[action(deprecated_aliases = ["markdown::MovePageUp"])]
        ScrollPageUp,
        /// 在 Markdown 预览中向下滚动一页
        #[action(deprecated_aliases = ["markdown::MovePageDown"])]
        ScrollPageDown,
        /// 向上滚动约一个可视行
        ScrollUp,
        /// 向下滚动约一个可视行
        ScrollDown,
        /// 在 Markdown 预览中向上滚动一个 Markdown 元素
        ScrollUpByItem,
        /// 在 Markdown 预览中向下滚动一个 Markdown 元素
        ScrollDownByItem,
        /// 滚动到 Markdown 预览顶部
        ScrollToTop,
        /// 滚动到 Markdown 预览底部
        ScrollToBottom,
        /// 打开与编辑器同步的跟随式 Markdown 预览
        OpenFollowingPreview,
        /// 关闭 Markdown 预览并将焦点返回原编辑器
        CloseAndReturnToEditor
    ]
);

pub fn init(cx: &mut App) {
    workspace::register_serializable_item::<MarkdownPreviewView>(cx);

    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        markdown_preview_view::MarkdownPreviewView::register(workspace, window, cx);
    })
    .detach();
}
