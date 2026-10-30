use client::ZED_URL_SCHEME;
use gpui::{AsyncApp, actions};

actions!(
    cli,
    [
        /// 注册 zed:// URL 方案处理程序
        RegisterZedScheme
    ]
);

pub async fn register_zed_scheme(cx: &AsyncApp) -> anyhow::Result<()> {
    cx.update(|cx| cx.register_url_scheme(ZED_URL_SCHEME)).await
}
