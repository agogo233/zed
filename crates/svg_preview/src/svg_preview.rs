use gpui::{App, actions};
use workspace::Workspace;

pub mod svg_preview_view;

pub use zed_actions::preview::svg::{OpenPreview, OpenPreviewToTheSide};

actions!(
    svg,
    [
        /// 打开与编辑器同步的跟随式 SVG 预览
        OpenFollowingPreview
    ]
);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        crate::svg_preview_view::SvgPreviewView::register(workspace, window, cx);
    })
    .detach();
}
